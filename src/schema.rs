use crate::google::protobuf::Value;

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        convert_to_value(value)
    }
}
fn convert_to_value(val: serde_json::Value) -> Value {
    use std::collections::HashMap;

    use crate::google::protobuf::{ListValue, Struct, value::Kind};
    let kind = Some(match val {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(bool) => Kind::BoolValue(bool),
        serde_json::Value::Number(number) => {
            return Value {
                kind: number.as_f64().map(|f| Kind::NumberValue(f)),
            };
        }
        serde_json::Value::String(string) => Kind::StringValue(string),
        serde_json::Value::Array(values) => {
            let mut list = Vec::with_capacity(values.len());
            for v in values {
                list.push(convert_to_value(v));
            }
            Kind::ListValue(ListValue { values: list })
        }
        serde_json::Value::Object(map) => {
            let mut fields = HashMap::with_capacity(map.len());
            for (k, v) in map {
                fields.insert(k, convert_to_value(v));
            }
            Kind::StructValue(Struct { fields })
        }
    });
    Value { kind }
}
