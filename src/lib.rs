//! This library provides a strict JSON parser as defined by
//! [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) and
//! [ECMA-404](https://www.ecma-international.org/publications-and-standards/standards/ecma-404/).
//! Parsing values generates a [`CodeMap`] that keeps track of the position of
//! each JSON value fragment in the parsed document.
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
//! - Highly configurable printing methods.
//! - Macro to build any value statically.
//! - JSON Canonicalization Scheme implementation ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785))
//!   enabled with the `canonicalization` feature.
//! - `serde` support (by enabling the `serde` feature).
//! - Conversion from/to `serde_json::Value` (by enabling the `serde_json` feature).
//! - Thoroughly tested.
//!
//! # Usage
//!
//! ```
//! use std::fs;
//! use json_syntax::{JsonValue, JsonParse, JsonPrint};
//!
//! let filename = "tests/inputs/y_structure_500_nested_arrays.json";
//! let input = fs::read_to_string(filename).unwrap();
//! let mut value = JsonValue::parse_str(&input).expect("parse error").0;
//! println!("value: {}", value.pretty_print());
//! ```
use smallvec::SmallVec;
use std::{fmt, str::FromStr};

pub use locspan;

#[cfg(feature = "canonicalize")]
pub use ryu_js;

pub mod array;
pub mod code_map;
mod convert;
pub mod kind;
pub mod lexical;
mod macros;
pub mod number;
pub mod object;
pub mod parse;
pub mod print;
pub mod string;
// pub mod tracing;
pub mod visitor;

pub use array::{JsonArray, JsonArrayBuf};
pub use code_map::JsonCodeMap;
pub use kind::{JsonKind, JsonKindSet};
use lexical::{JsonBorrowLexical, JsonLexicalEq, JsonLexicalPartialEq};
pub use number::{InvalidJsonNumber, JsonNumber, JsonNumberBuf};
pub use object::JsonObject;
pub use parse::JsonParse;
pub use print::JsonPrint;
pub use string::JsonString;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "serde")]
pub use serde::{from_slice, from_str, from_value, to_value};

use crate::array::JsonArrayExt;

/// JSON Value.
///
/// # Parsing
///
/// You can parse a `Value` by importing the [`JsonParse`] trait providing a
/// collection of parsing functions.
///
/// ## Example
///
/// ```
/// use json_syntax::{JsonValue, JsonParse, JsonCodeMap};
/// let (value, code_map) = JsonValue::parse_str("{ \"key\": \"value\" }").unwrap();
/// ```
///
/// The `code_map` value of type [`JsonCodeMap`] contains code-mapping information
/// about all the fragments of the JSON value (their location in the source
/// text).
///
/// # Comparison
///
/// The `PartialEq` and `PartialOrd` implementations will compare the JSON
/// values semantically, without regard for the object entries indexes. The only
/// exception is when one key has multiple values, in which cases the values are
/// ordered by index. It is possible to compare the lexical representation of
/// objects (where the entries indexes matter) by using the
/// [`JsonBorrowLexical::as_lexical`] method on both ends and comparing the
/// results.
///
/// ## Example
///
/// ```
/// use json_syntax::{json, lexical::JsonBorrowLexical};
///
/// let a = json!({ "a": 0, "b": 1 });
/// let b = json!({ "b": 1, "a": 0 });
///
/// assert_eq!(a, b); // equals, because semantically equivalent.
/// assert_ne!(a.as_lexical(), b.as_lexical()); // not equals.
/// ```
///
/// # Printing
///
/// The [`JsonPrint`] trait provide a highly configurable printing method.
///
/// ## Example
///
/// ```
/// use json_syntax::{JsonValue, JsonParse, JsonPrint};
///
/// let value = JsonValue::parse_str("[ 0, 1, { \"key\": \"value\" }, null ]").unwrap().0;
///
/// println!("{}", value.pretty_print()); // multi line, indent with 2 spaces
/// println!("{}", value.inline_print()); // single line, spaces
/// println!("{}", value.compact_print()); // single line, no spaces
///
/// let mut options = json_syntax::print::JsonPrintOptions::PRETTY;
/// options.indent = json_syntax::print::Indent::Tabs(1);
/// println!("{}", value.print_with(&options)); // multi line, indent with tabs
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum JsonValue {
	/// `null`.
	Null,

	/// Boolean `true` or `false`.
	Boolean(bool),

	/// Number.
	Number(JsonNumberBuf),

	/// String.
	String(JsonString),

	/// Array.
	Array(JsonArrayBuf),

	/// Object.
	Object(JsonObject),
}

impl JsonValue {
	pub fn get_fragment(&self, index: usize) -> Result<JsonFragment<'_>, usize> {
		if index == 0 {
			Ok(JsonFragment::Value(self))
		} else {
			match self {
				Self::Array(a) => a.get_fragment(index - 1),
				Self::Object(o) => o.get_fragment(index - 1),
				_ => Err(index - 1),
			}
		}
	}

	#[inline]
	pub fn kind(&self) -> JsonKind {
		match self {
			Self::Null => JsonKind::Null,
			Self::Boolean(_) => JsonKind::Boolean,
			Self::Number(_) => JsonKind::Number,
			Self::String(_) => JsonKind::String,
			Self::Array(_) => JsonKind::Array,
			Self::Object(_) => JsonKind::Object,
		}
	}

	#[inline]
	pub fn is_kind(&self, kind: JsonKind) -> bool {
		self.kind() == kind
	}

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

	/// Checks if the value is either an empty array or an empty object.
	#[inline]
	pub fn is_empty_array_or_object(&self) -> bool {
		match self {
			Self::Array(a) => a.is_empty(),
			Self::Object(o) => o.is_empty(),
			_ => false,
		}
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
	pub fn as_number(&self) -> Option<&JsonNumber> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn as_number_mut(&mut self) -> Option<&mut JsonNumberBuf> {
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

	/// Alias for [`as_string`](Self::as_string).
	#[inline]
	pub fn as_str(&self) -> Option<&str> {
		self.as_string()
	}

	#[inline]
	pub fn as_string_mut(&mut self) -> Option<&mut JsonString> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_array(&self) -> Option<&[Self]> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_array_mut(&mut self) -> Option<&mut JsonArrayBuf> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	/// Return the given value as an array, even if it is not an array.
	///
	/// Returns the input value as is if it is already an array,
	/// or puts it in a slice with a single element if it is not.
	#[inline]
	pub fn force_as_array(&self) -> &[Self] {
		match self {
			Self::Array(a) => a,
			other => core::slice::from_ref(other),
		}
	}

	#[inline]
	pub fn as_object(&self) -> Option<&JsonObject> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut JsonObject> {
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
	pub fn into_number(self) -> Option<JsonNumberBuf> {
		match self {
			Self::Number(n) => Some(n),
			_ => None,
		}
	}

	#[inline]
	pub fn into_string(self) -> Option<JsonString> {
		match self {
			Self::String(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn into_array(self) -> Option<JsonArrayBuf> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<JsonObject> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	pub fn traverse(&self) -> Traverse<'_> {
		let mut stack = SmallVec::new();
		stack.push(JsonFragment::Value(self));
		Traverse { offset: 0, stack }
	}

	/// Recursively count the number of values for which `f` returns `true`.
	pub fn count(&self, mut f: impl FnMut(usize, JsonFragment) -> bool) -> usize {
		self.traverse().filter(|(i, q)| f(*i, *q)).count()
	}

	/// Returns the volume of the value.
	///
	/// The volume is the sum of all values and recursively nested values
	/// included in `self`, including `self` (the volume is at least `1`).
	///
	/// This is equivalent to `value.traverse().filter(|(_, f)| f.is_value()).count()`.
	pub fn volume(&self) -> usize {
		self.traverse().filter(|(_, f)| f.is_value()).count()
	}

	/// Move and return the value, leaves `null` in its place.
	#[inline(always)]
	pub fn take(&mut self) -> Self {
		let mut result = Self::Null;
		std::mem::swap(&mut result, self);
		result
	}

	/// Puts this JSON value in canonical form according to
	/// [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785).
	///
	/// The given `buffer` is used to canonicalize the number values.
	#[cfg(feature = "canonicalize")]
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		match self {
			Self::Number(n) => n.canonicalize_with(buffer),
			Self::Array(a) => {
				for item in a {
					item.canonicalize_with(buffer)
				}
			}
			Self::Object(o) => o.canonicalize_with(buffer),
			_ => (),
		}
	}

	/// Puts this JSON value in canonical form according to
	/// [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}

	/// Returns the canonical form of this value according to
	/// [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalized(&self) -> Self {
		let mut result = self.clone();
		result.canonicalize();
		result
	}
}

impl JsonBorrowLexical for JsonValue {}

impl JsonLexicalPartialEq for JsonValue {
	fn lexical_eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Null, Self::Null) => true,
			(Self::Boolean(a), Self::Boolean(b)) => a == b,
			(Self::Number(a), Self::Number(b)) => a == b,
			(Self::String(a), Self::String(b)) => a == b,
			(Self::Array(a), Self::Array(b)) => a.lexical_eq(b),
			(Self::Object(a), Self::Object(b)) => a.lexical_eq(b),
			_ => false,
		}
	}
}

impl JsonLexicalEq for JsonValue {}

impl fmt::Display for JsonValue {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.compact_print().fmt(f)
	}
}

impl From<JsonValue> for ::std::string::String {
	fn from(value: JsonValue) -> Self {
		value.to_string()
	}
}

impl From<bool> for JsonValue {
	fn from(b: bool) -> Self {
		Self::Boolean(b)
	}
}

impl From<JsonNumberBuf> for JsonValue {
	fn from(n: JsonNumberBuf) -> Self {
		Self::Number(n)
	}
}

impl<'n> From<&'n JsonNumber> for JsonValue {
	fn from(n: &'n JsonNumber) -> Self {
		Self::Number(unsafe { JsonNumberBuf::new_unchecked(n.as_bytes().into()) })
	}
}

impl From<JsonString> for JsonValue {
	fn from(s: JsonString) -> Self {
		Self::String(s)
	}
}

impl From<::std::string::String> for JsonValue {
	fn from(s: ::std::string::String) -> Self {
		Self::String(s.into())
	}
}

impl<'s> From<&'s str> for JsonValue {
	fn from(s: &'s str) -> Self {
		Self::String(s.into())
	}
}

impl From<JsonArrayBuf> for JsonValue {
	fn from(a: JsonArrayBuf) -> Self {
		Self::Array(a)
	}
}

impl From<JsonObject> for JsonValue {
	fn from(o: JsonObject) -> Self {
		Self::Object(o)
	}
}

impl FromStr for JsonValue {
	type Err = parse::JsonParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::parse_str(s)?.0)
	}
}

macro_rules! from_integer {
	($($ty:ident),*) => {
		$(
			impl From<$ty> for JsonValue {
				fn from(n: $ty) -> Self {
					JsonValue::Number(n.into())
				}
			}
		)*
	};
}

from_integer! {
	u8,
	u16,
	u32,
	u64,
	i8,
	i16,
	i32,
	i64
}

macro_rules! try_from_float {
	($($ty:ident),*) => {
		$(
			impl TryFrom<$ty> for JsonValue {
				type Error = number::TryFromFloatError;

				fn try_from(n: $ty) -> Result<Self, Self::Error> {
					Ok(JsonValue::Number(n.try_into()?))
				}
			}
		)*
	};
}

try_from_float! {
	f32,
	f64
}

pub enum JsonFragment<'a> {
	Value(&'a JsonValue),
	Entry(object::EntryRef<'a>),
	Key(&'a object::Key),
}

impl<'a> JsonFragment<'a> {
	pub fn is_entry(&self) -> bool {
		matches!(self, Self::Entry(_))
	}

	pub fn is_key(&self) -> bool {
		matches!(self, Self::Key(_))
	}

	pub fn is_value(&self) -> bool {
		matches!(self, Self::Value(_))
	}

	pub fn is_null(&self) -> bool {
		matches!(self, Self::Value(JsonValue::Null))
	}

	pub fn is_number(&self) -> bool {
		matches!(self, Self::Value(JsonValue::Number(_)))
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Self::Value(JsonValue::String(_)))
	}

	pub fn is_array(&self) -> bool {
		matches!(self, Self::Value(JsonValue::Array(_)))
	}

	pub fn is_object(&self) -> bool {
		matches!(self, Self::Value(JsonValue::Object(_)))
	}

	pub fn strip(self) -> JsonFragment<'a> {
		match self {
			Self::Value(v) => JsonFragment::Value(v),
			Self::Entry(e) => JsonFragment::Entry(e),
			Self::Key(k) => JsonFragment::Key(k),
		}
	}
}

impl<'a> Clone for JsonFragment<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for JsonFragment<'a> {}

impl<'a> JsonFragment<'a> {
	pub fn sub_fragments(&self) -> JsonFragments<'a> {
		match self {
			Self::Value(JsonValue::Array(a)) => JsonFragments::Array(a.iter()),
			Self::Value(JsonValue::Object(o)) => JsonFragments::Object(o.iter()),
			Self::Entry((key, value)) => JsonFragments::Entry(Some(key), Some(value)),
			_ => JsonFragments::None,
		}
	}
}

pub enum JsonFragments<'a> {
	None,
	Array(core::slice::Iter<'a, JsonValue>),
	Object(object::Iter<'a>),
	Entry(Option<&'a object::Key>, Option<&'a JsonValue>),
}

impl<'a> Iterator for JsonFragments<'a> {
	type Item = JsonFragment<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Array(a) => a.next().map(JsonFragment::Value),
			Self::Object(e) => e.next().map(JsonFragment::Entry),
			Self::Entry(k, v) => k
				.take()
				.map(JsonFragment::Key)
				.or_else(|| v.take().map(JsonFragment::Value)),
		}
	}
}

impl<'a> DoubleEndedIterator for JsonFragments<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Array(a) => a.next_back().map(JsonFragment::Value),
			Self::Object(e) => e.next_back().map(JsonFragment::Entry),
			Self::Entry(k, v) => v
				.take()
				.map(JsonFragment::Value)
				.or_else(|| k.take().map(JsonFragment::Key)),
		}
	}
}

pub struct Traverse<'a> {
	offset: usize,
	stack: SmallVec<[JsonFragment<'a>; 8]>,
}

impl<'a> Iterator for Traverse<'a> {
	type Item = (usize, JsonFragment<'a>);

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(v) => {
				self.stack.extend(v.sub_fragments().rev());
				let i = self.offset;
				self.offset += 1;
				Some((i, v))
			}
			None => None,
		}
	}
}

#[cfg(test)]
mod tests {
	#[cfg(feature = "canonicalize")]
	#[test]
	fn canonicalize_01() {
		use super::*;
		let mut value: JsonValue = json!({
			"b": 0.00000000001,
			"c": {
				"foo": true,
				"bar": false
			},
			"a": [ "foo", "bar" ]
		});

		value.canonicalize();

		assert_eq!(
			value.compact_print().to_string(),
			"{\"a\":[\"foo\",\"bar\"],\"b\":1e-11,\"c\":{\"bar\":false,\"foo\":true}}"
		)
	}

	#[cfg(feature = "canonicalize")]
	#[test]
	fn canonicalize_02() {
		use super::*;
		let (mut value, _) = JsonValue::parse_str(
			"{
			\"numbers\": [333333333.33333329, 1E30, 4.50, 2e-3, 0.000000000000000000000000001],
			\"string\": \"\\u20ac$\\u000F\\u000aA'\\u0042\\u0022\\u005c\\\\\\\"\\/\",
			\"literals\": [null, true, false]
		}",
		)
		.unwrap();

		value.canonicalize();

		assert_eq!(
			value.compact_print().to_string(),
			"{\"literals\":[null,true,false],\"numbers\":[333333333.3333333,1e+30,4.5,0.002,1e-27],\"string\":\"€$\\u000f\\nA'B\\\"\\\\\\\\\\\"/\"}"
		)
	}
}
