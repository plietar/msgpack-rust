use serde;
use rmp::Value;
use rmp::value::Integer::{U64, I64};
use rmp::value::Float::{F64, F32};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    /// Uncategorized error.
    Custom(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while encoding value" }
    fn cause(&self) -> Option<&::std::error::Error> { None }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }
}

#[derive(Debug)]
enum State {
    Value(Value),
    Array(Vec<Value>),
    Object(Vec<(Value, Value)>),
}

pub struct Serializer {
    state: Vec<State>,
}

impl Serializer {
    pub fn new() -> Serializer {
        Serializer {
            state: Vec::new(),
        }
    }

    pub fn unwrap(mut self) -> Value {
        match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        }
    }
}

impl serde::ser::Serializer for Serializer {
    type Error = Error;

    #[inline]
    fn serialize_unit(&mut self) -> Result<(), Error> {
        self.state.push(State::Value(Value::Nil));
        Ok(())
    }

    #[inline]
    fn serialize_bool(&mut self, value: bool) -> Result<(), Error> {
        self.state.push(State::Value(Value::Boolean(value)));
        Ok(())
    }

    #[inline]
    fn serialize_i64(&mut self, value: i64) -> Result<(), Error> {
        self.state.push(State::Value(Value::Integer(I64(value))));
        Ok(())
    }

    #[inline]
    fn serialize_u64(&mut self, value: u64) -> Result<(), Error> {
        self.state.push(State::Value(Value::Integer(U64(value))));
        Ok(())
    }

    #[inline]
    fn serialize_f32(&mut self, value: f32) -> Result<(), Error> {
        self.state.push(State::Value(Value::Float(F32(value))));
        Ok(())
    }

    #[inline]
    fn serialize_f64(&mut self, value: f64) -> Result<(), Error> {
        self.state.push(State::Value(Value::Float(F64(value))));
        Ok(())
    }

    #[inline]
    fn serialize_char(&mut self, value: char) -> Result<(), Error> {
        let mut s = String::new();
        s.push(value);
        self.serialize_str(&s)
    }

    #[inline]
    fn serialize_str(&mut self, value: &str) -> Result<(), Error> {
        self.state.push(State::Value(Value::String(String::from(value))));
        Ok(())
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<(), Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<V>(&mut self, value: V) -> Result<(), Error>
        where V: serde::ser::Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::SeqVisitor,
    {
        let len = visitor.len().unwrap_or(0);
        let values = Vec::with_capacity(len);

        self.state.push(State::Array(values));

        while let Some(()) = try!(visitor.visit(self)) { }

        let values = match self.state.pop().unwrap() {
            State::Array(values) => values,
            state => panic!("Expected array, found {:?}", state),
        };

        self.state.push(State::Value(Value::Array(values)));

        Ok(())
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), Error>
        where T: serde::ser::Serialize,
    {
        try!(value.serialize(self));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        match *self.state.last_mut().unwrap() {
            State::Array(ref mut values) => { values.push(value); }
            ref state => panic!("expected array, found {:?}", state),
        }

        Ok(())
    }

    #[inline]
    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor,
    {
        let values = Vec::new();

        self.state.push(State::Object(values));

        while let Some(()) = try!(visitor.visit(self)) { }

        let values = match self.state.pop().unwrap() {
            State::Object(values) => values,
            state => panic!("expected object, found {:?}", state),
        };

        self.state.push(State::Value(Value::Map(values)));

        Ok(())
    }

    #[inline]
    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
        where K: serde::ser::Serialize,
              V: serde::ser::Serialize,
    {
        try!(key.serialize(self));

        let key = match self.state.pop().unwrap() {
            State::Value(key) => key,
            state => panic!("expected key, found {:?}", state),
        };

        try!(value.serialize(self));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        match *self.state.last_mut().unwrap() {
            State::Object(ref mut values) => { values.push((key, value)); }
            ref state => panic!("expected object, found {:?}", state),
        }

        Ok(())
    }

    /*
    #[inline]
    fn serialize_unit_variant(&mut self,
                          _name: &str,
                          _variant_index: usize,
                          variant: &str) -> Result<(), Error> {
        let mut values = BTreeMap::new();
        values.insert(String::from(variant), Value::Array(vec![]));

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                _name: &str,
                                _variant_index: usize,
                                variant: &str,
                                value: T) -> Result<(), Error>
        where T: serde::ser::Serialize,
    {
        let mut values = BTreeMap::new();
        values.insert(String::from(variant), to_value(&value));

        self.state.push(State::Value(Value::Object(values)));

        Ok(())
    }

    #[inline]
    fn serialize_tuple_variant<V>(&mut self,
                              _name: &str,
                              _variant_index: usize,
                              variant: &str,
                              visitor: V) -> Result<(), Error>
        where V: serde::ser::SeqVisitor,
    {
        try!(self.serialize_seq(visitor));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        let mut object = BTreeMap::new();

        object.insert(String::from(variant), value);

        self.state.push(State::Value(Value::Object(object)));

        Ok(())
    }

    #[inline]
    fn serialize_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               variant: &str,
                               visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor,
    {
        try!(self.serialize_map(visitor));

        let value = match self.state.pop().unwrap() {
            State::Value(value) => value,
            state => panic!("expected value, found {:?}", state),
        };

        let mut object = BTreeMap::new();

        object.insert(String::from(variant), value);

        self.state.push(State::Value(Value::Object(object)));

        Ok(())
    }
    */
}

pub fn to_value<T: ?Sized>(value: &T) -> Value
    where T: serde::Serialize
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser).ok().unwrap();
    ser.unwrap()
}
