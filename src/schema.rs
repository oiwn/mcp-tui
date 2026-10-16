//! Editable argument suggestions, not JSON Schema validation.

use serde_json::{Map, Value};

const MAX_SCHEMA_DEPTH: usize = 16;

/// Suggest an object-root argument value from a tool's input schema.
///
/// Explicit values win in this order: `default`, first `examples` value,
/// first `enum` value. They are copied as supplied, without validation.
/// Otherwise all declared object properties receive simple placeholders.
/// Arrays start empty; references and compositions are not resolved.
/// Schema traversal stops after 16 levels. Any non-object root becomes `{}`.
pub fn object_skeleton(schema: &Value) -> Value {
    match placeholder(schema, 0) {
        object @ Value::Object(_) => object,
        _ => Value::Object(Map::new()),
    }
}

fn placeholder(schema: &Value, depth: usize) -> Value {
    if depth >= MAX_SCHEMA_DEPTH {
        return Value::Null;
    }
    let Some(schema) = schema.as_object() else {
        return Value::Null;
    };
    if let Some(value) = schema.get("default") {
        return value.clone();
    }
    for key in ["examples", "enum"] {
        if let Some(value) = schema
            .get(key)
            .and_then(Value::as_array)
            .and_then(|values| values.first())
        {
            return value.clone();
        }
    }
    if [
        "$ref",
        "$dynamicRef",
        "allOf",
        "anyOf",
        "oneOf",
        "not",
        "if",
        "then",
        "else",
    ]
    .iter()
    .any(|key| schema.contains_key(*key))
    {
        return Value::Null;
    }
    let kind = match schema.get("type") {
        Some(Value::String(kind)) => kind.as_str(),
        None if schema.get("properties").is_some_and(Value::is_object) => {
            "object"
        }
        _ => return Value::Null,
    };
    match kind {
        "object" => {
            let properties = schema.get("properties").and_then(Value::as_object);
            Value::Object(
                properties
                    .into_iter()
                    .flatten()
                    .map(|(name, property)| {
                        (name.clone(), placeholder(property, depth + 1))
                    })
                    .collect(),
            )
        }
        "array" => Value::Array(Vec::new()),
        "string" => Value::String(String::new()),
        "integer" | "number" => Value::from(0),
        "boolean" => Value::Bool(false),
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn property(schema: Value) -> Value {
        object_skeleton(&json!({"type":"object", "properties":{"value":schema}}))
    }

    #[test]
    fn explicit_values_follow_precedence_including_falsey_values() {
        for value in [
            Value::Null,
            json!(false),
            json!(0),
            json!(""),
            json!([]),
            json!({"nested":true}),
        ] {
            assert_eq!(
                property(
                    json!({"default":value, "examples":["example"], "enum":["enum"]})
                ),
                json!({"value":value})
            );
        }
        assert_eq!(
            property(json!({"examples":[false, true], "enum":["enum"]})),
            json!({"value":false})
        );
        assert_eq!(
            property(json!({"examples":[], "enum":[0, 1]})),
            json!({"value":0})
        );
        assert_eq!(
            property(json!({"examples":"malformed", "enum":["first", "second"]})),
            json!({"value":"first"})
        );
        assert_eq!(
            property(json!({"type":"boolean", "examples":{}, "enum":[]})),
            json!({"value":false})
        );
    }

    #[test]
    fn explicit_root_objects_are_preserved() {
        let value = json!({"nested":{"items":[1, "two", null]}});
        for schema in [
            json!({"default":value}),
            json!({"examples":[value]}),
            json!({"enum":[value]}),
        ] {
            assert_eq!(object_skeleton(&schema), value);
        }
    }

    #[test]
    fn primitives_get_simple_placeholders_without_validation() {
        for (schema, expected) in [
            (json!({"type":"string", "minLength":5}), json!("")),
            (json!({"type":"integer", "minimum":10}), json!(0)),
            (json!({"type":"number"}), json!(0)),
            (json!({"type":"boolean"}), json!(false)),
            (json!({"type":"null"}), Value::Null),
        ] {
            assert_eq!(property(schema), json!({"value":expected}));
        }
    }

    #[test]
    fn objects_include_required_and_optional_nested_properties() {
        let schema = json!({"type":"object", "required":["name"], "properties":{
            "name":{"type":"string"},
            "options":{"properties":{
                "enabled":{"type":"boolean"},
                "count":{"default":3},
                "empty":{"type":"object"}
            }}
        }});
        let before = schema.clone();
        assert_eq!(
            object_skeleton(&schema),
            json!({"name":"", "options":{"enabled":false,"count":3,"empty":{}}})
        );
        assert_eq!(schema, before);
    }

    #[test]
    fn arrays_are_empty_unless_an_explicit_value_exists() {
        for items in [
            json!({"type":"string"}),
            json!({"type":"object"}),
            json!({"$ref":"https://example.invalid/schema"}),
            json!(false),
        ] {
            assert_eq!(
                property(json!({"type":"array", "items":items, "minItems":1})),
                json!({"value":[]})
            );
        }
        assert_eq!(
            property(json!({"type":"array", "examples":[[1, 2]]})),
            json!({"value":[1, 2]})
        );
    }

    #[test]
    fn missing_and_malformed_schema_shapes_do_not_panic() {
        for schema in [
            Value::Null,
            json!(false),
            json!(true),
            json!(4),
            json!("object"),
            json!([]),
            json!({}),
            json!({"type":false}),
            json!({"type":"unknown"}),
            json!({"properties":[]}),
        ] {
            assert_eq!(object_skeleton(&schema), json!({}));
            assert_eq!(property(schema), json!({"value":null}));
        }
        for properties in [Value::Null, json!(true), json!([]), json!("bad")] {
            assert_eq!(
                object_skeleton(
                    &json!({"type":"object", "properties":properties})
                ),
                json!({})
            );
        }
        assert_eq!(
            object_skeleton(&json!({"properties":{"broken":false}})),
            json!({"broken":null})
        );
    }

    #[test]
    fn references_compositions_and_type_unions_fall_back() {
        for schema in [
            json!({"$ref":"#/$defs/self", "$defs":{"self":{"$ref":"#/$defs/self"}}}),
            json!({"$ref":"https://example.invalid/schema", "type":"object"}),
            json!({"$dynamicRef":"#node"}),
            json!({"allOf":[{"type":"object"}]}),
            json!({"anyOf":[{"type":"string"}]}),
            json!({"oneOf":[{"type":"integer"}]}),
            json!({"type":"object", "not":{}}),
            json!({"type":"object", "if":{}, "then":{}, "else":{}}),
            json!({"type":["string", "null"]}),
        ] {
            assert_eq!(object_skeleton(&schema), json!({}));
            assert_eq!(property(schema), json!({"value":null}));
        }
        assert_eq!(
            property(json!({"$ref":"#/unknown", "default":"suggestion"})),
            json!({"value":"suggestion"})
        );
    }

    #[test]
    fn non_object_root_suggestions_always_fall_back_to_object() {
        for value in [
            Value::Null,
            json!(false),
            json!(1),
            json!("text"),
            json!([{}]),
        ] {
            assert_eq!(
                object_skeleton(
                    &json!({"type":"object", "properties":{"x":{"type":"string"}}, "default":value})
                ),
                json!({})
            );
        }
        for kind in ["string", "number", "integer", "boolean", "array", "null"] {
            assert_eq!(object_skeleton(&json!({"type":kind})), json!({}));
        }
    }

    #[test]
    fn deep_schema_traversal_stops_at_the_bound_and_returns_serializable_json()
    -> Result<(), serde_json::Error> {
        let schema = (0..MAX_SCHEMA_DEPTH * 2).fold(
            json!({"type":"string"}),
            |inner, _| json!({"type":"object", "properties":{"child":inner}}),
        );
        let result = object_skeleton(&schema);
        let mut cursor = &result;
        for _ in 0..MAX_SCHEMA_DEPTH {
            assert!(cursor.is_object());
            cursor = &cursor["child"];
        }
        assert_eq!(cursor, &Value::Null);
        assert_eq!(
            serde_json::from_str::<Value>(&serde_json::to_string(&result)?)?,
            result
        );
        Ok(())
    }
}
