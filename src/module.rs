use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use eyre::{Context, Result, bail, eyre};
use tera::{Tera, TeraResult};
use toml::{Table, Value};

use crate::config::{self, ModuleConfig};

// Comment markers are short (e.g. `#`, `//`, `<!--`), so we allow one to four
// non-whitespace characters followed by a single space before the directive.
static SPLICE_START: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^\S{1,4}\sSPLICE: (?<name>.+)$").unwrap());
static SPLICE_END: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^\S{1,4}\sSPLICE END").unwrap());

fn splice(kwargs: tera::Kwargs, state: &tera::State) -> TeraResult<String> {
    let comment = kwargs.must_get::<&str>("comment")?;
    let name = kwargs.must_get::<&str>("name")?;

    // Look up any preserved content for this splice by its exact name. We avoid
    // `get_from_path` here because splice names may contain `.`, which it would
    // interpret as nested-map traversal and fail to match.
    let splices = state.get::<tera::Map>("splices")?;
    let content = splices
        .as_ref()
        .and_then(|map| map.get(&tera::value::Key::Str(name)))
        .and_then(tera::Value::as_str)
        .unwrap_or_default();

    Ok(format!(
        "{comment} SPLICE: {name}\n{content}{comment} SPLICE END"
    ))
}

#[derive(Debug)]
pub struct Module {
    pub config: config::ModuleConfig,
    prefix: Option<PathBuf>,

    templates: Tera,
    splices: BTreeMap<PathBuf, BTreeMap<String, String>>,
}

impl Module {
    #[tracing::instrument(skip(raw_args))]
    pub fn from_dir(dir: &Path, raw_args: Table, prefix: Option<PathBuf>) -> Result<Self> {
        let config = config::read(&dir.join(config::FILE_NAME))?.try_module()?;

        let mut templates = Tera::new();
        templates.register_function("splice", splice);

        templates.global_context().insert(
            "args",
            &Self::validate_args(&config, raw_args).wrap_err("while validating args")?,
        );

        templates
            .load_from_glob(&dir.join("templates").join("**").join("*").to_string_lossy())
            .wrap_err_with(|| format!("could not load templates in `{}`", dir.display()))?;

        Ok(Module {
            config,
            prefix,
            templates,
            splices: BTreeMap::new(),
        })
    }

    fn validate_args(
        config: &ModuleConfig,
        mut raw_args: Table,
    ) -> Result<BTreeMap<String, Value>> {
        let mut validated = BTreeMap::new();

        // make sure every arg schema is internally consistent (e.g. defaults
        // match their types) before we start validating any caller input, so
        // we don't mix mistakes from modules vs applications.
        for (name, validator) in &config.args {
            validator
                .check()
                .wrap_err_with(|| format!("`{name}` has an invalid schema"))?;
        }

        for (name, validator) in &config.args {
            validated.insert(
                name.to_string(),
                validator
                    .validate(raw_args.remove(name))
                    .wrap_err_with(|| format!("`{name}` failed validation"))?,
            );
        }

        let remaining: Vec<String> = raw_args
            .into_iter()
            .map(|(k, _)| format!("`{k}`"))
            .collect();
        if !remaining.is_empty() {
            bail!("Unknown arguments: {}", remaining.join(", "))
        }

        Ok(validated)
    }

    fn files_of_interest(&self) -> Result<impl Iterator<Item = (PathBuf, &str)>> {
        let prefix = self
            .prefix
            .as_deref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(PathBuf::from(".")))
            .to_path_buf();

        // TODO: render templates in names as well
        Ok(self
            .templates
            .get_template_names()
            .map(move |template| (prefix.join(template), template)))
    }

    pub fn collect_splices(&mut self) -> Result<()> {
        let mut out = BTreeMap::new();

        for (filename, _) in self.files_of_interest()? {
            let mut splices = BTreeMap::new();

            let exists = filename.try_exists().wrap_err_with(|| {
                format!("could not check the existence of {}", filename.display())
            })?;
            if !exists {
                out.insert(filename, splices);
                continue;
            }

            let file = File::open(&filename)
                .wrap_err_with(|| format!("could not open `{}` for reading", filename.display()))?;
            let reader = BufReader::new(file);

            let mut state: Option<(String, String)> = None;

            for (i, line_res) in reader.lines().enumerate() {
                let line = line_res?;

                match state {
                    None => {
                        if let Some(c) = SPLICE_START.captures(&line) {
                            let name = c
                                .name("name")
                                .expect("missing name means the regex is misconfigured!")
                                .as_str()
                                .to_owned();

                            state = Some((name, String::new()))
                        } else if SPLICE_END.is_match(&line) {
                            return Err(eyre!("Found a SPLICE END on line {}", i + 1,))
                                .wrap_err(format!("in `{}`", filename.display()));
                        }
                    }
                    Some((name, mut splice)) => {
                        if SPLICE_START.is_match(&line) {
                            return Err(eyre!(
                                "Found a SPLICE on line {} before the end of the `{name}` splice.",
                                i + 1,
                            ))
                            .wrap_err(format!("in `{}`", filename.display()));
                        } else if SPLICE_END.is_match(&line) {
                            splices.insert(name, splice);
                            state = None;
                        } else {
                            splice.push_str(&line);
                            splice.push('\n'); // TODO: need to do \r\n for Windows?

                            state = Some((name, splice)) // TODO: does this allocate on every line?
                        }
                    }
                }
            }

            // A splice that opens but never closes would otherwise be silently
            // dropped on the next render, throwing away the user's content. Treat
            // it as an error so the unterminated block is fixed before we write.
            if let Some((name, _)) = state {
                return Err(eyre!(
                    "Reached the end of the file while still inside the `{name}` splice (missing SPLICE END)."
                ))
                .wrap_err(format!("in `{}`", filename.display()));
            }

            // TODO: migrations between files over time
            out.insert(filename, splices);
        }

        self.splices = out;

        Ok(())
    }

    fn render(&self, filename: &Path, template: &str) -> Result<String> {
        let mut context = tera::Context::new();
        context.insert("filename", &filename);
        context.insert("splices", &self.splices.get(filename));

        self.templates
            .render(template, &context)
            .wrap_err_with(|| format!("failed to render `{template}` to `{}`", filename.display()))
    }

    #[tracing::instrument]
    pub fn files(&self) -> Result<BTreeMap<PathBuf, String>> {
        let mut out = BTreeMap::new();

        for (filename, template) in self.files_of_interest()? {
            out.insert(filename.clone(), self.render(&filename, template)?);
        }

        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn config(src: &str) -> ModuleConfig {
        toml::from_str(src).unwrap()
    }

    fn args(src: &str) -> Table {
        toml::from_str(src).unwrap()
    }

    macro_rules! assert_args_fail {
        ($config:expr, $args:expr, $msg:expr) => {
            assert_eq!(
                $msg,
                Module::validate_args(&config($config), args($args))
                    .unwrap_err()
                    .chain()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(": ")
            )
        };
    }

    #[test]
    fn parses_values() {
        let validated = Module::validate_args(
            &config("args.name.type = \"string\"\nargs.count.type = \"int\""),
            args("name = \"hey\"\ncount = 3"),
        )
        .unwrap();

        let mut expected = BTreeMap::new();
        expected.insert("name".to_string(), Value::from("hey"));
        expected.insert("count".to_string(), Value::from(3));

        assert_eq!(expected, validated);
    }

    #[test]
    fn fills_in_defaults() {
        let validated =
            Module::validate_args(&config("args.name.default = \"hey\""), args("")).unwrap();

        let mut expected = BTreeMap::new();
        expected.insert("name".to_string(), Value::from("hey"));

        assert_eq!(expected, validated);
    }

    #[test]
    fn contextualizes_validation_failure() {
        assert_args_fail!(
            "args.name.type = \"string\"",
            "name = 1",
            "`name` failed validation: expected type string, but got a value of type integer"
        );
    }

    #[test]
    fn contextualizes_missing_required_arg() {
        assert_args_fail!(
            "args.name.type = \"string\"",
            "",
            "`name` failed validation: value is required"
        );
    }

    #[test]
    fn contextualizes_nested_failure() {
        assert_args_fail!(
            "args.tags.items.type = \"string\"",
            "tags = [1]",
            "`tags` failed validation: at index 0: expected type string, but got a value of type integer"
        );
    }

    #[test]
    fn rejects_unknown_argument() {
        assert_args_fail!("", "surprise = 1", "Unknown arguments: `surprise`");
    }

    #[test]
    fn rejects_invalid_schema_before_input() {
        // a bad default is the module author's mistake, so it should surface
        // even when the caller supplies a perfectly valid value for the arg.
        assert_args_fail!(
            "args.count.type = \"int\"\nargs.count.default = \"nope\"",
            "count = 3",
            "`count` has an invalid schema: default does not match the schema: expected type integer, but got a value of type string"
        );
    }

    #[test]
    fn rejects_unknown_arguments_sorted() {
        assert_args_fail!(
            "",
            "zed = 1\nalpha = 2",
            "Unknown arguments: `alpha`, `zed`"
        );
    }
}
