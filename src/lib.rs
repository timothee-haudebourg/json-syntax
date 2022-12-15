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
//! - Highly configurable printing methods.
//! - Macro to build any value statically.
//! - JSON Canonicalization Scheme implementation ([RFC 8785](https://www.rfc-editor.org/rfc/rfc8785))
//!   enabled with the `canonicalization` feature.
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
//! let mut value = Value::parse_str(&input, |span| span).expect("parse error");
//! println!("value: {}", value.pretty_print());
//! ```
pub use json_number::Number;
use locspan::Meta;
use locspan_derive::*;
use smallvec::SmallVec;
use std::fmt;

pub mod object;
pub mod parse;
mod unordered;
pub use parse::Parse;
pub mod print;
pub use print::Print;
mod macros;

pub use unordered::*;

/// Value wrapped inside a [`locspan::Meta`] container.
///
/// This type alias is useful if the same metadata is used for the top level
/// value and the sub value to avoid repetition of the `M` type.
pub type MetaValue<M> = Meta<Value<M>, M>;

/// String stack capacity.
///
/// If a string is longer than this value,
/// it will be stored on the heap.
pub const SMALL_STRING_CAPACITY: usize = 16;

/// String.
pub type String = smallstr::SmallString<[u8; SMALL_STRING_CAPACITY]>;

/// Array.
pub type Array<M> = Vec<Meta<Value<M>, M>>;

pub use object::Object;

/// Number buffer stack capacity.
///
/// If the number is longer than this value,
/// it will be stored on the heap.
pub const NUMBER_CAPACITY: usize = SMALL_STRING_CAPACITY;

/// Number buffer.
pub type NumberBuf = json_number::SmallNumberBuf<NUMBER_CAPACITY>;

/// Value kind.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Kind {
	Null,
	Boolean,
	Number,
	String,
	Array,
	Object,
}

impl fmt::Display for Kind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Boolean => write!(f, "boolean"),
			Self::Number => write!(f, "number"),
			Self::String => write!(f, "string"),
			Self::Array => write!(f, "array"),
			Self::Object => write!(f, "object"),
		}
	}
}

/// Value.
///
/// The type parameter `M` is the type of metadata attached to each syntax node
/// (values, sub values and object keys).
/// The metadata is derived from the [`locspan::Span`] of the node in the source
/// document using the metadata builder function passed to the parsing function
/// (see the [`Parse`] trait for more details).
///
/// # Parsing
///
/// You can parse a `Value` by importing the [`Parse`] trait providing a
/// collection of parsing functions.
///
/// ## Example
///
/// ```
/// use json_syntax::{Value, Parse};
/// use locspan::{Meta, Span};
///
/// let value: Meta<Value<Span>, Span> = Value::parse_str("{ \"key\": \"value\" }", |span| span).unwrap();
/// ```
///
/// You don't need to specify the return type, here only shown to make it clear.
/// Furthermore the `MetaValue<Span>` type alias can be used instead of
/// `Meta<Value<Span>, Span>` to avoid repetition of the metadata type.
///
/// # Comparison
///
/// This type implements the usual comparison traits `PartialEq`, `Eq`,
/// `PartialOrd` and `Ord`. However these implementations will also compare the
/// metadata.
/// If you want to do comparisons while ignoring ths metadata, you can use
/// the [`locspan::Stripped`] type
/// (combined with the [`locspan::BorrowStripped`] trait).
///
/// ## Example
///
/// ```
/// use json_syntax::{Value, Parse};
/// use locspan::BorrowStripped;
///
/// let a = Value::parse_str("null", |_| 0).unwrap();
/// let b = Value::parse_str("null", |_| 1).unwrap();
///
/// assert_ne!(a, b); // not equals because the metadata is different.
/// assert_eq!(a.stripped(), b.stripped()); // equals because the metadata is ignored.
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
/// let value = Value::parse_str("[ 0, 1, { \"key\": \"value\" }, null ]", |span| span).unwrap();
///
/// println!("{}", value.pretty_print()); // multi line, indent with 2 spaces
/// println!("{}", value.inline_print()); // single line, spaces
/// println!("{}", value.compact_print()); // single line, no spaces
///
/// let mut options = json_syntax::print::Options::pretty();
/// options.indent = json_syntax::print::Indent::Tabs(1);
/// println!("{}", value.print_with(options)); // multi line, indent with tabs
/// ```
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
#[locspan(ignore(M))]
pub enum Value<M> {
	/// `null`.
	Null,

	/// Boolean `true` or `false`.
	Boolean(#[locspan(stripped)] bool),

	/// Number.
	Number(#[locspan(stripped)] NumberBuf),

	/// String.
	String(#[locspan(stripped)] String),

	/// Array.
	Array(Array<M>),

	/// Object.
	Object(Object<M>),
}

impl<M> Value<M> {
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
	pub fn as_array(&self) -> Option<&[Meta<Self, M>]> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn as_array_mut(&mut self) -> Option<&mut Array<M>> {
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
	pub fn force_as_array(value: &Meta<Self, M>) -> Meta<&[Meta<Self, M>], &M> {
		match value {
			Meta(Self::Array(a), meta) => Meta(a, meta),
			other @ Meta(_, meta) => Meta(core::slice::from_ref(other), meta),
		}
	}

	#[inline]
	pub fn as_object(&self) -> Option<&Object<M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	#[inline]
	pub fn as_object_mut(&mut self) -> Option<&mut Object<M>> {
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
	pub fn into_array(self) -> Option<Array<M>> {
		match self {
			Self::Array(a) => Some(a),
			_ => None,
		}
	}

	#[inline]
	pub fn into_object(self) -> Option<Object<M>> {
		match self {
			Self::Object(o) => Some(o),
			_ => None,
		}
	}

	pub fn traverse(&self) -> TraverseStripped<M> {
		let mut stack = SmallVec::new();
		stack.push(StrippedFragmentRef::Value(self));
		TraverseStripped { stack }
	}

	/// Recursively count the number of values for which `f` returns `true`.
	pub fn count(&self, mut f: impl FnMut(StrippedFragmentRef<M>) -> bool) -> usize {
		self.traverse().filter(|i| f(*i)).count()
	}

	/// Returns the volume of the value.
	///
	/// The volume is the sum of all values and recursively nested values
	/// included in `self`, including `self` (the volume is at least `1`).
	///
	/// This is equivalent to `value.traverse().filter(StrippedFragmentRef::is_value).count()`.
	pub fn volume(&self) -> usize {
		self.traverse()
			.filter(StrippedFragmentRef::is_value)
			.count()
	}

	/// Recursively maps the metadata inside the value.
	pub fn map_metadata<N>(self, mut f: impl FnMut(M) -> N) -> Value<N> {
		match self {
			Self::Null => Value::Null,
			Self::Boolean(b) => Value::Boolean(b),
			Self::Number(n) => Value::Number(n),
			Self::String(s) => Value::String(s),
			Self::Array(a) => Value::Array(
				a.into_iter()
					.map(|Meta(item, meta)| Meta(item.map_metadata(&mut f), f(meta)))
					.collect(),
			),
			Self::Object(o) => Value::Object(o.map_metadata(f)),
		}
	}

	/// Tries to recursively maps the metadata inside the value.
	pub fn try_map_metadata<N, E>(
		self,
		mut f: impl FnMut(M) -> Result<N, E>,
	) -> Result<Value<N>, E> {
		match self {
			Self::Null => Ok(Value::Null),
			Self::Boolean(b) => Ok(Value::Boolean(b)),
			Self::Number(n) => Ok(Value::Number(n)),
			Self::String(s) => Ok(Value::String(s)),
			Self::Array(a) => {
				let mut items = Vec::with_capacity(a.len());
				for item in a {
					items.push(item.try_map_metadata_recursively(&mut f)?)
				}
				Ok(Value::Array(items))
			}
			Self::Object(o) => Ok(Value::Object(o.try_map_metadata(f)?)),
		}
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

pub trait Traversal<'a> {
	type Fragment;
	type Traverse: Iterator<Item = Self::Fragment>;

	fn traverse(&'a self) -> Self::Traverse;
}

impl<'a, M: 'a> Traversal<'a> for Meta<Value<M>, M> {
	type Fragment = FragmentRef<'a, M>;
	type Traverse = Traverse<'a, M>;

	fn traverse(&'a self) -> Self::Traverse {
		let mut stack = SmallVec::new();
		stack.push(FragmentRef::Value(self));
		Traverse { stack }
	}
}

impl<M, N> locspan::MapMetadataRecursively<M, N> for Value<M> {
	type Output = Value<N>;

	fn map_metadata_recursively<F: FnMut(M) -> N>(self, f: F) -> Value<N> {
		self.map_metadata(f)
	}
}

impl<M, N, E> locspan::TryMapMetadataRecursively<M, N, E> for Value<M> {
	type Output = Value<N>;

	fn try_map_metadata_recursively<F: FnMut(M) -> Result<N, E>>(
		self,
		f: F,
	) -> Result<Value<N>, E> {
		self.try_map_metadata(f)
	}
}

impl<M> From<bool> for Value<M> {
	fn from(b: bool) -> Self {
		Self::Boolean(b)
	}
}

impl<M> From<NumberBuf> for Value<M> {
	fn from(n: NumberBuf) -> Self {
		Self::Number(n)
	}
}

impl<'n, M> From<&'n Number> for Value<M> {
	fn from(n: &'n Number) -> Self {
		Self::Number(unsafe { NumberBuf::new_unchecked(n.as_bytes().into()) })
	}
}

impl<M> From<String> for Value<M> {
	fn from(s: String) -> Self {
		Self::String(s)
	}
}

impl<M> From<::std::string::String> for Value<M> {
	fn from(s: ::std::string::String) -> Self {
		Self::String(s.into())
	}
}

impl<'s, M> From<&'s str> for Value<M> {
	fn from(s: &'s str) -> Self {
		Self::String(s.into())
	}
}

impl<M> From<Array<M>> for Value<M> {
	fn from(a: Array<M>) -> Self {
		Self::Array(a)
	}
}

impl<M> From<Object<M>> for Value<M> {
	fn from(o: Object<M>) -> Self {
		Self::Object(o)
	}
}

macro_rules! from_integer {
	($($ty:ident),*) => {
		$(
			impl<M> From<$ty> for Value<M> {
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
			impl<M> TryFrom<$ty> for Value<M> {
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

pub enum StrippedFragmentRef<'a, M> {
	Value(&'a Value<M>),
	Entry(&'a object::Entry<M>),
	Key(&'a object::Key),
}

impl<'a, M> StrippedFragmentRef<'a, M> {
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
}

impl<'a, M> Clone for StrippedFragmentRef<'a, M> {
	fn clone(&self) -> Self {
		match self {
			Self::Value(v) => Self::Value(*v),
			Self::Entry(e) => Self::Entry(e),
			Self::Key(k) => Self::Key(k),
		}
	}
}

impl<'a, M> Copy for StrippedFragmentRef<'a, M> {}

impl<'a, M> StrippedFragmentRef<'a, M> {
	pub fn sub_fragments(&self) -> SubFragments<'a, M> {
		match self {
			Self::Value(Value::Array(a)) => SubFragments::Array(a.iter()),
			Self::Value(Value::Object(o)) => SubFragments::Object(o.iter()),
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			_ => SubFragments::None,
		}
	}
}

pub enum FragmentRef<'a, M> {
	Value(&'a Meta<Value<M>, M>),
	Entry(&'a object::Entry<M>),
	Key(&'a Meta<object::Key, M>),
}

impl<'a, M> FragmentRef<'a, M> {
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
		matches!(self, Self::Value(Meta(Value::Null, _)))
	}

	pub fn is_number(&self) -> bool {
		matches!(self, Self::Value(Meta(Value::Number(_), _)))
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Self::Value(Meta(Value::String(_), _)))
	}

	pub fn is_array(&self) -> bool {
		matches!(self, Self::Value(Meta(Value::Array(_), _)))
	}

	pub fn is_object(&self) -> bool {
		matches!(self, Self::Value(Meta(Value::Object(_), _)))
	}

	pub fn strip(self) -> StrippedFragmentRef<'a, M> {
		match self {
			Self::Value(v) => StrippedFragmentRef::Value(v.value()),
			Self::Entry(e) => StrippedFragmentRef::Entry(e),
			Self::Key(k) => StrippedFragmentRef::Key(k.value()),
		}
	}
}

impl<'a, M> locspan::Strip for FragmentRef<'a, M> {
	type Stripped = StrippedFragmentRef<'a, M>;

	fn strip(self) -> Self::Stripped {
		self.strip()
	}
}

impl<'a, M> Clone for FragmentRef<'a, M> {
	fn clone(&self) -> Self {
		match self {
			Self::Value(v) => Self::Value(*v),
			Self::Entry(e) => Self::Entry(e),
			Self::Key(k) => Self::Key(*k),
		}
	}
}

impl<'a, M> Copy for FragmentRef<'a, M> {}

impl<'a, M> FragmentRef<'a, M> {
	pub fn sub_fragments(&self) -> SubFragments<'a, M> {
		match self {
			Self::Value(Meta(Value::Array(a), _)) => SubFragments::Array(a.iter()),
			Self::Value(Meta(Value::Object(o), _)) => SubFragments::Object(o.iter()),
			Self::Entry(e) => SubFragments::Entry(Some(&e.key), Some(&e.value)),
			_ => SubFragments::None,
		}
	}
}

pub enum SubFragments<'a, M> {
	None,
	Array(core::slice::Iter<'a, Meta<Value<M>, M>>),
	Object(core::slice::Iter<'a, object::Entry<M>>),
	Entry(
		Option<&'a Meta<object::Key, M>>,
		Option<&'a Meta<Value<M>, M>>,
	),
}

impl<'a, M> Iterator for SubFragments<'a, M> {
	type Item = FragmentRef<'a, M>;

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

pub struct TraverseStripped<'a, M> {
	stack: SmallVec<[StrippedFragmentRef<'a, M>; 8]>,
}

impl<'a, M> Iterator for TraverseStripped<'a, M> {
	type Item = StrippedFragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(v) => {
				self.stack.extend(v.sub_fragments().map(FragmentRef::strip));
				Some(v)
			}
			None => None,
		}
	}
}

pub struct Traverse<'a, M> {
	stack: SmallVec<[FragmentRef<'a, M>; 8]>,
}

impl<'a, M> Iterator for Traverse<'a, M> {
	type Item = FragmentRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stack.pop() {
			Some(v) => {
				self.stack.extend(v.sub_fragments());
				Some(v)
			}
			None => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[cfg(feature = "canonicalize")]
	fn canonicalize_01() {
		let mut value: Meta<Value<()>, ()> = json!({
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
}
