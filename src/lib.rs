//! This library provides a strict JSON parser as defined by
//! [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) and
//! [ECMA-404](https://www.ecma-international.org/publications-and-standards/standards/ecma-404/).
//! It is built on the [`locspan`](https://crates.io/crates/locspan) library
//! so as to keep track of the position of each JSON value in the parsed
//! document.
//!
//! # Features
//!
//! - Strict implementation of [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) and
//!   [ECMA-404](https://www.ecma-international.org/publications-and-standards/standards/ecma-404/).
//! - No stack overflow, your memory is the limit.
//! - Numbers are stored in lexical form thanks to the [`json-number`](https://crates.io/crates/json-number) crate,
//!   their precision is not limited.
//! - Duplicate values are preserved. A JSON object is just a list of entries,
//!   in the order of definition.
//! - Strings are stored on the stack whenever possible, thanks to the [`smallstr`](https://crates.io/crates/smallstr) crate.
//! - The parser is configurable to accept documents that do not strictly
//!   adhere to the standard.
//! - Thoroughly tested.
//!
//! # Usage
//!
//! ```
//! use std::fs;
//! use json_syntax::{Value, Parse};
//! use decoded_char::DecodedChars;
//! use locspan::Loc;
//!
//! fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
//!   Ok(t)
//! }
//!
//! let filename = "tests/inputs/y_structure_500_nested_arrays.json";
//! let input = fs::read_to_string(filename).unwrap();
//! let Loc(value, value_location) = Value::parse(filename, input.decoded_chars().map(infallible)).expect("parse error");
//!
//! // ...
//! ```
pub use json_number::Number;
use locspan::Loc;
use locspan_derive::*;

pub mod parse;
pub use parse::Parse;

/// String stack capacity.
///
/// If a string is longer than this value,
/// it will be stored on the heap.
const SMALL_STRING_CAPACITY: usize = 16;

/// String.
pub type String = smallstr::SmallString<[u8; SMALL_STRING_CAPACITY]>;

/// Array.
pub type Array<S, P = locspan::Span> = Vec<Loc<Value<S, P>, S, P>>;

/// Object entry.
#[derive(
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(S, P)]
pub struct Entry<S, P = locspan::Span> {
	#[stripped_deref]
	pub key: Loc<Key, S, P>,
	pub value: Loc<Value<S, P>, S, P>,
}

impl<S, P> Entry<S, P> {
	pub fn new(key: Loc<Key, S, P>, value: Loc<Value<S, P>, S, P>) -> Self {
		Self { key, value }
	}
}

/// Object key stack capacity.
///
/// If the key is longer than this value,
/// it will be stored on the heap.
const KEY_CAPACITY: usize = 16;

/// Object key.
pub type Key = smallstr::SmallString<[u8; KEY_CAPACITY]>;

/// Object.
pub type Object<S, P = locspan::Span> = Vec<Entry<S, P>>;

/// Number buffer stack capacity.
///
/// If the number is longer than this value,
/// it will be stored on the heap.
const NUMBER_CAPACITY: usize = SMALL_STRING_CAPACITY;

/// Number buffer.
pub type NumberBuf = json_number::SmallNumberBuf<NUMBER_CAPACITY>;

/// Value.
///
/// The two types parameters are used to locate/map values inside the source
/// file.
/// The `S` parameter is the type used to identify the source file (generally
/// a string slice, a path, or an index).
/// The `P` parameter is the type used to locate the value *inside* the file.
/// By default the `locspan::Span` type is used, since it is what the parser
/// uses.
///
/// # Comparison
///
/// This type implements the usual comparison traits `PartialEq`, `Eq`,
/// `PartialOrd` and `Ord`. However these implementations will also compare the
/// code mapping information (source file and span).
/// If you want to do comparisons while ignoring this information, you can use
/// the [`locspan::Stripped`] type.
#[derive(
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
#[stripped_ignore(S, P)]
pub enum Value<S, P = locspan::Span> {
	/// `null`.
	Null,

	/// Boolean `true` or `false`.
	Boolean(#[stripped] bool),

	/// Number.
	Number(#[stripped] NumberBuf),

	/// String.
	String(#[stripped] String),

	/// Array.
	Array(Array<S, P>),

	/// Object.
	Object(Object<S, P>),
}

impl<S, P> Value<S, P> {
	#[inline]
	pub fn is_null(&self) -> bool {
		matches!(self, Self::Null)
	}

	#[inline]
	pub fn is_boolean(&self) -> bool {
		matches!(self, Self::Boolean(_))
	}

	#[inline]
	pub fn is_number(&self) -> bool {
		matches!(self, Self::Number(_))
	}

	#[inline]
	pub fn is_string(&self) -> bool {
		matches!(self, Self::String(_))
	}

	#[inline]
	pub fn is_array(&self) -> bool {
		matches!(self, Self::Array(_))
	}

	#[inline]
	pub fn is_object(&self) -> bool {
		matches!(self, Self::Object(_))
	}

	#[inline]
	pub fn as_boolean(&self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	#[inline]
	pub fn as_boolean_mut(&mut self) -> Option<&mut bool> {
		match self {
			Self::Boolean(b) => Some(b),
			_ => None,
		}
	}

	#[inline]
	pub fn as_number(&self) -> Option<&Number> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn as_number_mut(&mut self) -> Option<&mut NumberBuf> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn as_string(&self) -> Option<&str> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_string_mut(&mut self) -> Option<&mut String> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_array(&self) -> Option<&[Loc<Self, S, P>]> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_array_mut(&mut self) -> Option<&mut Array<S, P>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object(&self) -> Option<&Object<S, P>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut Object<S, P>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn into_boolean(self) -> Option<bool> {
		match self {
			Self::Boolean(b) => Some(b),
			_ => None,
		}
	}

	#[inline]
	pub fn into_number(self) -> Option<NumberBuf> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn into_string(self) -> Option<String> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn into_array(self) -> Option<Array<S, P>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<Object<S, P>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}
}
