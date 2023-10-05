use crate::{
    Error, Marker, Result, MARKER_SINGLE_BOOL, MARKER_SINGLE_F64, MARKER_SINGLE_I16,
    MARKER_SINGLE_I32, MARKER_SINGLE_I64, MARKER_SINGLE_I8, MARKER_SINGLE_STRING,
    MARKER_SINGLE_STRUCT, MARKER_SINGLE_U16, MARKER_SINGLE_U32, MARKER_SINGLE_U64,
    MARKER_SINGLE_U8,
};
use serde::Serialize;
use std::io;

pub struct Serializer<'b> {
    buffer: &'b mut dyn io::Write,
    state: State,
    is_root: bool,
}

#[derive(Clone, Eq, PartialEq)]
pub enum State {
    Empty,
    First { length: usize },
    Rest,
}

impl<'b> Serializer<'b> {
    pub fn new_root(buffer: &'b mut dyn io::Write) -> Self {
        Self {
            buffer,
            state: State::Empty,
            is_root: true,
        }
    }

    fn write_marker(&mut self, marker: Marker) -> Result<()> {
        if self.is_root && marker == MARKER_SINGLE_STRUCT {
            self.is_root = false;

            return Ok(());
        }

        if self.is_root && marker != MARKER_SINGLE_STRUCT {
            return Err(Error::root_must_be_struct(marker));
        }

        match self.state {
            State::Empty => self.buffer.write_all(&[marker.to_byte()])?,
            // special case sequences of bytes as strings
            State::First { length } if marker == MARKER_SINGLE_U8 => {
                self.buffer.write_all(&[MARKER_SINGLE_STRING.to_byte()])?;
                self.buffer.write_all(&crate::varint::encode(length))?;

                self.state = State::Rest;
            }
            State::First { length } => {
                self.buffer.write_all(&[marker.to_sequence().to_byte()])?;
                self.buffer.write_all(&crate::varint::encode(length))?;

                self.state = State::Rest;
            }
            State::Rest => {}
        };

        Ok(())
    }
}

impl<'a, 'b> serde::Serializer for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = StructSerializer<'a, 'b>;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_BOOL)?;
        self.buffer.write_all(&[v as u8])?;

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_I8)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_I16)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_I32)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_I64)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_U8)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_U16)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_U32)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_U64)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok> {
        Err(Error::f32_is_not_supported())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_F64)?;
        self.buffer.write_all(&v.to_le_bytes())?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_U8)?;
        self.buffer.write_all(&[v as u8])?;

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_STRING)?;
        self.buffer.write_all(&crate::varint::encode(v.len()))?;
        self.buffer.write_all(v.as_bytes())?;

        Ok(())
    }

    // epee expects "bytes" to be marked as a string ...
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.write_marker(MARKER_SINGLE_STRING)?;
        self.buffer.write_all(&crate::varint::encode(v.len()))?;
        self.buffer.write_all(v)?;

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::options_are_not_supported())
    }

    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        Err(Error::options_are_not_supported())
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::unit_is_not_supported())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<Self::Ok> {
        Err(Error::enums_are_not_supported())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        Err(Error::enums_are_not_supported())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or_else(Error::no_length)?;
        self.state = State::First { length: len };

        if len == 0 {
            self.write_marker(Marker::Sequence { element: 255 })?;
        }

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::tuple_structs_are_not_supported())
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::enums_are_not_supported())
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::enums_are_not_supported())
    }

    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        StructSerializer::new(self, len)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::enums_are_not_supported())
    }
}

impl<'a, 'b> serde::ser::SerializeSeq for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.state = State::Empty;

        Ok(())
    }
}

impl<'a, 'b> serde::ser::SerializeTuple for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<'a, 'b> serde::ser::SerializeTupleStruct for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> serde::ser::SerializeTupleVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> serde::ser::SerializeMap for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> serde::ser::SerializeStructVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

pub struct StructSerializer<'a, 'b> {
    inner: &'a mut Serializer<'b>,
    outer_state: State,
}

impl<'a, 'b> StructSerializer<'a, 'b> {
    fn new(inner: &'a mut Serializer<'b>, number_of_fields: usize) -> Result<Self> {
        inner.write_marker(MARKER_SINGLE_STRUCT)?;
        inner
            .buffer
            .write_all(&crate::varint::encode(number_of_fields))?;

        let current_state = inner.state.clone();
        inner.state = State::Empty;

        Ok(Self {
            inner,
            outer_state: current_state,
        })
    }
}

impl<'a, 'b> serde::ser::SerializeStruct for StructSerializer<'a, 'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let name_length = key.len() as u8;
        self.inner.buffer.write_all(&[name_length])?;
        self.inner.buffer.write_all(key.as_bytes())?;

        value.serialize(&mut *self.inner)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.state = self.outer_state;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serializer as _;

    #[test]
    fn given_serializer_in_non_sequence_state_serializes_marker_first() {
        let mut buffer = Vec::new();
        let mut serializer = Serializer {
            buffer: &mut buffer,
            state: State::Empty,
            is_root: false,
        };

        (&mut serializer).serialize_bool(true).unwrap();

        let expected_buffer_content = vec![11, 0x01];

        assert_eq!(buffer, expected_buffer_content)
    }

    #[test]
    fn given_serializer_in_sequence_state_serializes_first_array_marker_then_length_and_then_elements_without_marker(
    ) {
        let mut buffer = Vec::new();
        let mut serializer = Serializer {
            buffer: &mut buffer,
            state: State::Empty,
            is_root: false,
        };

        let ser = &mut serializer;
        let seq = ser.serialize_seq(Some(3)).unwrap();
        seq.serialize_bool(true).unwrap();
        seq.serialize_bool(true).unwrap();
        seq.serialize_bool(true).unwrap();
        serde::ser::SerializeSeq::end(seq).unwrap();

        let expected_buffer_content = vec![11 | 0x80, 0x0c, 0x01, 0x01, 0x01];

        assert_eq!(buffer, expected_buffer_content)
    }
}
