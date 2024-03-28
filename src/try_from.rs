use core::fmt;
use std::{collections::BTreeMap, marker::PhantomData, str::FromStr};

use crate::{array::JsonArray, code_map::Mapped, CodeMap, Kind, Object, Value};

#[derive(Debug)]
pub struct Unexpected {
	pub expected: Kind,
	pub found: Kind,
}

impl fmt::Display for Unexpected {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "expected {}, found {}", self.expected, self.found)
	}
}

pub struct NumberType<T>(PhantomData<T>);

impl<T> Clone for NumberType<T> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T> Copy for NumberType<T> {}

impl<T> Default for NumberType<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<T> fmt::Debug for NumberType<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "NumberType")
	}
}

impl<T> fmt::Display for NumberType<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "NumberType")
	}
}

pub enum TryIntoNumberError<T> {
	Unexpected(Unexpected),
	OutOfBounds(T),
}

impl<T> TryIntoNumberError<T> {
	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TryIntoNumberError<U> {
		match self {
			Self::Unexpected(e) => TryIntoNumberError::Unexpected(e),
			Self::OutOfBounds(t) => TryIntoNumberError::OutOfBounds(f(t)),
		}
	}
}

impl std::error::Error for Unexpected {}

pub trait TryFromJsonSyntax: Sized {
	type Error;

	fn try_from_json_syntax(json: &Value, code_map: &CodeMap) -> Result<Self, Self::Error> {
		Self::try_from_json_syntax_at(json, code_map, 0)
	}

	fn try_from_json_syntax_at(
		json: &Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error>;
}

impl<T: TryFromJsonSyntax> TryFromJsonSyntax for Box<T> {
	type Error = T::Error;

	fn try_from_json_syntax_at(
		json: &Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		T::try_from_json_syntax_at(json, code_map, offset).map(Box::new)
	}
}

impl<T: TryFromJsonSyntax> TryFromJsonSyntax for Option<T> {
	type Error = T::Error;

	fn try_from_json_syntax_at(
		json: &Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Null => Ok(None),
			other => T::try_from_json_syntax_at(other, code_map, offset).map(Some),
		}
	}
}

pub trait TryFromJsonObject: Sized {
	type Error;

	fn try_from_json_object(object: &Object, code_map: &CodeMap) -> Result<Self, Self::Error> {
		Self::try_from_json_object_at(object, code_map, 0)
	}

	fn try_from_json_object_at(
		object: &Object,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error>;
}

impl<T: TryFromJsonObject> TryFromJsonObject for Box<T> {
	type Error = T::Error;

	fn try_from_json_object_at(
		object: &Object,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		T::try_from_json_object_at(object, code_map, offset).map(Box::new)
	}
}

impl TryFromJsonSyntax for () {
	type Error = Mapped<Unexpected>;

	fn try_from_json_syntax_at(
		json: &Value,
		_code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Null => Ok(()),
			other => Err(Mapped::new(
				offset,
				Unexpected {
					expected: Kind::Null,
					found: other.kind(),
				},
			)),
		}
	}
}

impl TryFromJsonSyntax for bool {
	type Error = Mapped<Unexpected>;

	fn try_from_json_syntax_at(
		json: &Value,
		_code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Boolean(value) => Ok(*value),
			other => Err(Mapped::new(
				offset,
				Unexpected {
					expected: Kind::Boolean,
					found: other.kind(),
				},
			)),
		}
	}
}

macro_rules! number_from_json_syntax {
	($($ty:ident),*) => {
		$(
			impl TryFromJsonSyntax for $ty {
				type Error = Mapped<TryIntoNumberError<NumberType<$ty>>>;

				fn try_from_json_syntax_at(json: &Value, _code_map: &CodeMap, offset: usize) -> Result<Self, Self::Error> {
					match json {
						Value::Number(value) => value.parse().map_err(|_| Mapped::new(offset, TryIntoNumberError::OutOfBounds(NumberType::default()))),
						other => Err(Mapped::new(offset, TryIntoNumberError::Unexpected(Unexpected {
							expected: Kind::Number,
							found: other.kind()
						})))
					}
				}
			}
		)*
	};
}

number_from_json_syntax!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

impl TryFromJsonSyntax for String {
	type Error = Mapped<Unexpected>;

	fn try_from_json_syntax_at(
		json: &Value,
		_code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::String(value) => Ok(value.to_string()),
			other => Err(Mapped::new(
				offset,
				Unexpected {
					expected: Kind::String,
					found: other.kind(),
				},
			)),
		}
	}
}

impl<T: TryFromJsonSyntax> TryFromJsonSyntax for Vec<T>
where
	T::Error: From<Mapped<Unexpected>>,
{
	type Error = T::Error;

	fn try_from_json_syntax_at(
		json: &Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Array(value) => value
				.iter_mapped(code_map, offset)
				.map(|item| T::try_from_json_syntax_at(item.value, code_map, item.offset))
				.collect::<Result<Vec<_>, _>>(),
			other => Err(Mapped::new(
				offset,
				Unexpected {
					expected: Kind::Array,
					found: other.kind(),
				},
			)
			.into()),
		}
	}
}

impl<K: FromStr + Ord, V: TryFromJsonSyntax> TryFromJsonSyntax for BTreeMap<K, V>
where
	V::Error: From<Mapped<Unexpected>> + From<Mapped<K::Err>>,
{
	type Error = V::Error;

	fn try_from_json_syntax_at(
		json: &Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Object(object) => {
				let mut result = BTreeMap::new();

				for entry in object.iter_mapped(code_map, offset) {
					result.insert(
						entry
							.value
							.key
							.value
							.parse()
							.map_err(|e| Mapped::new(entry.value.key.offset, e))?,
						V::try_from_json_syntax_at(
							entry.value.value.value,
							code_map,
							entry.value.value.offset,
						)?,
					);
				}

				Ok(result)
			}
			other => Err(Mapped::new(
				offset,
				Unexpected {
					expected: Kind::Array,
					found: other.kind(),
				},
			)
			.into()),
		}
	}
}
