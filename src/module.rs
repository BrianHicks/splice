use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use eyre::{Context, Result};
use tera::Tera;
use toml::Table;

use crate::config;

#[derive(Debug)]
pub struct Module {
    pub config: config::ModuleConfig,
    #[expect(dead_code)]
    args: Table,
    prefix: Option<PathBuf>,

    templates: Tera,
}

impl Module {
    #[tracing::instrument]
    pub fn from_dir(dir: &Path, args: Table, prefix: Option<PathBuf>) -> Result<Self> {
        let config = config::read(&dir.join(config::FILE_NAME))?;

        // TODO: validate args

        Ok(Module {
            config: config.try_module()?,
            args,
            prefix,

            templates: Tera::new(&dir.join("templates").join("**").join("*").to_string_lossy())
                .wrap_err_with(|| format!("could not load templates in `{}`", dir.display()))?,
        })
    }

    #[tracing::instrument]
    pub fn files(&self) -> Result<BTreeMap<PathBuf, String>> {
        let mut out = BTreeMap::new();

        // TODO: put args etc in here
        let context = tera::Context::new();
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
