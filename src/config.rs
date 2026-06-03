use std::{
    collections::BTreeMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use eyre::{Context, Result, bail};
use serde::{Deserialize, Deserializer, Serialize};
use toml::Table;

use crate::validator::Validator;

pub const FILE_NAME: &str = "splice.toml";

pub fn read(path: &Path) -> Result<Config> {
    let contents = read_to_string(path)
        .wrap_err_with(|| format!("could not read config file at `{}`", path.display()))?;

    toml::from_str(&contents).wrap_err("could not parse config file as TOML")
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Config {
    App(AppConfig),
    Module(ModuleConfig),
}

impl Config {
    pub fn try_app(self) -> Result<AppConfig> {
        match self {
            Self::App(config) => Ok(config),
            Self::Module(_) => bail!("Expected an app config, but got a module config."),
        }
    }

    pub fn try_module(self) -> Result<ModuleConfig> {
        match self {
            Self::App(_) => bail!("Expected a module config, but got an app config."),
            Self::Module(config) => Ok(config),
        }
    }
}

impl<'de> Deserialize<'de> for Config {
    /// Deserializes as an app if `type` is absent, and according to the value
    /// of `type` otherwise.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper<'a> {
            #[serde(rename = "type")]
            type_: Option<&'a str>,
            #[serde(flatten)]
            rest: Table,
        }

        let Helper { type_, rest } = Helper::deserialize(deserializer)?;

        match type_ {
            Some("app") | None => rest
                .try_into()
                .map(Config::App)
                .map_err(serde::de::Error::custom),
            Some("module") => rest
                .try_into()
                .map(Config::Module)
                .map_err(serde::de::Error::custom),
            Some(otherwise) => Err(serde::de::Error::custom(format!(
                "Unexpected type `{otherwise}`. Expected `app` or `module`"
            ))),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ModuleConfig {
    #[serde(default, rename = "module")]
    pub modules: Vec<ModuleInvocation>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub args: BTreeMap<String, Validator>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AppConfig {
    #[serde(default, rename = "module")]
    pub modules: Vec<ModuleInvocation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ModuleInvocation {
    #[serde(default, skip_serializing_if = "Table::is_empty")]
    pub args: toml::Table,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<PathBuf>,
    #[serde(flatten)]
    pub location: ModuleLocation,
}

impl ModuleInvocation {
    /// Since modules can include other modules, and each module invocation can
    /// generate output at a different prefix, we need to keep track of nesting.
    pub fn inherit_prefix(&self, parent: &Path) -> Self {
        let prefix = Some(match &self.prefix {
            None => parent.to_path_buf(),
            Some(existing) => parent.join(existing),
        });
        Self {
            prefix,
            ..self.clone()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum ModuleLocation {
    // TODO: "path" is probably overloaded. Goal here is to get a local path, so
    // "local" or "dir" or something? But it conflicts with adding modules which
    // output to a specific path.
    Local { path: PathBuf },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_most_minimal() {
        let config: Config = toml::from_str("").unwrap();

        if let Config::App(app_config) = config {
            assert_eq!(app_config.modules, vec![]);
        } else {
            panic!("Expected an app config from a minimal config")
        }
    }

    #[test]
    fn roundtrip_app() {
        // Needed since Deserialize is implemented and Serialize is derived.
        let config = Config::App(AppConfig {
            modules: vec![ModuleInvocation {
                args: Table::new(),
                prefix: Some(PathBuf::from("/foo")),
                location: ModuleLocation::Local {
                    path: PathBuf::from("."),
                },
            }],
        });

        let serialized = toml::to_string(&config).unwrap();
        let deserialized = toml::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn roundtrip_module() {
        // Needed since Deserialize is implemented and Serialize is derived.
        let config = Config::Module(ModuleConfig {
            args: BTreeMap::new(),
            modules: vec![ModuleInvocation {
                args: Table::new(),
                prefix: Some(PathBuf::from("/foo")),
                location: ModuleLocation::Local {
                    path: PathBuf::from("."),
                },
            }],
        });

        let serialized = toml::to_string(&config).unwrap();
        let deserialized = toml::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn parse_explicit_app() {}
}
