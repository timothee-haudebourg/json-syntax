use std::fmt;
use locspan::Meta;
use serde::{de, Deserialize, Deserializer};
// use serde_json::error::Error;

use crate::{Value, Array, Object, object};

impl<'de> Deserialize<'de> for Value<()> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(ValueVisitor)
	}
}

impl<'de> Deserialize<'de> for Object<()> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_map(ObjectVisitor)
	}
}

struct ValueVisitor;

impl<'de> de::Visitor<'de> for ValueVisitor {
	type Value = Value<()>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("any valid JSON value")
	}

	#[inline]
	fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value<()>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value<()>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value<()>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value<()>, E> {
		match value.try_into() {
			Ok(v) => Ok(v),
			Err(_) => Err(E::invalid_value(de::Unexpected::Float(value), &mut ExpectedFiniteFloat))
		}
	}

	#[inline]
	fn visit_str<E: de::Error>(self, value: &str) -> Result<Value<()>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_string<E: de::Error>(self, value: String) -> Result<Value<()>, E> {
		Ok(value.into())
	}

	#[inline]
	fn visit_none<E: de::Error>(self) -> Result<Value<()>, E> {
		Ok(Value::Null)
	}

	#[inline]
	fn visit_some<D>(self, deserializer: D) -> Result<Value<()>, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer)
	}

	#[inline]
	fn visit_unit<E: de::Error>(self) -> Result<Value<()>, E> {
		Ok(Value::Null)
	}

	#[inline]
	fn visit_seq<V>(self, visitor: V) -> Result<Value<()>, V::Error>
	where
		V: de::SeqAccess<'de>,
	{
		ArrayVisitor.visit_seq(visitor).map(Into::into)
	}

	fn visit_map<V>(self, visitor: V) -> Result<Value<()>, V::Error>
	where
		V: de::MapAccess<'de>,
	{
		ObjectVisitor.visit_map(visitor).map(Into::into)
	}
}

struct ExpectedFiniteFloat;

impl de::Expected for ExpectedFiniteFloat {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "a finite float")
	}
}

struct ArrayVisitor;

impl<'de> de::Visitor<'de> for ArrayVisitor {
	type Value = Array<()>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("JSON array")
	}

	#[inline]
	fn visit_seq<V>(self, mut visitor: V) -> Result<Array<()>, V::Error>
	where
		V: de::SeqAccess<'de>,
	{
		let mut arr = Array::<()>::with_capacity(visitor.size_hint().unwrap_or(0));
		while let Some(v) = visitor.next_element::<Value<()>>()? {
			arr.push(Meta(v, ()));
		}
		Ok(arr)
	}
}

struct ObjectVisitor;

impl<'de> de::Visitor<'de> for ObjectVisitor {
	type Value = Object<()>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("JSON object")
	}

	fn visit_map<V>(self, mut visitor: V) -> Result<Object<()>, V::Error>
	where
		V: de::MapAccess<'de>,
	{
		let mut obj = Object::<()>::with_capacity(visitor.size_hint().unwrap_or(0));
		while let Some((k, v)) = visitor.next_entry::<object::Key, Value<()>>()? {
			obj.insert(Meta(k, ()), Meta(v, ()));
		}
		Ok(obj)
	}
}

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl std::error::Error for Error {
	
}

impl serde::de::Error for Error {
	fn custom<T>(msg: T) -> Self where T:fmt::Display {
		Self(msg.to_string())
	}
}

macro_rules! deserialize_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: de::Visitor<'de>,
        {
			match self {
				Value::Number(n) => n.deserialize_any(visitor).map_err(|e| Error(e.to_string())),
				_ => Err(self.invalid_type(&visitor))
			}
        }
    };
}

fn visit_array_ref<'de, M, V>(array: &'de [Value<M>], visitor: V) -> Result<V::Value, Error>
where
    V: de::Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqRefDeserializer::new(array);
    let seq = tri!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_object_ref<'de, V>(object: &'de Map<String, Value>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapRefDeserializer::new(object);
    let map = tri!(visitor.visit_map(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de, M> Deserializer<'de> for &'de Value<M> {
	type Error = Error;

	#[inline]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Value::Null => visitor.visit_unit(),
			Value::Boolean(v) => visitor.visit_bool(*v),
			Value::Number(v) => v.deserialize_any(visitor).map_err(|e| Error(e.to_string())),
			Value::String(v) => visitor.visit_borrowed_str(v),
			Value::Array(v) => visit_array_ref(visitor),
			Value::Object(v) => v.deserialize_any(visitor),
		}
	}

	deserialize_number!(deserialize_i8);
	deserialize_number!(deserialize_i16);
	deserialize_number!(deserialize_i32);
	deserialize_number!(deserialize_i64);
	deserialize_number!(deserialize_u8);
	deserialize_number!(deserialize_u16);
	deserialize_number!(deserialize_u32);
	deserialize_number!(deserialize_u64);
	deserialize_number!(deserialize_f32);
	deserialize_number!(deserialize_f64);

	#[inline]
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		if self.is_null() {
			visitor.visit_none()
		} else {
			visitor.visit_some(self)
		}
	}

	#[inline]
	fn deserialize_enum<V>(
		self,
		name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		match self {
			Value::String(v) => v.deserialize_enum(name, variants, visitor),
			Value::Object(v) => v.deserialize_enum(name, variants, visitor),
			other => Err(de::Error::invalid_type(other.unexpected(), &"string or map")),
		}
	}

	#[inline]
	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		if let Some(v) = self.to_bool() {
			visitor.visit_bool(v)
		} else {
			Err(self.invalid_type(&visitor))
		}
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		if let Some(v) = self.as_string() {
			v.deserialize_str(visitor)
		} else {
			Err(self.invalid_type(&visitor))
		}
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		match self.destructure_ref() {
			DestructuredRef::String(v) => v.deserialize_bytes(visitor),
			DestructuredRef::Array(v) => v.deserialize_bytes(visitor),
			other => Err(other.invalid_type(&visitor)),
		}
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_bytes(visitor)
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		if self.is_null() {
			visitor.visit_unit()
		} else {
			Err(self.invalid_type(&visitor))
		}
	}

	fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		if let Some(v) = self.as_array() {
			v.deserialize_seq(visitor)
		} else {
			Err(self.invalid_type(&visitor))
		}
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		visitor: V,
	) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		if let Some(v) = self.as_object() {
			v.deserialize_map(visitor)
		} else {
			Err(self.invalid_type(&visitor))
		}
	}

	fn deserialize_struct<V>(
		self,
		name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		match self.destructure_ref() {
			DestructuredRef::Array(v) => v.deserialize_struct(name, fields, visitor),
			DestructuredRef::Object(v) => v.deserialize_struct(name, fields, visitor),
			other => Err(other.invalid_type(&visitor)),
		}
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
	where
		V: de::Visitor<'de>,
	{
		visitor.visit_unit()
	}
}