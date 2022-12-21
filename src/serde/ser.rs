use locspan::Meta;
use serde::{ser::Impossible, Serialize};
use smallstr::SmallString;
use std::fmt;

use crate::{object::Key, Array, NumberBuf, Object, Value};

#[derive(Debug, Clone)]
pub enum SerializeError {
	Custom(String),
	NonStringKey,
}

impl fmt::Display for SerializeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Custom(msg) => msg.fmt(f),
			Self::NonStringKey => write!(f, "key must be a string"),
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
pub struct Serializer<F> {
	metadata_builder: F,
}

impl<F, M> Serializer<F>
where
	F: Fn() -> M,
{
	/// Creates a new [`Value<M>`] serializer using the given function to
	/// annotate the output value.
	pub fn new(metadata_builder: F) -> Self {
		Self { metadata_builder }
	}

	fn build_metadata(&self) -> M {
		(self.metadata_builder)()
	}
}

impl<F, M> serde::Serializer for Serializer<F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	type SerializeSeq = SerializeArray<M, F>;
	type SerializeTuple = SerializeArray<M, F>;
	type SerializeTupleStruct = SerializeArray<M, F>;
	type SerializeTupleVariant = SerializeTupleVariant<M, F>;
	type SerializeMap = SerializeObject<M, F>;
	type SerializeStruct = SerializeObject<M, F>;
	type SerializeStructVariant = SerializeStructVariant<M, F>;

	#[inline(always)]
	fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(Value::Boolean(v), self.build_metadata()))
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
		Ok(Meta(Value::Number(v.into()), self.build_metadata()))
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
		Ok(Meta(Value::Number(v.into()), self.build_metadata()))
	}

	#[inline(always)]
	fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(
			NumberBuf::try_from(v)
				.map(Value::Number)
				.unwrap_or(Value::Null),
			self.build_metadata(),
		))
	}

	#[inline(always)]
	fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(
			NumberBuf::try_from(v)
				.map(Value::Number)
				.unwrap_or(Value::Null),
			self.build_metadata(),
		))
	}

	#[inline(always)]
	fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
		let mut s = SmallString::new();
		s.push(v);
		Ok(Meta(Value::String(s), self.build_metadata()))
	}

	#[inline(always)]
	fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(Value::String(v.into()), self.build_metadata()))
	}

	#[inline(always)]
	fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
		let vec = v
			.iter()
			.map(|&b| Meta(Value::Number(b.into()), self.build_metadata()))
			.collect();
		Ok(Meta(Value::Array(vec), self.build_metadata()))
	}

	#[inline(always)]
	fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(Value::Null, self.build_metadata()))
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
	fn serialize_newtype_struct<T: ?Sized>(
		self,
		_name: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: Serialize,
	{
		value.serialize(self)
	}

	#[inline(always)]
	fn serialize_newtype_variant<T: ?Sized>(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<Self::Ok, Self::Error>
	where
		T: Serialize,
	{
		let mut obj = Object::new();
		let meta = self.build_metadata();
		let key_metadata = self.build_metadata();
		obj.insert(Meta(variant.into(), key_metadata), value.serialize(self)?);
		Ok(Meta(Value::Object(obj), meta))
	}

	#[inline(always)]
	fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
		self.serialize_unit()
	}

	#[inline(always)]
	fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: Serialize,
	{
		value.serialize(self)
	}

	#[inline(always)]
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
		Ok(SerializeArray {
			array: Vec::with_capacity(len.unwrap_or(0)),
			metadata_builder: self.metadata_builder,
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
			metadata_builder: self.metadata_builder,
		})
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
		Ok(SerializeObject {
			obj: Object::new(),
			next_key: None,
			metadata_builder: self.metadata_builder,
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
			metadata_builder: self.metadata_builder,
		})
	}

	fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
	where
		T: fmt::Display,
	{
		Ok(Meta(
			Value::String(value.to_string().into()),
			self.build_metadata(),
		))
	}
}

pub struct KeySerializer<M>(M);

impl<M> serde::Serializer for KeySerializer<M> {
	type Ok = Meta<Key, M>;
	type Error = SerializeError;

	type SerializeSeq = Impossible<Meta<Key, M>, SerializeError>;
	type SerializeTuple = Impossible<Meta<Key, M>, SerializeError>;
	type SerializeTupleStruct = Impossible<Meta<Key, M>, SerializeError>;
	type SerializeTupleVariant = Impossible<Meta<Key, M>, SerializeError>;
	type SerializeMap = Impossible<Meta<Key, M>, SerializeError>;
	type SerializeStruct = Impossible<Meta<Key, M>, SerializeError>;
	type SerializeStructVariant = Impossible<Meta<Key, M>, SerializeError>;

	#[inline]
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
	) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(variant.into(), self.0))
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
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
	}

	fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.to_string().into(), self.0))
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
		Ok(Meta(s, self.0))
	}

	#[inline]
	fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
		Ok(Meta(value.into(), self.0))
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
		Ok(Meta(value.to_string().into(), self.0))
	}
}

pub struct SerializeArray<M, F> {
	array: Array<M>,
	metadata_builder: F,
}

impl<M, F> SerializeArray<M, F>
where
	F: Fn() -> M,
{
	fn build_metadata(&self) -> M {
		(self.metadata_builder)()
	}
}

impl<M, F> serde::ser::SerializeSeq for SerializeArray<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		self.array
			.push(value.serialize(Serializer::new(self.metadata_builder.clone()))?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let meta = self.build_metadata();
		Ok(Meta(Value::Array(self.array), meta))
	}
}

impl<M, F> serde::ser::SerializeTuple for SerializeArray<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		serde::ser::SerializeSeq::serialize_element(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		serde::ser::SerializeSeq::end(self)
	}
}

impl<M, F> serde::ser::SerializeTupleStruct for SerializeArray<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		serde::ser::SerializeSeq::serialize_element(self, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		serde::ser::SerializeSeq::end(self)
	}
}

pub struct SerializeTupleVariant<M, F> {
	name: Key,
	array: Array<M>,
	metadata_builder: F,
}

impl<M, F> SerializeTupleVariant<M, F>
where
	F: Fn() -> M,
{
	fn build_metadata(&self) -> M {
		(self.metadata_builder)()
	}
}

impl<M, F> serde::ser::SerializeTupleVariant for SerializeTupleVariant<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		self.array
			.push(value.serialize(Serializer::new(self.metadata_builder.clone()))?);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let mut obj = Object::new();

		let key_meta = self.build_metadata();
		let value_meta = self.build_metadata();
		let meta = self.build_metadata();
		obj.insert(
			Meta(self.name, key_meta),
			Meta(Value::Array(self.array), value_meta),
		);

		Ok(Meta(Value::Object(obj), meta))
	}
}

pub struct SerializeStructVariant<M, F> {
	name: Key,
	obj: Object<M>,
	metadata_builder: F,
}

impl<M, F> SerializeStructVariant<M, F>
where
	F: Fn() -> M,
{
	fn build_metadata(&self) -> M {
		(self.metadata_builder)()
	}
}

impl<M, F> serde::ser::SerializeStructVariant for SerializeStructVariant<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_field<T: ?Sized>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		let key = Meta(key.into(), self.build_metadata());
		self.obj.insert(
			key,
			value.serialize(Serializer::new(self.metadata_builder.clone()))?,
		);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let mut obj = Object::new();

		let key_meta = self.build_metadata();
		let value_meta = self.build_metadata();
		let meta = self.build_metadata();
		obj.insert(
			Meta(self.name, key_meta),
			Meta(Value::Object(self.obj), value_meta),
		);

		Ok(Meta(Value::Object(obj), meta))
	}
}

pub struct SerializeObject<M, F> {
	obj: Object<M>,
	next_key: Option<Meta<Key, M>>,
	metadata_builder: F,
}

impl<M, F> SerializeObject<M, F>
where
	F: Fn() -> M,
{
	fn build_metadata(&self) -> M {
		(self.metadata_builder)()
	}
}

impl<M, F> serde::ser::SerializeMap for SerializeObject<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		self.next_key = Some(key.serialize(KeySerializer(self.build_metadata()))?);
		Ok(())
	}

	fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		let key = self
			.next_key
			.take()
			.expect("serialize_value called before serialize_key");
		self.obj.insert(
			key,
			value.serialize(Serializer::new(self.metadata_builder.clone()))?,
		);
		Ok(())
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		let meta = self.build_metadata();
		Ok(Meta(Value::Object(self.obj), meta))
	}
}

impl<M, F> serde::ser::SerializeStruct for SerializeObject<M, F>
where
	F: Clone + Fn() -> M,
{
	type Ok = Meta<Value<M>, M>;
	type Error = SerializeError;

	fn serialize_field<T: ?Sized>(
		&mut self,
		key: &'static str,
		value: &T,
	) -> Result<(), Self::Error>
	where
		T: Serialize,
	{
		serde::ser::SerializeMap::serialize_entry(self, key, value)
	}

	fn end(self) -> Result<Self::Ok, Self::Error> {
		serde::ser::SerializeMap::end(self)
	}
}
