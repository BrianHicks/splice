use std::path::{Path, PathBuf};

use eyre::{Result, bail};
use toml::Table;

use crate::config;

#[derive(Debug)]
pub struct Module {
    pub config: config::ModuleConfig,
    args: Table,
    prefix: Option<PathBuf>,
}

impl Module {
    pub fn from_dir(dir: &Path, args: Table, prefix: Option<PathBuf>) -> Result<Self> {
        let config = config::read(&dir.join(config::FILE_NAME))?;

        Ok(Module {
            config: config.try_module()?,
            args,
            prefix,
        })
    }
}
