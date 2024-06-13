use serde::{ser::Impossible, Serialize};
use smallstr::SmallString;
use std::fmt;

use super::NUMBER_TOKEN;
use crate::{object::Key, Array, NumberBuf, Object, Value};

impl Serialize for Value {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Null => serializer.serialize_unit(),
			Self::Boolean(b) => serializer.serialize_bool(*b),
			Self::Number(n) => n.serialize(serializer),
			Self::String(s) => serializer.serialize_str(s),
			Self::Array(a) => {
				use serde::ser::SerializeSeq;
				let mut seq = serializer.serialize_seq(Some(a.len()))?;

				for item in a {
					seq.serialize_element(item)?
				}

				seq.end()
			}
			Self::Object(o) => o.serialize(serializer),
		}
	}
}

impl Serialize for Object {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeMap;
		let mut map = serializer.serialize_map(Some(self.len()))?;

		for entry in self {
			map.serialize_entry(entry.key.as_str(), &entry.value)?;
		}

		map.end()
	}
}

#[derive(Debug, Clone)]
pub enum SerializeError {
	Custom(String),
	NonStringKey,
	MalformedHighPrecisionNumber,
}

impl fmt::Display for SerializeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Custom(msg) => msg.fmt(f),
			Self::NonStringKey => write!(f, "key must be a string"),
			Self::MalformedHighPrecisionNumber => write!(f, "malformed high-precision number"),
		}
	}
}

impl std::error::Error for SerializeError {}

impl serde::ser::Error for SerializeError {
	fn custom<T>(msg: T) -> Self
	where
		T: fmt::Display,
	{
		Self::Custom(msg.to_string())
	}
}

/// [`Value`] serializer.
pub struct Serializer;

impl serde::Serializer for Serializer {
	type Ok = Value;
	type Error = SerializeError;

	type SerializeSeq = SerializeArray;
	type SerializeTuple = SerializeArray;
	type SerializeTupleStruct = SerializeArray;
	type SerializeTupleVariant = SerializeTupleVariant;
	type SerializeMap = SerializeMap;
	type SerializeStruct = SerializeMap;
	type SerializeStructVariant = SerializeStructVariant;

	#[inline(always)]
	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Boolean(v))
	}

	#[inline(always)]
	fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
		self.serialize_i64(v as i64)
	}

	#[inline(always)]
	fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
		self.serialize_i64(v as i64)
	}

	#[inline(always)]
	fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
		self.serialize_i64(v as i64)
	}

	#[inline(always)]
	fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Number(v.into()))
	}

	#[inline(always)]
	fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
		self.serialize_u64(v as u64)
	}

	#[inline(always)]
	fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
		self.serialize_u64(v as u64)
	}

	#[inline(always)]
	fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
		self.serialize_u64(v as u64)
	}

	#[inline(always)]
	fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Number(v.into()))
	}

	#[inline(always)]
	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		Ok(NumberBuf::try_from(v)
			.map(Value::Number)
			.unwrap_or(Value::Null))
	}

	#[inline(always)]
	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(NumberBuf::try_from(v)
			.map(Value::Number)
			.unwrap_or(Value::Null))
	}

	#[inline(always)]
	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		let mut s = SmallString::new();
		s.push(v);
		Ok(Value::String(s))
	}

	#[inline(always)]
	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		Ok(Value::String(v.into()))
	}

	#[inline(always)]
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		let vec = v.iter().map(|&b| Value::Number(b.into())).collect();
		Ok(Value::Array(vec))
	}

	#[inline(always)]
	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Null)
	}

	#[inline(always)]
	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		self.serialize_unit()
	}

	#[inline(always)]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		self.serialize_str(variant)
	}

	#[inline(always)]
	fn serialize_newtype_struct<T>(
		self,
		_name: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	#[inline(always)]
	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let mut obj = Object::new();
		obj.insert(variant.into(), value.serialize(self)?);
		Ok(Value::Object(obj))
	}

	#[inline(always)]
	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		self.serialize_unit()
	}

	#[inline(always)]
	fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	#[inline(always)]
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Ok(SerializeArray {
			array: Vec::with_capacity(len.unwrap_or(0)),
		})
	}

	#[inline(always)]
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		self.serialize_seq(Some(len))
	}

	#[inline(always)]
	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		self.serialize_seq(Some(len))
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		Ok(SerializeTupleVariant {
			name: variant.into(),
			array: Vec::with_capacity(len),
		})
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Ok(SerializeMap::Object {
			obj: Object::new(),
			next_key: None,
		})
	}

	fn serialize_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		self.serialize_map(Some(len))
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		Ok(SerializeStructVariant {
			name: variant.into(),
			obj: Object::new(),
		})
	}

	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + fmt::Display,
	{
		Ok(Value::String(value.to_string().into()))
	}
}

pub struct StringNumberSerializer;

impl serde::Serializer for StringNumberSerializer {
	type Ok = NumberBuf;
	type Error = SerializeError;

	type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
	type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
	type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
	type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
	type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
	type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
	type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

	fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		NumberBuf::new(v.as_bytes().into())
			.map_err(|_| SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_newtype_struct<T>(
		self,
		_name: &'static str,
		_value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		Err(SerializeError::MalformedHighPrecisionNumber)
	}
}

pub struct KeySerializer;

impl serde::Serializer for KeySerializer {
	type Ok = Key;
	type Error = SerializeError;

	type SerializeSeq = Impossible<Key, SerializeError>;
	type SerializeTuple = Impossible<Key, SerializeError>;
	type SerializeTupleStruct = Impossible<Key, SerializeError>;
	type SerializeTupleVariant = Impossible<Key, SerializeError>;
	type SerializeMap = Impossible<Key, SerializeError>;
	type SerializeStruct = Impossible<Key, SerializeError>;
	type SerializeStructVariant = Impossible<Key, SerializeError>;

	#[inline]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		Ok(variant.into())
	}

	#[inline]
	fn serialize_newtype_struct<T>(
		self,
		_name: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
		Ok(value.to_string().into())
	}

	fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	#[inline]
	fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
		let mut s = Key::new();
		s.push(value);
		Ok(s)
	}

	#[inline]
	fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
		Ok(value.into())
	}

	fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_newtype_variant<T>(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(SerializeError::NonStringKey)
	}

	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + Serialize,
	{
		Err(SerializeError::NonStringKey)
	}

	fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleStruct, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_struct(
		self,
		_name: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStruct, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant, Self::Error> {
		Err(SerializeError::NonStringKey)
	}

	fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: ?Sized + fmt::Display,
	{
		Ok(value.to_string().into())
	}
}

pub struct SerializeArray {
	array: Array,
}

impl serde::ser::SerializeSeq for SerializeArray {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.array.push(value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		Ok(Value::Array(self.array))
	}
}

impl serde::ser::SerializeTuple for SerializeArray {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		serde::ser::SerializeSeq::serialize_element(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		serde::ser::SerializeSeq::end(self)
	}
}

impl serde::ser::SerializeTupleStruct for SerializeArray {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		serde::ser::SerializeSeq::serialize_element(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		serde::ser::SerializeSeq::end(self)
	}
}

pub struct SerializeTupleVariant {
	name: Key,
	array: Array,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		self.array.push(value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let mut obj = Object::new();
		obj.insert(self.name, Value::Array(self.array));

		Ok(Value::Object(obj))
	}
}

pub struct SerializeStructVariant {
	name: Key,
	obj: Object,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		let key = key.into();
		self.obj.insert(key, value.serialize(Serializer)?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let mut obj = Object::new();
		obj.insert(self.name, Value::Object(self.obj));

		Ok(Value::Object(obj))
	}
}

pub enum SerializeMap {
	Object { obj: Object, next_key: Option<Key> },
	Number(Option<NumberBuf>),
}

impl serde::ser::SerializeMap for SerializeMap {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		match self {
			Self::Number(_) => Err(SerializeError::MalformedHighPrecisionNumber),
			Self::Object { obj, next_key } => {
				let key = key.serialize(KeySerializer)?;

				if obj.is_empty() && key == NUMBER_TOKEN {
					*self = Self::Number(None)
				} else {
					*next_key = Some(key);
				}

				Ok(())
			}
		}
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		match self {
			Self::Number(n) => {
				*n = Some(value.serialize(StringNumberSerializer)?);
				Ok(())
			}
			Self::Object { obj, next_key } => {
				let key = next_key
					.take()
					.expect("serialize_value called before serialize_key");
				obj.insert(key, value.serialize(Serializer)?);
				Ok(())
			}
		}
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		match self {
			Self::Number(Some(n)) => Ok(Value::Number(n)),
			Self::Number(None) => Err(SerializeError::MalformedHighPrecisionNumber),
			Self::Object { obj, .. } => Ok(Value::Object(obj)),
		}
	}
}

impl serde::ser::SerializeStruct for SerializeMap {
	type Ok = Value;
	type Error = SerializeError;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
	where
		T: ?Sized + Serialize,
	{
		serde::ser::SerializeMap::serialize_entry(self, key, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		serde::ser::SerializeMap::end(self)
	}
}
