use core::fmt;
use std::fs;

use regex::Regex;
use rustyscript::deno_core::serde::{
    de::DeserializeOwned,
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Deserialize, Serialize, Serializer,
};

pub struct ElmTypeSerializer<'a> {
    pub output: &'a mut String,
}

pub struct ElmTypeSerializerSeq<'a> {
    output: &'a mut String,
}

pub struct ElmTypeSerializerStruct<'a> {
    output: &'a mut String,
    first_field: bool,
}

impl<'a> SerializeSeq for ElmTypeSerializerSeq<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(ElmTypeSerializer {
            output: &mut self.output,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeStruct for ElmTypeSerializerStruct<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.first_field {
            self.first_field = false;
        } else {
            *self.output += ", ";
        }
        *self.output += key;
        *self.output += " : ";
        value.serialize(ElmTypeSerializer {
            output: self.output,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        *self.output += " }";
        Ok(())
    }
}

impl<'a> SerializeTuple for ElmTypeSerializerStruct<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.first_field {
            self.first_field = false;
        } else {
            *self.output += ", ";
        }
        value.serialize(ElmTypeSerializer {
            output: self.output,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        *self.output += " )";
        Ok(())
    }
}

impl<'a> SerializeMap for ElmTypeSerializerStruct<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.first_field {
            *self.output += "Dict ";
            key.serialize(ElmTypeSerializer {
                output: self.output,
            })?;
            *self.output += " ";
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.first_field {
            self.first_field = false;
            value.serialize(ElmTypeSerializer {
                output: self.output,
            })?;
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> SerializeTupleStruct for ElmTypeSerializerStruct<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.first_field {
            self.first_field = false;
        } else {
            *self.output += ", ";
        }
        value.serialize(ElmTypeSerializer {
            output: self.output,
        })
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        *self.output += " )";
        Ok(())
    }
}

impl<'a> SerializeTupleVariant for ElmTypeSerializer<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> SerializeStructVariant for ElmTypeSerializer<'a> {
    type Ok = ();

    type Error = fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> Serializer for ElmTypeSerializer<'a> {
    type Ok = ();
    type Error = fmt::Error; // Use a simple error form for demonstration.

    // Primitive types
    type SerializeSeq = ElmTypeSerializerSeq<'a>;
    type SerializeTuple = ElmTypeSerializerStruct<'a>;
    type SerializeTupleStruct = ElmTypeSerializerStruct<'a>;
    type SerializeTupleVariant = Self;
    type SerializeMap = ElmTypeSerializerStruct<'a>;
    type SerializeStruct = ElmTypeSerializerStruct<'a>;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        *self.output += "Bool";
        Ok(())
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        *self.output += "Int";
        Ok(())
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        *self.output += "Float";
        Ok(())
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        *self.output += "Float";
        Ok(())
    }

    fn serialize_str(self, _: &str) -> Result<Self::Ok, Self::Error> {
        *self.output += "String";
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        *self.output += "Maybe Never";
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        *self.output += "Maybe ";
        value.serialize(self)?;
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        *self.output += "()";
        Ok(())
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        *self.output += "List ";
        Ok(ElmTypeSerializerSeq {
            output: self.output,
        })
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(ElmTypeSerializerStruct {
            output: self.output,
            first_field: true,
        })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(ElmTypeSerializerStruct {
            output: self.output,
            first_field: true,
        })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ElmTypeSerializerStruct {
            output: self.output,
            first_field: true,
        })
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        *self.output += "{ ";
        Ok(ElmTypeSerializerStruct {
            output: self.output,
            first_field: true,
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }
}
