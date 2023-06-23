use serde::ser::SerializeMap;
use serde::Serialize;

use crate::Value;

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Object(val) => {
                let mut map = serializer.serialize_map(Some(val.len()))?;
                for (k, v) in val {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            &Value::I64(v) => serializer.serialize_i64(v),
            &Value::I32(v) => serializer.serialize_i32(v),
            &Value::I16(v) => serializer.serialize_i16(v),
            &Value::I8(v) => serializer.serialize_i8(v),
            &Value::U64(v) => serializer.serialize_u64(v),
            &Value::U32(v) => serializer.serialize_u32(v),
            &Value::U16(v) => serializer.serialize_u16(v),
            &Value::U8(v) => serializer.serialize_u8(v),
            &Value::F64(v) => serializer.serialize_f64(v),
            Value::String(v) => serializer.serialize_str(v),
            Value::Bytes(v) => serializer.serialize_bytes(v),
            &Value::Bool(v) => serializer.serialize_bool(v),
            Value::Seq(v) => v.serialize(serializer),
        }
    }
}
