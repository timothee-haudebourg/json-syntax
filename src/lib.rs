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
//! use json_syntax::{Value, Parse, Print};
//!
//! let filename = "tests/inputs/y_structure_500_nested_arrays.json";
//! let input = fs::read_to_string(filename).unwrap();
//! let mut value = Value::parse_str(&input).expect("parse error").0;
//! println!("value: {}", value.pretty_print());
//! ```
pub use json_number::{InvalidNumber, Number};
use smallvec::SmallVec;
use std::{fmt, str::FromStr};

pub mod array;
pub mod code_map;
pub mod object;
pub mod parse;
mod unordered;
pub use code_map::CodeMap;
pub use parse::Parse;
pub mod print;
pub use print::Print;
pub mod kind;
pub use kind::{Kind, KindSet};
mod convert;
mod macros;
mod try_from;
pub use try_from::*;

pub mod number {
	pub use json_number::Buffer;
}

#[cfg(feature = "serde")]
mod serde;

#[cfg(feature = "serde")]
pub use self::serde::*;

pub use unordered::*;

/// String stack capacity.
///
/// If a string is longer than this value,
/// it will be stored on the heap.
pub const SMALL_STRING_CAPACITY: usize = 16;

/// String.
pub type String = smallstr::SmallString<[u8; SMALL_STRING_CAPACITY]>;

pub use array::Array;

pub use object::Object;

/// Number buffer stack capacity.
///
/// If the number is longer than this value,
/// it will be stored on the heap.
pub const NUMBER_CAPACITY: usize = SMALL_STRING_CAPACITY;

/// Number buffer.
pub type NumberBuf = json_number::SmallNumberBuf<NUMBER_CAPACITY>;

/// JSON Value.
///
/// # Parsing
///
/// You can parse a `Value` by importing the [`Parse`] trait providing a
/// collection of parsing functions.
///
/// ## Example
///
/// ```
/// use json_syntax::{Value, Parse, CodeMap};
/// let (value, code_map) = Value::parse_str("{ \"key\": \"value\" }").unwrap();
/// ```
///
/// The `code_map` value of type [`CodeMap`] contains code-mapping information
/// about all the fragments of the JSON value (their location in the source
/// text).
///
/// # Comparison
///
/// This type implements the usual comparison traits `PartialEq`, `Eq`,
/// `PartialOrd` and `Ord`. However by default JSON object entries ordering
/// matters, meaning that `{ "a": 0, "b": 1 }` is **not** equal to
/// `{ "b": 1, "a": 0 }`.
/// If you want to do comparisons while ignoring entries ordering, you can use
/// the [`Unordered`] type (combined with the [`UnorderedPartialEq`] trait).
/// Any `T` reference can be turned into an [`Unordered<T>`] reference
/// at will using the [`BorrowUnordered::as_unordered`] method.
///
/// ## Example
///
/// ```
/// use json_syntax::{json, Unordered, BorrowUnordered};
///
/// let a = json!({ "a": 0, "b": 1 });
/// let b = json!({ "b": 1, "a": 0 });
///
/// assert_ne!(a, b); // not equals entries are in a different order.
/// assert_eq!(a.as_unordered(), b.as_unordered()); // equals modulo entry order.
/// assert_eq!(Unordered(a), Unordered(b)); // equals modulo entry order.
/// ```
///
/// # Printing
///
/// The [`Print`] trait provide a highly configurable printing method.
///
/// ## Example
///
/// ```
/// use json_syntax::{Value, Parse, Print};
///
/// let value = Value::parse_str("[ 0, 1, { \"key\": \"value\" }, null ]").unwrap().0;
///
/// println!("{}", value.pretty_print()); // multi line, indent with 2 spaces
/// println!("{}", value.inline_print()); // single line, spaces
/// println!("{}", value.compact_print()); // single line, no spaces
///
/// let mut options = json_syntax::print::Options::pretty();
/// options.indent = json_syntax::print::Indent::Tabs(1);
/// println!("{}", value.print_with(options)); // multi line, indent with tabs
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Value {
	/// `null`.
	Null,

	/// Boolean `true` or `false`.
	Boolean(bool),

	/// Number.
	Number(NumberBuf),

	/// String.
	String(String),

	/// Array.
	Array(Array),

	/// Object.
	Object(Object),
}

pub fn get_array_fragment(array: &[Value], mut index: usize) -> Result<FragmentRef, usize> {
	for v in array {
		match v.get_fragment(index) {
			Ok(value) => return Ok(value),
			Err(i) => index = i,
		}
	}

	Err(index)
}

impl Value {
	pub fn get_fragment(&self, index: usize) -> Result<FragmentRef, usize> {
		if index == 0 {
			Ok(FragmentRef::Value(self))
		} else {
			match self {
				Self::Array(a) => get_array_fragment(a, index - 1),
				Self::Object(o) => o.get_fragment(index - 1),
				_ => Err(index - 1),
			}
		}
	}

	#[inline]
	pub fn kind(&self) -> Kind {
		match self {
			Self::Null => Kind::Null,
			Self::Boolean(_) => Kind::Boolean,
			Self::Number(_) => Kind::Number,
			Self::String(_) => Kind::String,
			Self::Array(_) => Kind::Array,
			Self::Object(_) => Kind::Object,
		}
	}

	#[inline]
	pub fn is_kind(&self, kind: Kind) -> bool {
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

	/// Alias for [`as_string`](Self::as_string).
	#[inline]
	pub fn as_str(&self) -> Option<&str> {
		self.as_string()
	}

	#[inline]
	pub fn as_string_mut(&mut self) -> Option<&mut String> {
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
	pub fn as_array_mut(&mut self) -> Option<&mut Array> {
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
	pub fn as_object(&self) -> Option<&Object> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut Object> {
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
	pub fn into_array(self) -> Option<Array> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<Object> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	pub fn traverse(&self) -> Traverse {
		let mut stack = SmallVec::new();
		stack.push(FragmentRef::Value(self));
		Traverse { offset: 0, stack }
	}

	/// Recursively count the number of values for which `f` returns `true`.
	pub fn count(&self, mut f: impl FnMut(usize, FragmentRef) -> bool) -> usize {
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
			Self::Number(n) => *n = NumberBuf::from_number(n.canonical_with(buffer)),
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
}

impl UnorderedPartialEq for Value {
	fn unordered_eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Null, Self::Null) => true,
			(Self::Boolean(a), Self::Boolean(b)) => a == b,
			(Self::Number(a), Self::Number(b)) => a == b,
			(Self::String(a), Self::String(b)) => a == b,
			(Self::Array(a), Self::Array(b)) => a.unordered_eq(b),
			(Self::Object(a), Self::Object(b)) => a.unordered_eq(b),
			_ => false,
		}
	}
}

impl UnorderedEq for Value {}

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.compact_print().fmt(f)
	}
}

impl From<Value> for ::std::string::String {
	fn from(value: Value) -> Self {
		value.to_string()
	}
}

impl From<bool> for Value {
	fn from(b: bool) -> Self {
		Self::Boolean(b)
	}
}

impl From<NumberBuf> for Value {
	fn from(n: NumberBuf) -> Self {
		Self::Number(n)
	}
}

impl<'n> From<&'n Number> for Value {
	fn from(n: &'n Number) -> Self {
		Self::Number(unsafe { NumberBuf::new_unchecked(n.as_bytes().into()) })
	}
}

impl From<String> for Value {
	fn from(s: String) -> Self {
		Self::String(s)
	}
}

impl From<::std::string::String> for Value {
	fn from(s: ::std::string::String) -> Self {
		Self::String(s.into())
	}
}

impl<'s> From<&'s str> for Value {
	fn from(s: &'s str) -> Self {
		Self::String(s.into())
	}
}

impl From<Array> for Value {
	fn from(a: Array) -> Self {
		Self::Array(a)
	}
}

impl From<Object> for Value {
	fn from(o: Object) -> Self {
		Self::Object(o)
	}
}

impl FromStr for Value {
	type Err = parse::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(Self::parse_str(s)?.0)
	}
}

macro_rules! from_integer {
	($($ty:ident),*) => {
		$(
			impl From<$ty> for Value {
				fn from(n: $ty) -> Self {
					Value::Number(n.into())
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
			impl TryFrom<$ty> for Value {
				type Error = json_number::TryFromFloatError;

				fn try_from(n: $ty) -> Result<Self, Self::Error> {
					Ok(Value::Number(n.try_into()?))
				}
			}
		)*
	};
}

try_from_float! {
	f32,
	f64
}

pub enum FragmentRef<'a> {
	Value(&'a Value),
	Entry(&'a object::Entry),
	Key(&'a object::Key),
}

impl<'a> FragmentRef<'a> {
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
		matches!(self, Self::Value(Value::Null))
	}

	pub fn is_number(&self) -> bool {
		matches!(self, Self::Value(Value::Number(_)))
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Self::Value(Value::String(_)))
	}

	pub fn is_array(&self) -> bool {
		matches!(self, Self::Value(Value::Array(_)))
	}

	pub fn is_object(&self) -> bool {
		matches!(self, Self::Value(Value::Object(_)))
	}

	pub fn strip(self) -> FragmentRef<'a> {
		match self {
			Self::Value(v) => FragmentRef::Value(v),
			Self::Entry(e) => FragmentRef::Entry(e),
			Self::Key(k) => FragmentRef::Key(k),
		}
	}
}

impl<'a> Clone for FragmentRef<'a> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<'a> Copy for FragmentRef<'a> {}

impl<'a> FragmentRef<'a> {
	pub fn sub_fragments(&self) -> SubFragments<'a> {
		match self {
			Self::Value(Value::Array(a)) => SubFragments::Array(a.iter()),
			Self::Value(Value::Object(o)) => SubFragments::Object(o.iter()),
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a> {
	None,
	Array(core::slice::Iter<'a, Value>),
	Object(core::slice::Iter<'a, object::Entry>),
	Entry(Option<&'a object::Key>, Option<&'a Value>),
}

impl<'a> Iterator for SubFragments<'a> {
	type Item = FragmentRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Array(a) => a.next().map(FragmentRef::Value),
			Self::Object(e) => e.next().map(FragmentRef::Entry),
			Self::Entry(k, v) => k
				.take()
				.map(FragmentRef::Key)
				.or_else(|| v.take().map(FragmentRef::Value)),
		}
	}
}

impl<'a> DoubleEndedIterator for SubFragments<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		match self {
			Self::None => None,
			Self::Array(a) => a.next_back().map(FragmentRef::Value),
			Self::Object(e) => e.next_back().map(FragmentRef::Entry),
			Self::Entry(k, v) => v
				.take()
				.map(FragmentRef::Value)
				.or_else(|| k.take().map(FragmentRef::Key)),
		}
	}
}

pub struct Traverse<'a> {
	offset: usize,
	stack: SmallVec<[FragmentRef<'a>; 8]>,
}

impl<'a> Iterator for Traverse<'a> {
	type Item = (usize, FragmentRef<'a>);

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
		let mut value: Value = json!({
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
		let (mut value, _) = Value::parse_str(
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
