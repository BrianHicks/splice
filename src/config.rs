use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};
use toml::Table;

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Config {
    App(AppConfig),
    Module(ModuleConfig),
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
    modules: Vec<ModuleInvocation>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AppConfig {
    #[serde(default, rename = "module")]
    modules: Vec<ModuleInvocation>,
}

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ModuleInvocation {
    #[serde(flatten)]
    location: ModuleLocation,
    #[serde(default, skip_serializing_if = "Table::is_empty")]
    arguments: toml::Table,
}

#[derive(Debug, Deserialize, Serialize)]
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
                arguments: Table::new(),
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
            modules: vec![ModuleInvocation {
                arguments: Table::new(),
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
