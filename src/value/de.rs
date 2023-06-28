use std::collections::HashMap;
use std::fmt;

use serde::de;
use serde::de::{DeserializeSeed, SeqAccess};
use serde::{de::Visitor, Deserialize};

use super::Value;

macro_rules! visit {
    ($fn:ident, $typ:ty, $field:ident) => {
        fn $fn<E>(self, v: $typ) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Value::$field(v))
        }
    };
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid epee value")
            }

            visit!(visit_i64, i64, I64);
            visit!(visit_i32, i32, I32);
            visit!(visit_i16, i16, I16);
            visit!(visit_i8, i8, I8);
            visit!(visit_u64, u64, U64);
            visit!(visit_u32, u32, U32);
            visit!(visit_u16, u16, U16);
            visit!(visit_u8, u8, U8);
            visit!(visit_f64, f64, F64);
            visit!(visit_byte_buf, Vec<u8>, Bytes);
            visit!(visit_bool, bool, Bool);

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut output = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(val) = seq.next_element()? {
                    output.push(val);
                }

                Ok(Value::Seq(output))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut output: HashMap<String, Value> = HashMap::new();
                while let Some(key) = map.next_key()? {
                    let val = map.next_value()?;
                    output.insert(key, val);
                }

                Ok(Value::Object(output))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

struct KeyDeserializer;

impl<'de> DeserializeSeed<'de> for KeyDeserializer {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyDeserializer {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(s.to_string())
    }

    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(s)
    }
}
