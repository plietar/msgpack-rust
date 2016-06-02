use serde;
use rmp::Value;
use rmp::Marker;
use std::fmt;
use std::result;
use std::vec;

#[derive(Debug)]
pub enum Error {
    TypeMismatch(Marker),
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
    Syntax(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while decoding value" }
    fn cause(&self) -> Option<&::std::error::Error> { None }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl serde::de::Error for Error {
    fn invalid_value(msg: &str) -> Error {
        Error::Syntax(format!("syntax error: {}", msg))
    }

    fn invalid_length(len: usize) -> Error {
        Error::LengthMismatch(len as u32)
    }

    fn invalid_type(ty: serde::de::Type) -> Error {
        match ty {
            serde::de::Type::Bool => Error::TypeMismatch(Marker::True),
            serde::de::Type::Usize => Error::TypeMismatch(Marker::FixPos(0)),
            serde::de::Type::U8 => Error::TypeMismatch(Marker::U8),
            serde::de::Type::U16 => Error::TypeMismatch(Marker::U16),
            serde::de::Type::U32 => Error::TypeMismatch(Marker::U32),
            serde::de::Type::U64 => Error::TypeMismatch(Marker::U64),
            serde::de::Type::Isize => Error::TypeMismatch(Marker::FixNeg(0)),
            serde::de::Type::I8 => Error::TypeMismatch(Marker::I8),
            serde::de::Type::I16 => Error::TypeMismatch(Marker::I16),
            serde::de::Type::I32 => Error::TypeMismatch(Marker::I32),
            serde::de::Type::I64 => Error::TypeMismatch(Marker::I64),
            serde::de::Type::F32 => Error::TypeMismatch(Marker::F32),
            serde::de::Type::F64 => Error::TypeMismatch(Marker::F64),
            serde::de::Type::Char => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::Str => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::String => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::Unit => Error::TypeMismatch(Marker::Null),
            serde::de::Type::Option => Error::TypeMismatch(Marker::Null),
            serde::de::Type::Seq => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Map => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::UnitStruct => Error::TypeMismatch(Marker::Null),
            serde::de::Type::NewtypeStruct => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::TupleStruct => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Struct => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::Tuple => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Enum => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::StructVariant => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::TupleVariant => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::UnitVariant => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Bytes => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::FieldName => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::VariantName => Error::TypeMismatch(Marker::Str32),
        }
    }

    fn end_of_stream() -> Error {
        Error::Uncategorized("end of stream".to_string())
    }

    fn missing_field(_field: &str) -> Error {
        Error::Uncategorized("missing field".to_string())
    }

    fn unknown_field(_field: &str) -> Error {
        Error::Uncategorized("unknown field".to_string())
    }

     fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Uncategorized(msg.into())
    }
}

pub type Result<T> = result::Result<T, Error>;

pub struct Deserializer {
    value: Option<Value>,
}

impl Deserializer {
    pub fn new(value: Value) -> Deserializer {
        Deserializer {
            value: Some(value),
        }
    }
}

impl serde::Deserializer for Deserializer {
    type Error = Error;

    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        use rmp::Value::*;
        use rmp::value::Integer::*;
        use rmp::value::Float::*;

        let value = match self.value.take() {
            Some(value) => value,
            None => return Err(serde::de::Error::end_of_stream()),
        };

        match value {
            Nil => visitor.visit_none(),
            String(v) => visitor.visit_string(v),
            Boolean(v) => visitor.visit_bool(v),
            Integer(I64(v)) => visitor.visit_i64(v),
            Integer(U64(v)) => visitor.visit_u64(v),
            Float(F32(v)) => visitor.visit_f32(v),
            Float(F64(v)) => visitor.visit_f64(v),
            Binary(v) => visitor.visit_byte_buf(v),
            Array(v) => visitor.visit_seq(SeqVisitor {
                de: self,
                len: v.len(),
                actual: v.len(),
                iter: v.into_iter(),
            }),
            Map(v) => visitor.visit_map(MapVisitor {
                de: self,
                len: v.len(),
                actual: v.len(),
                iter: v.into_iter(),
                value: None,
            }),
            Ext(_, _) => unimplemented!(),
        }
    }

    #[inline]
    fn deserialize_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        match self.value {
            Some(Value::Nil) => visitor.visit_none(),
            Some(_) => visitor.visit_some(self),
            None => Err(serde::de::Error::end_of_stream()),
        }
    }
}

struct SeqVisitor<'a> {
    de: &'a mut Deserializer,
    iter: vec::IntoIter<Value>,
    len: usize,
    actual: usize,
}

impl <'a> serde::de::SeqVisitor for SeqVisitor<'a> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: serde::Deserialize,
    {
        match self.iter.next() {
            Some(value) => {
                self.len -= 1;
                self.de.value = Some(value);
                Ok(Some(try!(serde::Deserialize::deserialize(self.de))))
            }
            None => Ok(None),
        }
    }

    fn end(&mut self) -> Result<()> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.actual as u32))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

struct MapVisitor<'a> {
    de: &'a mut Deserializer,
    iter: vec::IntoIter<(Value, Value)>,
    value: Option<Value>,
    len: usize,
    actual: usize,
}

impl <'a> serde::de::MapVisitor for MapVisitor<'a> {
    type Error = Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>>
        where T: serde::Deserialize
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.len -= 1;
                self.value = Some(value);
                self.de.value = Some(key);
                Ok(Some(try!(serde::Deserialize::deserialize(self.de))))
            }
            None => Ok(None),
        }
    }

    fn visit_value<T>(&mut self) -> Result<T>
        where T: serde::Deserialize
    {
        let value = self.value.take().unwrap();
        self.de.value = Some(value);
        Ok(try!(serde::Deserialize::deserialize(self.de)))
    }

    fn end(&mut self) -> Result<()> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.actual as u32))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub fn from_value<T>(value: Value) -> Result<T>
    where T: serde::Deserialize {
    serde::Deserialize::deserialize(&mut Deserializer::new(value))
}
