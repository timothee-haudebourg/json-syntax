use std::fmt;

use locspan::Meta;
use serde::{
	de::{
		DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess, SeqAccess, Unexpected,
		VariantAccess,
	},
	forward_to_deserialize_any,
};

use crate::{
	object::{Entry, Key},
	Array, Object, Value,
};

impl<M> Value<M> {
	#[cold]
	fn invalid_type<E>(&self, exp: &dyn Expected) -> E
	where
		E: serde::de::Error,
	{
		serde::de::Error::invalid_type(self.unexpected(), exp)
	}

	#[cold]
	fn unexpected(&self) -> Unexpected {
		match self {
			Self::Null => Unexpected::Unit,
			Self::Boolean(b) => Unexpected::Bool(*b),
			Self::Number(n) => match n.as_u64() {
				Some(u) => Unexpected::Unsigned(u),
				None => match n.as_i64() {
					Some(i) => Unexpected::Signed(i),
					None => Unexpected::Float(n.as_f64_lossy()),
				},
			},
			Self::String(s) => Unexpected::Str(s),
			Self::Array(_) => Unexpected::Seq,
			Self::Object(_) => Unexpected::Map,
		}
	}
}

impl<'de> IntoDeserializer<'de, DeserializeError> for Value {
	type Deserializer = Self;

	fn into_deserializer(self) -> Self::Deserializer {
		self
	}
}

#[derive(Debug, Clone)]
pub enum DeserializeError {
	Custom(String),
	NonStringKey,
}

impl From<json_number::serde::Unexpected> for DeserializeError {
	fn from(u: json_number::serde::Unexpected) -> Self {
		Self::Custom(u.to_string())
	}
}

impl fmt::Display for DeserializeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Custom(msg) => msg.fmt(f),
			Self::NonStringKey => write!(f, "key must be a string"),
		}
	}
}

impl std::error::Error for DeserializeError {}

impl serde::de::Error for DeserializeError {
	fn custom<T>(msg: T) -> Self
	where
		T: fmt::Display,
	{
		Self::Custom(msg.to_string())
	}
}

macro_rules! deserialize_number {
	($method:ident) => {
		fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
		where
			V: serde::de::Visitor<'de>,
		{
			match self {
				Value::Number(n) => Ok(n.deserialize_any(visitor)?),
				_ => Err(self.invalid_type(&visitor)),
			}
		}
	};
}

impl<'de, M> serde::Deserializer<'de> for Value<M> {
	type Error = DeserializeError;

	#[inline]
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Self::Null => visitor.visit_unit(),
			Self::Boolean(v) => visitor.visit_bool(v),
			Self::Number(n) => Ok(n.deserialize_any(visitor)?),
			Self::String(s) => visitor.visit_string(s.into_string()),
			Self::Array(a) => visit_array(a, visitor),
			Self::Object(o) => visit_object(o, visitor),
		}
	}

	deserialize_number!(deserialize_i8);
	deserialize_number!(deserialize_i16);
	deserialize_number!(deserialize_i32);
	deserialize_number!(deserialize_i64);
	deserialize_number!(deserialize_i128);
	deserialize_number!(deserialize_u8);
	deserialize_number!(deserialize_u16);
	deserialize_number!(deserialize_u32);
	deserialize_number!(deserialize_u64);
	deserialize_number!(deserialize_u128);
	deserialize_number!(deserialize_f32);
	deserialize_number!(deserialize_f64);

	#[inline]
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::Null => visitor.visit_none(),
			_ => visitor.visit_some(self),
		}
	}

	#[inline]
	fn deserialize_enum<V>(
		self,
		_name: &str,
		_variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		let (variant, value) = match self {
			Value::Object(value) => {
				let mut iter = value.into_iter();
				let Entry {
					key: Meta(variant, _),
					value: Meta(value, _),
				} = match iter.next() {
					Some(v) => v,
					None => {
						return Err(serde::de::Error::invalid_value(
							Unexpected::Map,
							&"map with a single key",
						));
					}
				};
				// enums are encoded in json as maps with a single key:value pair
				if iter.next().is_some() {
					return Err(serde::de::Error::invalid_value(
						Unexpected::Map,
						&"map with a single key",
					));
				}
				(variant, Some(value))
			}
			Value::String(variant) => (variant, None),
			other => {
				return Err(serde::de::Error::invalid_type(
					other.unexpected(),
					&"string or map",
				));
			}
		};

		visitor.visit_enum(EnumDeserializer { variant, value })
	}

	#[inline]
	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::Boolean(v) => visitor.visit_bool(v),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_string(visitor)
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_string(visitor)
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::String(v) => visitor.visit_string(v.into_string()),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_byte_buf(visitor)
	}

	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::String(v) => visitor.visit_string(v.into_string()),
			Value::Array(v) => visit_array(v, visitor),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::Null => visitor.visit_unit(),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_unit(visitor)
	}

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::Array(v) => visit_array(v, visitor),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		_len: usize,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_seq(visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::Object(v) => visit_object(v, visitor),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self {
			Value::Array(v) => visit_array(v, visitor),
			Value::Object(v) => visit_object(v, visitor),
			_ => Err(self.invalid_type(&visitor)),
		}
	}

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.deserialize_string(visitor)
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		drop(self);
		visitor.visit_unit()
	}
}

fn visit_array<'de, M, V>(a: Array<M>, visitor: V) -> Result<V::Value, DeserializeError>
where
	V: serde::de::Visitor<'de>,
{
	let len = a.len();
	let mut deserializer = ArrayDeserializer::new(a);
	let seq = visitor.visit_seq(&mut deserializer)?;
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

fn visit_object<'de, M, V>(o: Object<M>, visitor: V) -> Result<V::Value, DeserializeError>
where
	V: serde::de::Visitor<'de>,
{
	let len = o.len();
	let mut deserializer = ObjectDeserializer::new(o);
	let map = visitor.visit_map(&mut deserializer)?;
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

struct ArrayDeserializer<M> {
	iter: std::vec::IntoIter<Meta<Value<M>, M>>,
}

impl<M> ArrayDeserializer<M> {
	fn new(array: Array<M>) -> Self {
		Self {
			iter: array.into_iter(),
		}
	}
}

impl<'de, M> SeqAccess<'de> for ArrayDeserializer<M> {
	type Error = DeserializeError;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		match self.iter.next() {
			Some(Meta(value, _)) => seed.deserialize(value).map(Some),
			None => Ok(None),
		}
	}

	fn size_hint(&self) -> Option<usize> {
		match self.iter.size_hint() {
			(lower, Some(upper)) if lower == upper => Some(upper),
			_ => None,
		}
	}
}

struct ObjectDeserializer<M> {
	iter: std::vec::IntoIter<Entry<M>>,
	value: Option<Value<M>>,
}

impl<M> ObjectDeserializer<M> {
	fn new(obj: Object<M>) -> Self {
		Self {
			iter: obj.into_iter(),
			value: None,
		}
	}
}

impl<'de, M> MapAccess<'de> for ObjectDeserializer<M> {
	type Error = DeserializeError;

	fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		match self.iter.next() {
			Some(Entry { key, value }) => {
				self.value = Some(value.into_value());
				let key_de = MapKeyDeserializer {
					key: key.into_value(),
				};
				seed.deserialize(key_de).map(Some)
			}
			None => Ok(None),
		}
	}

	fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		match self.value.take() {
			Some(value) => seed.deserialize(value),
			None => Err(serde::de::Error::custom("value is missing")),
		}
	}

	fn size_hint(&self) -> Option<usize> {
		match self.iter.size_hint() {
			(lower, Some(upper)) if lower == upper => Some(upper),
			_ => None,
		}
	}
}

struct MapKeyDeserializer {
	key: Key,
}

macro_rules! deserialize_integer_key {
	($method:ident => $visit:ident) => {
		fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
		where
			V: serde::de::Visitor<'de>,
		{
			match (self.key.parse(), self.key) {
				(Ok(integer), _) => visitor.$visit(integer),
				(Err(_), key) => visitor.visit_string(key.into_string()),
			}
		}
	};
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer {
	type Error = DeserializeError;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_string(self.key.into_string())
	}

	deserialize_integer_key!(deserialize_i8 => visit_i8);
	deserialize_integer_key!(deserialize_i16 => visit_i16);
	deserialize_integer_key!(deserialize_i32 => visit_i32);
	deserialize_integer_key!(deserialize_i64 => visit_i64);
	deserialize_integer_key!(deserialize_i128 => visit_i128);
	deserialize_integer_key!(deserialize_u8 => visit_u8);
	deserialize_integer_key!(deserialize_u16 => visit_u16);
	deserialize_integer_key!(deserialize_u32 => visit_u32);
	deserialize_integer_key!(deserialize_u64 => visit_u64);
	deserialize_integer_key!(deserialize_u128 => visit_u128);

	#[inline]
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		// Map keys cannot be null.
		visitor.visit_some(self)
	}

	#[inline]
	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		visitor.visit_newtype_struct(self)
	}

	fn deserialize_enum<V>(
		self,
		name: &'static str,
		variants: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		self.key
			.into_deserializer()
			.deserialize_enum(name, variants, visitor)
	}

	forward_to_deserialize_any! {
		bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
		tuple_struct map struct identifier ignored_any
	}
}

struct EnumDeserializer<M> {
	variant: Key,
	value: Option<Value<M>>,
}

impl<'de, M> EnumAccess<'de> for EnumDeserializer<M> {
	type Error = DeserializeError;
	type Variant = VariantDeserializer<M>;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer<M>), Self::Error>
	where
		V: DeserializeSeed<'de>,
	{
		let variant = self.variant.into_deserializer();
		let visitor = VariantDeserializer { value: self.value };
		seed.deserialize(variant).map(|v| (v, visitor))
	}
}

struct VariantDeserializer<M> {
	value: Option<Value<M>>,
}

impl<'de, M> VariantAccess<'de> for VariantDeserializer<M> {
	type Error = DeserializeError;

	fn unit_variant(self) -> Result<(), Self::Error> {
		match self.value {
			Some(value) => serde::Deserialize::deserialize(value),
			None => Ok(()),
		}
	}

	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
	where
		T: DeserializeSeed<'de>,
	{
		match self.value {
			Some(value) => seed.deserialize(value),
			None => Err(serde::de::Error::invalid_type(
				Unexpected::UnitVariant,
				&"newtype variant",
			)),
		}
	}

	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.value {
			Some(Value::Array(v)) => {
				if v.is_empty() {
					visitor.visit_unit()
				} else {
					visit_array(v, visitor)
				}
			}
			Some(other) => Err(serde::de::Error::invalid_type(
				other.unexpected(),
				&"tuple variant",
			)),
			None => Err(serde::de::Error::invalid_type(
				Unexpected::UnitVariant,
				&"tuple variant",
			)),
		}
	}

	fn struct_variant<V>(
		self,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value, Self::Error>
	where
		V: serde::de::Visitor<'de>,
	{
		match self.value {
			Some(Value::Object(v)) => visit_object(v, visitor),
			Some(other) => Err(serde::de::Error::invalid_type(
				other.unexpected(),
				&"struct variant",
			)),
			None => Err(serde::de::Error::invalid_type(
				Unexpected::UnitVariant,
				&"struct variant",
			)),
		}
	}
}
