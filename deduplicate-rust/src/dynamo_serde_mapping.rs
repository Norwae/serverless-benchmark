use serde::Serialize;
use std::collections::HashMap;
use rusoto_dynamodb::AttributeValue;
use serde_json::{Value, to_value, from_value};
use serde::de::DeserializeOwned;
use serde_json::Result;

pub fn serialize_to_dynamo<A: Serialize>(obj: A) -> Result<HashMap<String, AttributeValue>> {
    Ok(to_value(obj).map(json_object_to_map)?)
}

pub fn deserialize_from_dynamo<A: DeserializeOwned>(map: HashMap<String, AttributeValue>) -> std::result::Result<A, serde_json::Error> {
    from_value(map_to_json(map))
}

fn map_to_json(root: HashMap<String, AttributeValue>) -> Value {
    let wrapper = AttributeValue {
        m: Some(root),
        ..AttributeValue::default()
    };
    attribute_value_to_json(wrapper)
}

fn attribute_value_to_json(attr: AttributeValue) -> Value {
    if let Some(s) = attr.s {
        s.into()
    } else if let Some(string) = attr.n {
        if string.contains(".") {
            string.parse::<f64>().unwrap().into()
        } else {
            string.parse::<i64>().unwrap().into()
        }
    } else if let Some(m) = attr.m {
        let mut fields = serde_json::map::Map::new();
        for (k, v) in m {
            fields.insert(k, attribute_value_to_json(v));
        }
        Value::Object(fields.into())
    } else if let Some(l) = attr.l {
        let mut vector = Vec::new();

        for entry in l {
            vector.push(attribute_value_to_json(entry));
        }

        Value::Array(vector)
    } else if let Some(_) = attr.null {
        Value::Null
    } else if let Some(b) = attr.bool {
        Value::Bool(b)
    } else {
        unreachable!()
    }
}

fn json_to_attribute_value(json: Value) -> AttributeValue {
    let mut av = AttributeValue::default();
    match json {
        Value::Null => av.null = Some(true),
        Value::Bool(v) => av.bool = Some(v),
        Value::Number(v) => av.n = Some(v.to_string()),
        Value::String(v) => av.s = Some(v),
        Value::Array(v) => {
            let mut vec = Vec::new();
            for entry in v {
                vec.push(json_to_attribute_value(entry))
            }
            av.l = Some(vec)
        }
        Value::Object(v) => {
            let mut map = HashMap::new();
            for (key, value) in v {
                map.insert(key, json_to_attribute_value(value));
            }

            av.m = Some(map)
        }
    }

    return av;
}

fn json_object_to_map(json: Value) -> HashMap<String, AttributeValue> {
    json_to_attribute_value(json).m.expect("Root must be an object")
}