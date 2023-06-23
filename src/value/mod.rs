mod de;
mod ser;

use std::collections::HashMap;

use serde::de::Unexpected;

macro_rules! get_type {
    ($fn:ident, $ty:ty, $variant:ident) => {
        /// Gets this type from the [`Value`].
        pub fn $fn(value: Value) -> Option<$ty> {
            match value {
                Value::$variant(val) => Some(val),
                _ => None,
            }
        }
    };
}

macro_rules! is_type {
    ($fn:ident, $variant:ident) => {
        /// Checks if the current [`Value`] is this type.
        pub fn $fn(&self) -> bool {
            match self {
                Value::$variant(_) => true,
                _ => false,
            }
        }
    };
}

/// An enum implementing [`Deserialize`](serde::Deserialize) and [`Serialize`](serde::Serialize) with
/// every possible epee field.
#[derive(Debug)]
pub enum Value {
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Bool(bool),
    Object(HashMap<String, Value>),
    Seq(Vec<Value>),
}

impl Value {
    /// If the current value is an Object get a reference to the value at `key`
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    /// If the current value is an Object get a mutable reference to the value at `key`
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Object(obj) => obj.get_mut(key),
            _ => None,
        }
    }

    /// If the current value is an Object get and remove the value at `key`
    pub fn get_and_remove(&mut self, key: &str) -> Option<Value> {
        match self {
            Value::Object(obj) => obj.remove(key),
            _ => None, //visit!(visit_map, HashMap<String, Value>, Object);
        }
    }

    /// Gets the current `Value` types [`Unexpected`] representation.
    pub fn get_value_type_as_unexpected(&self) -> Unexpected {
        match self {
            Value::I64(v) => Unexpected::Signed(*v),
            Value::I32(v) => Unexpected::Signed(*v as i64),
            Value::I16(v) => Unexpected::Signed(*v as i64),
            Value::I8(v) => Unexpected::Signed(*v as i64),
            Value::U64(v) => Unexpected::Unsigned(*v),
            Value::U32(v) => Unexpected::Unsigned(*v as u64),
            Value::U16(v) => Unexpected::Unsigned(*v as u64),
            Value::U8(v) => Unexpected::Unsigned(*v as u64),
            Value::F64(v) => Unexpected::Float(*v),
            Value::String(v) => Unexpected::Str(v),
            Value::Bytes(v) => Unexpected::Bytes(v),
            Value::Bool(v) => Unexpected::Bool(*v),
            Value::Object(_) => Unexpected::Map,
            Value::Seq(_) => Unexpected::Seq,
        }
    }

    get_type!(get_i64, i64, I64);
    get_type!(get_i32, i32, I32);
    get_type!(get_i16, i16, I16);
    get_type!(get_i8, i8, I8);
    get_type!(get_u64, u64, U64);
    get_type!(get_u32, u32, U32);
    get_type!(get_u16, u16, U16);
    get_type!(get_u8, u8, U8);
    get_type!(get_f64, f64, F64);
    get_type!(get_string, String, String);
    get_type!(get_bytes, Vec<u8>, Bytes);
    get_type!(get_bool, bool, Bool);
    get_type!(get_raw_hashmap, HashMap<String, Value>, Object);
    get_type!(get_seq, Vec<Value>, Seq);

    is_type!(is_i64, I64);
    is_type!(is_i32, I32);
    is_type!(is_i16, I16);
    is_type!(is_i8, I8);
    is_type!(is_u64, U64);
    is_type!(is_u32, U32);
    is_type!(is_u16, U16);
    is_type!(is_u8, U8);
    is_type!(is_f64, F64);
    is_type!(is_string, String);
    is_type!(is_bytes, Bytes);
    is_type!(is_bool, Bool);
    is_type!(is_object, Object);
    is_type!(is_seq, Seq);
}
