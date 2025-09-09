use core::fmt;
use std::{collections::BTreeMap, marker::PhantomData, str::FromStr};

use crate::{array::JsonArray, code_map::Mapped, CodeMap, Kind, KindSet, Object, Value};

/// Conversion from JSON syntax, with code mapping info.
///
/// This trait is very similar to [`TryFrom<Value>`] but also passes code
/// code mapping info to the conversion function.
pub trait TryFromJson: Sized {
	/// Error that may be returned by the conversion function.
	type Error;

	/// Tries to convert the given JSON value into `Self`, using the given
	/// `code_map`.
	///
	/// It is assumed that the offset of `value` in the code map is `0`, for
	/// instance if it is the output of a [`Parse`](crate::Parse) trait
	/// function.
	fn try_from_json(value: Value, code_map: &CodeMap) -> Result<Self, Self::Error> {
		Self::try_from_json_at(value, code_map, 0)
	}

	/// Tries to convert the given JSON value into `Self`, using the given
	/// `code_map` and the offset of `value` in the code map.
	///
	/// Note to implementors: use the [`JsonArray::iter_mapped`] and
	/// [`Object::iter_mapped`] methods to visit arrays and objects while
	/// keeping track of the code map offset of each visited item.
	fn try_from_json_at(
		value: Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error>;
}

impl<T: TryFromJson> TryFromJson for Box<T> {
	type Error = T::Error;

	fn try_from_json_at(
		json: Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		T::try_from_json_at(json, code_map, offset).map(Box::new)
	}
}

impl<T: TryFromJson> TryFromJson for Option<T> {
	type Error = T::Error;

	fn try_from_json_at(
		json: Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Null => Ok(None),
			other => T::try_from_json_at(other, code_map, offset).map(Some),
		}
	}
}

/// Conversion from JSON syntax object, with code mapping info.
///
/// This trait is very similar to [`TryFrom<Object>`] but also passes code
/// code mapping info to the conversion function.
pub trait TryFromJsonObject: Sized {
	type Error;

	/// Tries to convert the given JSON object into `Self`, using the given
	/// `code_map`.
	///
	/// It is assumed that the offset of `object` in the code map is `0`, for
	/// instance if it is the output of a [`Parse`](crate::Parse) trait
	/// function.
	fn try_from_json_object(object: &Object, code_map: &CodeMap) -> Result<Self, Self::Error> {
		Self::try_from_json_object_at(object, code_map, 0)
	}

	/// Tries to convert the given JSON object into `Self`, using the given
	/// `code_map` and the offset of `object` in the code map.
	///
	/// Note to implementors: use the [`JsonArray::iter_mapped`] and
	/// [`Object::iter_mapped`] methods to visit arrays and objects while
	/// keeping track of the code map offset of each visited item.
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

/// Unexpected JSON value kind error.
///
/// This error may be returned by [`TryFromJson`] and [`TryFromJsonObject`]
/// when trying to convert a value of the wrong [`Kind`].
#[derive(Debug)]
pub struct Unexpected {
	/// Expected kind(s).
	pub expected: KindSet,

	/// Found kind.
	pub found: Kind,
}

impl fmt::Display for Unexpected {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"expected {}, found {}",
			self.expected.as_disjunction(),
			self.found
		)
	}
}

impl TryFromJson for () {
	type Error = Mapped<Unexpected>;

	fn try_from_json_at(
		json: Value,
		_code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Null => Ok(()),
			other => Err(Mapped(
				Unexpected {
					expected: KindSet::NULL,
					found: other.kind(),
				},
				offset,
			)),
		}
	}
}

impl TryFromJson for bool {
	type Error = Mapped<Unexpected>;

	fn try_from_json_at(
		json: Value,
		_code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Boolean(value) => Ok(value),
			other => Err(Mapped(
				Unexpected {
					expected: KindSet::BOOLEAN,
					found: other.kind(),
				},
				offset,
			)),
		}
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

macro_rules! number_from_json {
	($($ty:ident),*) => {
		$(
			impl TryFromJson for $ty {
				type Error = Mapped<TryIntoNumberError<NumberType<$ty>>>;

				fn try_from_json_at(json: Value, _code_map: &CodeMap, offset: usize) -> Result<Self, Self::Error> {
					match json {
						Value::Number(value) => value.parse().map_err(|_| Mapped(TryIntoNumberError::OutOfBounds(NumberType::default()), offset)),
						other => Err(Mapped(TryIntoNumberError::Unexpected(Unexpected {
							expected: KindSet::NUMBER,
							found: other.kind()
						}), offset))
					}
				}
			}
		)*
	};
}

number_from_json!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

impl TryFromJson for String {
	type Error = Mapped<Unexpected>;

	fn try_from_json_at(
		json: Value,
		_code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::String(value) => Ok(value.to_string()),
			other => Err(Mapped(
				Unexpected {
					expected: KindSet::STRING,
					found: other.kind(),
				},
				offset,
			)),
		}
	}
}

impl<T: TryFromJson> TryFromJson for Vec<T>
where
	T::Error: From<Mapped<Unexpected>>,
{
	type Error = T::Error;

	fn try_from_json_at(
		json: Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Array(value) => value
				.into_iter_mapped(code_map, offset)
				.map(|item| T::try_from_json_at(item.0, code_map, item.1))
				.collect::<Result<Vec<_>, _>>(),
			other => Err(Mapped(
				Unexpected {
					expected: KindSet::ARRAY,
					found: other.kind(),
				},
				offset,
			)
			.into()),
		}
	}
}

impl<K: FromStr + Ord, V: TryFromJson> TryFromJson for BTreeMap<K, V>
where
	V::Error: From<Mapped<Unexpected>> + From<Mapped<K::Err>>,
{
	type Error = V::Error;

	fn try_from_json_at(
		json: Value,
		code_map: &CodeMap,
		offset: usize,
	) -> Result<Self, Self::Error> {
		match json {
			Value::Object(object) => {
				let mut result = BTreeMap::new();

				for entry in object.into_iter_mapped(code_map, offset) {
					result.insert(
						entry
							.0
							 .0
							 .0
							.parse()
							.map_err(|e| Mapped(e, entry.0 .0 .1))?,
						V::try_from_json_at(entry.0 .1 .0, code_map, entry.0 .1 .1)?,
					);
				}

				Ok(result)
			}
			other => Err(Mapped(
				Unexpected {
					expected: KindSet::OBJECT,
					found: other.kind(),
				},
				offset,
			)
			.into()),
		}
	}
}
