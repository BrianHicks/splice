use std::collections::BTreeMap;
use std::fmt::Display;

use eyre::{Context, OptionExt, Result, bail};
use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Validator {
    #[serde(default, rename = "type")]
    type_: InferredValueType,
    #[serde(default)]
    default: Option<Value>,

    // complex objects
    #[serde(default)]
    items: Option<Box<Validator>>,
    #[serde(default)]
    values: Option<Box<Validator>>,
    #[serde(default)]
    properties: BTreeMap<String, Validator>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged, rename_all = "lowercase")]
pub enum InferredValueType {
    // we can often infer the type of the argument by the presence of other
    // fields. We do that wherever possible in order to save some typing.
    #[default]
    Inferred,
    Explicit(ValueType),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    // the usual scalar values
    #[serde(alias = "str")]
    String,
    #[serde(alias = "int")]
    Integer,
    Float,
    #[serde(alias = "bool")]
    Boolean,
    Datetime,

    // composite value. Array, as usual. But instead of Table, we have a
    // key-value Map (controlled by `values`) and a structured table
    // (struct+properties)
    Array,
    Map,
    Struct,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => f.write_str("string"),
            Self::Integer => f.write_str("integer"),
            Self::Float => f.write_str("float"),
            Self::Boolean => f.write_str("boolean"),
            Self::Datetime => f.write_str("datetime"),
            Self::Array => f.write_str("array"),
            Self::Map => f.write_str("map"),
            Self::Struct => f.write_str("struct"),
        }
    }
}

impl Validator {
    pub fn validate(&self, value: Option<Value>) -> Result<Value> {
        // TODO: self-check that the default matches the schema, if present.
        // (Maybe in a self-check method?)
        match value {
            None => match &self.default {
                Some(default) => Ok(default.clone()),
                None => bail!("value is required"),
            },
            Some(value) => {
                let target = self.infer_type()?;

                match (target, value) {
                    (ValueType::Integer, v @ Value::Integer(_)) => Ok(v),
                    (ValueType::Float, v @ Value::Float(_)) => Ok(v),
                    (ValueType::String, v @ Value::String(_)) => Ok(v),
                    (ValueType::Boolean, v @ Value::Boolean(_)) => Ok(v),
                    (ValueType::Datetime, v @ Value::Datetime(_)) => Ok(v),
                    (ValueType::Array, Value::Array(mut items)) => {
                        let validator = self
                            .items
                            .as_ref()
                            .ok_or_eyre("missing items schema for an array")?;

                        let mut out = Vec::with_capacity(items.len());
                        for (i, item) in items.drain(..).enumerate() {
                            out.push(
                                validator
                                    .validate(Some(item))
                                    .wrap_err_with(|| format!("at index {i}"))?,
                            )
                        }

                        Ok(Value::Array(out))
                    }
                    (ValueType::Map, Value::Table(mut table)) => {
                        let validator = self
                            .values
                            .as_ref()
                            .ok_or_eyre("missing values schema for a map")?;

                        let mut out = BTreeMap::new();
                        // TODO: is cloning all these keys OK? I dunno. There's no drain-equivalent on these.
                        for key in table.keys().cloned().collect::<Vec<String>>() {
                            let entry = table.remove(&key).unwrap();
                            let value = validator
                                .validate(Some(entry))
                                .wrap_err_with(|| format!("at key `{key}`"))?;
                            out.insert(key, value);
                        }

                        Ok(out.into())
                    }
                    (ValueType::Struct, Value::Table(mut table)) => {
                        if self.properties.is_empty() {
                            bail!("missing properties schema for a struct")
                        }

                        let mut out = BTreeMap::new();

                        for (field, validator) in &self.properties {
                            let validated = validator
                                .validate(table.remove(field))
                                .wrap_err_with(|| format!("at field `{field}`"))?;
                            out.insert(field, validated);
                        }

                        let remaining_keys: Vec<String> =
                            table.into_iter().map(|(k, _)| format!("`{k}`")).collect();
                        if !remaining_keys.is_empty() {
                            bail!("unknown keys: {}", remaining_keys.join(", "))
                        }

                        Ok(out.into())
                    }
                    (_, v) => bail!(
                        "expected type {target}, but got a value of type {}",
                        v.type_str()
                    ),
                }
            }
        }
    }

    fn infer_type(&self) -> Result<ValueType> {
        match self.type_ {
            InferredValueType::Explicit(t) => Ok(t),
            InferredValueType::Inferred => {
                if self.items.is_some() {
                    Ok(ValueType::Array)
                } else if self.values.is_some() {
                    Ok(ValueType::Map)
                } else if !self.properties.is_empty() {
                    Ok(ValueType::Struct)
                } else {
                    match &self.default {
                        None => bail!("Cannot infer a type"),
                        Some(Value::Integer(_)) => Ok(ValueType::Integer),
                        Some(Value::Float(_)) => Ok(ValueType::Float),
                        Some(Value::String(_)) => Ok(ValueType::String),
                        Some(Value::Boolean(_)) => Ok(ValueType::Boolean),
                        Some(Value::Datetime(_)) => Ok(ValueType::Datetime),
                        Some(Value::Array(_)) => bail!(
                            "The arg default was an array, but we're missing an `items` schema."
                        ),
                        Some(Value::Table(_)) => bail!(
                            "The arg default was a table, but we're missing a `values` or `properties` schema."
                        ),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(src: &str) -> Validator {
        toml::from_str(src).unwrap()
    }

    macro_rules! assert_validates {
        ($src:expr, $in:expr, $out:expr) => {
            assert_eq!(
                Value::try_from($out).unwrap(),
                parse($src)
                    .validate(Some(Value::try_from($in).unwrap()))
                    .unwrap()
            )
        };
    }

    macro_rules! assert_fails_validation {
        ($src:expr, $in:expr, $msg:expr) => {
            assert_eq!(
                $msg,
                parse($src)
                    .validate(Some(Value::try_from($in).unwrap()))
                    .unwrap_err()
                    .chain()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(": ")
            )
        };
    }

    #[test]
    fn default_is_used_if_value_is_none() {
        let v = parse("default = 1");

        assert_eq!(Value::Integer(1), v.validate(None).unwrap());
    }

    #[test]
    fn raises_error_if_no_default_and_no_value() {
        let v = parse("");

        assert_eq!(
            "value is required",
            v.validate(None).unwrap_err().to_string()
        )
    }

    #[test]
    fn validates_string_success() {
        assert_validates!("type = \"string\"", "hey", "hey")
    }

    #[test]
    fn validates_string_failure() {
        assert_fails_validation!(
            "type = \"string\"",
            1,
            "expected type string, but got a value of type integer"
        );
    }

    #[test]
    fn validates_int_success() {
        assert_validates!("type = \"int\"", 1, 1)
    }

    #[test]
    fn validates_int_failure() {
        assert_fails_validation!(
            "type = \"int\"",
            true,
            "expected type integer, but got a value of type boolean"
        );
    }

    #[test]
    fn validates_float_success() {
        assert_validates!("type = \"float\"", 1.0, 1.0)
    }

    #[test]
    fn validates_float_failure() {
        assert_fails_validation!(
            "type = \"float\"",
            1,
            "expected type float, but got a value of type integer"
        );
    }

    #[test]
    fn validates_bool_success() {
        assert_validates!("type = \"bool\"", true, true)
    }

    #[test]
    fn validate_bool_failure() {
        assert_fails_validation!(
            "type = \"bool\"",
            1,
            "expected type boolean, but got a value of type integer"
        )
    }

    #[test]
    fn validates_datetime_success() {
        let mut table = toml::from_str::<toml::Table>("dt = 2026-01-01T00:00:00Z").unwrap();
        let dt = table.remove("dt").unwrap();
        let validator = parse("type = \"datetime\"");

        assert_eq!(dt.clone(), validator.validate(Some(dt)).unwrap());
    }

    #[test]
    fn validates_datetime_failure() {
        assert_fails_validation!(
            "type = \"datetime\"",
            1,
            "expected type datetime, but got a value of type integer"
        )
    }

    #[test]
    fn validates_array_success() {
        assert_validates!("items.type = \"int\"", vec![1], vec![1])
    }

    #[test]
    fn validates_array_outer_failure() {
        assert_fails_validation!(
            "type = \"array\"",
            1,
            "expected type array, but got a value of type integer"
        )
    }

    #[test]
    fn validates_array_missing_items_failure() {
        assert_fails_validation!(
            "type = \"array\"",
            Vec::<u8>::new(),
            "missing items schema for an array"
        )
    }

    #[test]
    fn validates_array_item_failure() {
        assert_fails_validation!(
            "items.type = \"int\"",
            vec!["what"],
            "at index 0: expected type integer, but got a value of type string"
        )
    }

    #[test]
    fn validates_array_item_failure_mixed() {
        assert_fails_validation!(
            "items.type = \"int\"",
            vec![Value::from(1), Value::from("what")],
            "at index 1: expected type integer, but got a value of type string"
        )
    }

    #[test]
    fn validates_values_success() {
        let mut map = BTreeMap::new();
        map.insert("x", Value::from(1));
        map.insert("y", Value::from(2));

        assert_validates!("values.type = \"int\"", &map, &map)
    }

    #[test]
    fn validates_map_outer_failure() {
        assert_fails_validation!(
            "type = \"map\"",
            1,
            "expected type map, but got a value of type integer"
        )
    }

    #[test]
    fn validates_map_missing_values_failure() {
        assert_fails_validation!(
            "type = \"map\"",
            BTreeMap::<String, u8>::new(),
            "missing values schema for a map"
        )
    }

    #[test]
    fn validates_map_values_failure() {
        let mut map = BTreeMap::new();
        map.insert("x", Value::from("hey"));

        assert_fails_validation!(
            "values.type = \"int\"",
            map,
            "at key `x`: expected type integer, but got a value of type string"
        )
    }

    #[test]
    fn validates_map_values_mixed_failure() {
        let mut map = BTreeMap::new();
        map.insert("x", Value::from(1));
        map.insert("y", Value::from("hey"));

        assert_fails_validation!(
            "values.type = \"int\"",
            map,
            "at key `y`: expected type integer, but got a value of type string"
        )
    }

    #[test]
    fn validates_properties_success() {
        let mut map = BTreeMap::new();
        map.insert("x", Value::from(1));
        map.insert("y", Value::from("test"));

        assert_validates!(
            "properties.x.type = \"int\"\nproperties.y.type = \"string\"",
            &map,
            &map
        )
    }

    #[test]
    fn validates_properties_default_success() {
        let mut map = BTreeMap::new();
        map.insert("x", Value::from(1));

        assert_validates!(
            "properties.x.default = 1",
            BTreeMap::<String, u8>::new(),
            map
        )
    }

    #[test]
    fn validates_struct_outer_failure() {
        assert_fails_validation!(
            "type = \"struct\"",
            1,
            "expected type struct, but got a value of type integer"
        )
    }

    #[test]
    fn validates_map_missing_properties_failure() {
        assert_fails_validation!(
            "type = \"struct\"",
            BTreeMap::<String, u8>::new(),
            "missing properties schema for a struct"
        )
    }

    #[test]
    fn validates_map_missing_key_failure() {
        assert_fails_validation!(
            "properties.x.type = \"int\"",
            BTreeMap::<String, u8>::new(),
            "at field `x`: value is required"
        )
    }

    #[test]
    fn validates_map_unknown_key_failure() {
        let mut table = BTreeMap::new();
        table.insert("x", Value::from(1));
        table.insert("y", Value::from(1));

        assert_fails_validation!("properties.y.type = \"int\"", table, "unknown keys: `x`")
    }

    #[test]
    fn validates_map_property_type_failure() {
        let mut table = BTreeMap::new();
        table.insert("x", Value::from("hey"));

        assert_fails_validation!(
            "properties.x.type = \"int\"",
            table,
            "at field `x`: expected type integer, but got a value of type string"
        )
    }
}
