use crate::{
    varint, Error, Marker, Result, MARKER_SINGLE_BOOL, MARKER_SINGLE_F64, MARKER_SINGLE_I16,
    MARKER_SINGLE_I32, MARKER_SINGLE_I64, MARKER_SINGLE_I8, MARKER_SINGLE_STRING,
    MARKER_SINGLE_STRUCT, MARKER_SINGLE_U16, MARKER_SINGLE_U32, MARKER_SINGLE_U64,
    MARKER_SINGLE_U8, MARKER_U8,
};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::de::Visitor;
use std::io;

pub struct Deserializer<'b> {
    buffer: &'b mut dyn io::BufRead,
    read_header: bool,
}

impl<'b> Deserializer<'b> {
    pub fn new(buffer: &'b mut dyn io::BufRead) -> Self {
        Self {
            buffer,
            read_header: false,
        }
    }
}

impl<'b> Deserializer<'b> {
    fn read_expected_marker(&mut self, expected_marker: Marker) -> Result<()> {
        let actual_marker = self.read_marker()?;

        if expected_marker != actual_marker {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "expected marker {} but found {}",
                    expected_marker, actual_marker
                ),
            )
            .into());
        }

        Ok(())
    }

    fn read_marker(&mut self) -> Result<Marker> {
        let marker_value = self.buffer.read_u8()?;

        Ok(Marker::from_byte(marker_value))
    }

    fn read_string(&mut self, length: usize) -> Result<String> {
        let mut string_length = vec![0u8; length];
        self.buffer.read_exact(&mut string_length)?;
        let value = String::from_utf8(string_length)?;

        Ok(value)
    }

    fn read_varint_string(&mut self) -> Result<String> {
        let string_length = self.read_varint()?;
        let string = self.read_string(string_length)?;

        Ok(string)
    }

    fn read_bool(&mut self) -> Result<bool> {
        let v = self.buffer.read_u8()?;
        let value = match v {
            0 => false,
            1 => true,
            v => return Err(Error::unexpected_bool(v)),
        };

        Ok(value)
    }

    fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        self.buffer.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_varint(&mut self) -> Result<usize> {
        let varint = varint::decode(&mut self.buffer)?;

        Ok(varint)
    }

    fn dispatch_based_on_marker<'de, V>(&mut self, marker: Marker, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match marker {
            Marker::Sequence {
                element: element_marker,
            } => visitor.visit_seq(SeqAccess::with_varint_encoded_length(self, element_marker)?),
            MARKER_SINGLE_I64 => visitor.visit_i64(self.buffer.read_i64::<LittleEndian>()?),
            MARKER_SINGLE_I32 => visitor.visit_i32(self.buffer.read_i32::<LittleEndian>()?),
            MARKER_SINGLE_I16 => visitor.visit_i16(self.buffer.read_i16::<LittleEndian>()?),
            MARKER_SINGLE_I8 => visitor.visit_i8(self.buffer.read_i8()?),
            MARKER_SINGLE_U64 => visitor.visit_u64(self.buffer.read_u64::<LittleEndian>()?),
            MARKER_SINGLE_U32 => visitor.visit_u32(self.buffer.read_u32::<LittleEndian>()?),
            MARKER_SINGLE_U16 => visitor.visit_u16(self.buffer.read_u16::<LittleEndian>()?),
            MARKER_SINGLE_U8 => visitor.visit_u8(self.buffer.read_u8()?),
            MARKER_SINGLE_F64 => visitor.visit_f64(self.buffer.read_f64::<LittleEndian>()?),
            MARKER_SINGLE_STRING => visitor.visit_string(self.read_varint_string()?),
            MARKER_SINGLE_BOOL => visitor.visit_bool(self.read_bool()?),
            MARKER_SINGLE_STRUCT => visitor.visit_map(MapAccess::with_varint_encoded_fields(self)?),
            _ => Err(Error::unknown_marker(marker)),
        }
    }
}

pub struct MapAccess<'a, 'b> {
    de: &'a mut Deserializer<'b>,
    number_of_fields: usize,
    fields_read: usize,
}

impl<'a, 'b> MapAccess<'a, 'b> {
    /// Creates a new instance of [`MapAccess`] that initializes itself by
    /// reading a varint from the reader within [`Deserializer`] for the
    /// expected number of fields.
    fn with_varint_encoded_fields(de: &'a mut Deserializer<'b>) -> Result<Self> {
        let number_of_fields = varint::decode(&mut de.buffer)?;

        Ok(MapAccess {
            de,
            number_of_fields,
            fields_read: 0,
        })
    }
}

impl<'de, 'a, 'b> serde::de::MapAccess<'de> for MapAccess<'a, 'b> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.fields_read == self.number_of_fields {
            return Ok(None);
        }

        seed.deserialize(SectionFieldNameDeserializer { de: &mut *self.de })
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.de)?;
        self.fields_read += 1;

        Ok(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.number_of_fields)
    }
}

struct SectionFieldNameDeserializer<'a, 'b> {
    de: &'a mut Deserializer<'b>,
}

impl<'de, 'a, 'b> serde::de::Deserializer<'de> for SectionFieldNameDeserializer<'a, 'b> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        let field_name_length = self.de.buffer.read_u8()? as usize;
        let field_name = self.de.read_string(field_name_length)?;

        visitor.visit_string(field_name)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub struct SeqAccess<'a, 'b> {
    de: &'a mut Deserializer<'b>,
    /// How long we expect the sequence to be.
    length: usize,
    /// What kind of item we are expecting.
    element_marker: u8,
    /// How many items we already emitted.
    emitted_items: usize,
}

impl<'a, 'b> SeqAccess<'a, 'b> {
    fn with_varint_encoded_length(
        de: &'a mut Deserializer<'b>,
        element_marker: u8,
    ) -> Result<Self> {
        let length = de.read_varint()?;

        Ok(Self::with_length(de, element_marker, length))
    }

    fn with_length(de: &'a mut Deserializer<'b>, element_marker: u8, length: usize) -> Self {
        Self {
            de,
            length,
            element_marker,
            emitted_items: 0,
        }
    }
}

impl<'de, 'a, 'b> serde::de::SeqAccess<'de> for SeqAccess<'a, 'b> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.emitted_items == self.length {
            return Ok(None);
        }

        let element = seed.deserialize(SeqElementDeserializer {
            de: self.de,
            marker: self.element_marker,
        })?;
        self.emitted_items += 1;

        Ok(Some(element))
    }
}

struct SeqElementDeserializer<'a, 'b> {
    de: &'a mut Deserializer<'b>,
    marker: u8,
}

impl<'de, 'a, 'b> serde::de::Deserializer<'de> for SeqElementDeserializer<'a, 'b> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.de
            .dispatch_based_on_marker(Marker::Single { value: self.marker }, visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a, 'b> serde::Deserializer<'de> for &'a mut Deserializer<'b> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        if !self.read_header {
            self.read_header = true;
            return visitor.visit_map(MapAccess::with_varint_encoded_fields(self)?);
        }

        let marker = self.read_marker()?;
        self.dispatch_based_on_marker(marker, visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_BOOL)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_BOOL, visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_I8)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_I8, visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_I16)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_I16, visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_I32)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_I32, visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_I64)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_I64, visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_U8)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_U8, visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_U16)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_U16, visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_U32)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_U32, visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_U64)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_U64, visitor)
    }

    fn deserialize_f32<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::f32_is_not_supported())
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_F64)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_U64, visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_U8)?;
        visitor.visit_char(self.buffer.read_u8()? as char)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_STRING)?;
        self.dispatch_based_on_marker(MARKER_SINGLE_STRING, visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, v: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_STRING)?;
        let length = self.read_varint()?;
        let buffer = self.read_bytes(length)?;

        v.visit_byte_buf(buffer)
    }

    fn deserialize_option<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::options_are_not_supported())
    }

    fn deserialize_unit<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::unit_is_not_supported())
    }

    fn deserialize_unit_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        let marker = self.read_marker()?;
        self.dispatch_based_on_marker(marker, visitor)
    }

    fn deserialize_tuple<V>(
        self,
        expected_length: usize,
        v: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        // special case tuples.
        // byte arrays and sequences are serialized as "strings" in epee-bin
        // hence, if we are told to deserialize a tuple, we check if the marker is a string, if that is the case, tell the deserializer to deserialize it as individual bytes
        match self.read_marker()? {
            MARKER_SINGLE_STRING => {
                let got_length = self.read_varint()?;

                if expected_length != got_length {
                    return Err(Error::length_mismatch(expected_length, got_length));
                }

                v.visit_seq(SeqAccess::with_length(self, MARKER_U8, got_length))
            }
            marker => Err(Error::tuples_of_type_are_not_supported(marker)),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        _: usize,
        _: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::tuple_structs_are_not_supported())
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        _: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::enums_are_not_supported())
    }

    fn deserialize_identifier<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        todo!("Unsure what should be done here?")
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        todo!("Unsure what should be done here?")
    }
}
