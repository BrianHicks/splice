use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use eyre::{Context, Result, bail};
use tera::Tera;
use toml::{Table, Value};

use crate::config::{self, ModuleConfig};

#[derive(Debug)]
pub struct Module {
    pub config: config::ModuleConfig,
    args: BTreeMap<String, Value>,
    prefix: Option<PathBuf>,

    templates: Tera,
}

impl Module {
    #[tracing::instrument(skip(raw_args))]
    pub fn from_dir(dir: &Path, raw_args: Table, prefix: Option<PathBuf>) -> Result<Self> {
        let config = config::read(&dir.join(config::FILE_NAME))?.try_module()?;
        let args = Self::validate_args(&config, raw_args).wrap_err("while validating args")?;

        Ok(Module {
            config,
            args,
            prefix,

            templates: Tera::new(&dir.join("templates").join("**").join("*").to_string_lossy())
                .wrap_err_with(|| format!("could not load templates in `{}`", dir.display()))?,
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

    #[tracing::instrument]
    pub fn files(&self) -> Result<BTreeMap<PathBuf, String>> {
        let mut out = BTreeMap::new();

        let mut context = tera::Context::new();
        context
            .try_insert("args", &self.args)
            .wrap_err("could not serialize args to template")?;

        let prefix = self
            .prefix
            .as_deref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(PathBuf::from(".")));

        for template in self.templates.get_template_names() {
            // TODO: render templates in names as well
            out.insert(
                prefix.join(template),
                self.templates.render(template, &context)?,
            );
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
