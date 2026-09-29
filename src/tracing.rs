use core::fmt;
use std::error::Error;

use into_owned_trait::IntoOwned;

use crate::{JsonCodeMap, JsonValue, array::JsonArrayExt, code_map::JsonMapped};

impl JsonValue {
	/// Returns the [`JsonCodeMap`] index of the fragment described by `path`.
	///
	/// The returned index can be used to look up the byte span of the fragment
	/// in the `code_map` produced by parsing the same document.
	///
	/// Returns `None` if the path does not exist in this value (e.g., the key
	/// is missing, the array index is out of range, or a path segment tries to
	/// navigate into a fragment that has no sub-fragments).
	///
	/// # Segment semantics
	///
	/// - [`JsonFragmentPathSegment::ArrayIndex`]: navigates into the *i*-th
	///   element of an array.
	/// - [`JsonFragmentPathSegment::ObjectKey`]: navigates to the **key** string
	///   fragment of the *n*-th occurrence of the named key.  This is a terminal
	///   step; further path segments after this will return `None`.
	/// - [`JsonFragmentPathSegment::ObjectValue`]: navigates to the **value**
	///   of the *n*-th occurrence of the named key.
	pub fn locate_fragment(
		&self,
		code_map: &JsonCodeMap,
		path: &JsonFragmentPath,
	) -> Option<usize> {
		self.locate_fragment_at(code_map, path, 0)
	}

	fn locate_fragment_at(
		&self,
		code_map: &JsonCodeMap,
		path: &JsonFragmentPath,
		offset: usize,
	) -> Option<usize> {
		match path.split_first() {
			Some((segment, rest)) => match segment {
				JsonFragmentPathSegment::ArrayItem(i) => {
					let array = self.as_array()?;
					for (j, JsonMapped(item, item_offset)) in
						array.iter_mapped(code_map, offset).enumerate()
					{
						if j == *i {
							return item.locate_fragment_at(code_map, rest, item_offset);
						}
					}

					None
				}
				JsonFragmentPathSegment::ObjectEntry(k, occurrence, part) => {
					let object = self.as_object()?;
					let mut occ_count = 0usize;
					for JsonMapped(
						(JsonMapped(key, key_offset), JsonMapped(value, value_offset)),
						entry_offset,
					) in object.iter_mapped(code_map, offset)
					{
						if k.as_str() == key.as_str() {
							if occ_count == *occurrence {
								return match *part {
									JsonObjectEntryPart::All => {
										if rest.is_empty() {
											Some(entry_offset)
										} else {
											None
										}
									}
									JsonObjectEntryPart::Key => {
										if rest.is_empty() {
											Some(key_offset)
										} else {
											None
										}
									}
									JsonObjectEntryPart::Value => {
										value.locate_fragment_at(code_map, rest, value_offset)
									}
								};
							}

							occ_count += 1;
						}
					}

					None
				}
			},
			None => Some(offset),
		}
	}
}

pub type JsonBacktrace<T> = [JsonBacktraceFrame<T>];

pub type JsonBacktraceBuf<T> = Vec<JsonBacktraceFrame<T>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonBacktraceFrame<T> {
	/// File in which the error occurred.
	pub file: T,

	/// Path to the exact fragment that caused the error.
	pub fragment: JsonFragmentPathBuf,
}

impl<T> JsonBacktraceFrame<T> {
	pub fn new(file: T) -> Self {
		Self {
			file,
			fragment: Vec::new(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum JsonBacktraceBuilder<'a, T> {
	Root,
	File(&'a Self, T),
	Item(&'a Self, JsonFragmentPathItemRef<'a>),
}

impl<T> JsonBacktraceBuilder<'_, T> {
	/// Creates an empty stack with no items.
	pub fn new() -> &'static Self {
		&Self::Root
	}

	pub fn file<'a>(&'a self, file: T) -> JsonBacktraceBuilder<'a, T> {
		JsonBacktraceBuilder::File(self, file)
	}

	pub fn push<'a>(&'a self, item: JsonFragmentPathItemRef<'a>) -> JsonBacktraceBuilder<'a, T> {
		JsonBacktraceBuilder::Item(self, item)
	}

	pub fn array_index(&self, index: usize) -> JsonBacktraceBuilder<'_, T> {
		self.push(JsonFragmentPathItemRef::ArrayItem(index))
	}

	/// Push a navigation step to the whole entry (key + value) for the first occurrence of `key`.
	///
	/// Equivalent to `object_entry_at(key, 0)`. For objects with duplicate keys, use
	/// [`object_entry_at`](Self::object_entry_at) to specify the occurrence index.
	pub fn object_entry<'b>(&'b self, key: impl Into<&'b str>) -> JsonBacktraceBuilder<'b, T> {
		self.push(JsonFragmentPathItemRef::ObjectEntry(
			key.into(),
			0,
			JsonObjectEntryPart::All,
		))
	}

	/// Push a navigation step to the whole entry (key + value) for the `occurrence`-th occurrence of `key`.
	///
	/// `0` selects the first occurrence, `1` the second, etc.
	pub fn object_entry_at<'b>(
		&'b self,
		key: impl Into<&'b str>,
		occurrence: usize,
	) -> JsonBacktraceBuilder<'b, T> {
		self.push(JsonFragmentPathItemRef::ObjectEntry(
			key.into(),
			occurrence,
			JsonObjectEntryPart::All,
		))
	}

	/// Push a navigation step to the key string of the first occurrence of `key`.
	///
	/// Equivalent to `object_key_at(key, 0)`. For objects with duplicate keys, use
	/// [`object_key_at`](Self::object_key_at) to specify the occurrence index.
	pub fn object_key<'b>(&'b self, key: impl Into<&'b str>) -> JsonBacktraceBuilder<'b, T> {
		self.push(JsonFragmentPathItemRef::ObjectEntry(
			key.into(),
			0,
			JsonObjectEntryPart::Key,
		))
	}

	/// Push a navigation step to the key string of the `occurrence`-th occurrence of `key`.
	///
	/// `0` selects the first occurrence, `1` the second, etc.
	pub fn object_key_at<'b>(
		&'b self,
		key: impl Into<&'b str>,
		occurrence: usize,
	) -> JsonBacktraceBuilder<'b, T> {
		self.push(JsonFragmentPathItemRef::ObjectEntry(
			key.into(),
			occurrence,
			JsonObjectEntryPart::Key,
		))
	}

	/// Push a navigation step to the value of the first occurrence of `key`.
	///
	/// Equivalent to `object_value_at(key, 0)`. For objects with duplicate keys, use
	/// [`object_value_at`](Self::object_value_at) to specify the occurrence index.
	pub fn object_value<'b>(&'b self, key: impl Into<&'b str>) -> JsonBacktraceBuilder<'b, T> {
		self.push(JsonFragmentPathItemRef::ObjectEntry(
			key.into(),
			0,
			JsonObjectEntryPart::Value,
		))
	}

	/// Push a navigation step to the value of the `occurrence`-th occurrence of `key`.
	///
	/// `0` selects the first occurrence, `1` the second, etc.
	pub fn object_value_at<'b>(
		&'b self,
		key: impl Into<&'b str>,
		occurrence: usize,
	) -> JsonBacktraceBuilder<'b, T> {
		self.push(JsonFragmentPathItemRef::ObjectEntry(
			key.into(),
			occurrence,
			JsonObjectEntryPart::Value,
		))
	}

	pub fn build(self) -> JsonBacktraceBuf<T::Owned>
	where
		T: Clone + IntoOwned,
	{
		match self {
			Self::Root => JsonBacktraceBuf::new(),
			Self::File(parent, file) => {
				let mut backtrace = parent.clone().build();
				backtrace.push(JsonBacktraceFrame::new(file.into_owned()));
				backtrace
			}
			Self::Item(parent, item) => {
				let mut backtrace = parent.clone().build();

				if let Some(last) = backtrace.last_mut() {
					last.fragment.push(item.into_owned());
				}

				backtrace
			}
		}
	}
}

impl<T: Clone + IntoOwned> From<JsonBacktraceBuilder<'_, T>> for JsonBacktraceBuf<T::Owned> {
	fn from(value: JsonBacktraceBuilder<'_, T>) -> Self {
		value.build()
	}
}

/// A value paired with a source location.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonLocated<T, F> {
	pub value: T,
	pub location: JsonBacktraceBuf<F>,
}

impl<T, F> JsonLocated<T, F> {
	pub fn new(value: T, location: JsonBacktraceBuf<F>) -> Self {
		Self { value, location }
	}

	pub fn map<U>(self, f: impl FnOnce(T) -> U) -> JsonLocated<U, F> {
		JsonLocated {
			value: f(self.value),
			location: self.location,
		}
	}

	pub fn cast<U>(self) -> JsonLocated<U, F>
	where
		T: Into<U>,
	{
		self.map(Into::into)
	}
}

impl<T: fmt::Display, F> fmt::Display for JsonLocated<T, F> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.value.fmt(f)
	}
}

impl<T: 'static + Error, F: fmt::Debug> Error for JsonLocated<T, F> {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(&self.value)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonFragmentPathSegment {
	ArrayItem(usize),
	ObjectEntry(String, usize, JsonObjectEntryPart),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JsonObjectEntryPart {
	All,
	Key,
	Value,
}

#[derive(Debug, Clone, Copy)]
pub enum JsonFragmentPathItemRef<'a> {
	ArrayItem(usize),
	ObjectEntry(&'a str, usize, JsonObjectEntryPart),
}

impl JsonFragmentPathItemRef<'_> {
	pub fn into_owned(self) -> JsonFragmentPathSegment {
		match self {
			Self::ArrayItem(i) => JsonFragmentPathSegment::ArrayItem(i),
			Self::ObjectEntry(k, n, part) => {
				JsonFragmentPathSegment::ObjectEntry(k.to_owned(), n, part)
			}
		}
	}
}

pub type JsonFragmentPathBuf = Vec<JsonFragmentPathSegment>;

pub type JsonFragmentPath = [JsonFragmentPathSegment];

pub trait JsonAt: Sized {
	fn json_at<F>(self, location: impl Into<JsonBacktraceBuf<F>>) -> JsonLocated<Self, F>;
}

impl<T> JsonAt for T {
	fn json_at<F>(self, location: impl Into<JsonBacktraceBuf<F>>) -> JsonLocated<Self, F> {
		JsonLocated::new(self, location.into())
	}
}

pub trait JsonErrorAt<T, E> {
	fn json_err_at<F>(
		self,
		location: impl Into<JsonBacktraceBuf<F>>,
	) -> Result<T, JsonLocated<E, F>>;
}

impl<T, E> JsonErrorAt<T, E> for Result<T, E> {
	fn json_err_at<F>(
		self,
		location: impl Into<JsonBacktraceBuf<F>>,
	) -> Result<T, JsonLocated<E, F>> {
		self.map_err(|e| e.json_at(location))
	}
}

#[cfg(test)]
mod tests {
	use crate::{JsonParse, JsonValue};

	use super::*;

	#[test]
	fn locate_01() {
		// { "a": 0, "b": [1, 2] }
		// index 0: object          (volume 9)
		// index 1: entry "a": 0   (volume 3)
		// index 2: key "a"         (volume 1)
		// index 3: value 0         (volume 1)
		// index 4: entry "b": [...](volume 5)
		// index 5: key "b"         (volume 1)
		// index 6: array [1, 2]    (volume 3)
		// index 7: value 1         (volume 1)
		// index 8: value 2         (volume 1)
		let (value, code_map) = JsonValue::parse_str(r#"{ "a": 0, "b": [1, 2] }"#).unwrap();

		// empty path → root (the object itself)
		assert_eq!(value.locate_fragment(&code_map, &[]), Some(0));

		// entry fragments (All)
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					0,
					JsonObjectEntryPart::All
				)]
			),
			Some(1)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"b".into(),
					0,
					JsonObjectEntryPart::All
				)]
			),
			Some(4)
		);

		// key fragments
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					0,
					JsonObjectEntryPart::Key
				)]
			),
			Some(2)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"b".into(),
					0,
					JsonObjectEntryPart::Key
				)]
			),
			Some(5)
		);

		// value fragments
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					0,
					JsonObjectEntryPart::Value
				)]
			),
			Some(3)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"b".into(),
					0,
					JsonObjectEntryPart::Value
				)]
			),
			Some(6)
		);

		// array elements inside "b"
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[
					JsonFragmentPathSegment::ObjectEntry("b".into(), 0, JsonObjectEntryPart::Value),
					JsonFragmentPathSegment::ArrayItem(0),
				]
			),
			Some(7)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[
					JsonFragmentPathSegment::ObjectEntry("b".into(), 0, JsonObjectEntryPart::Value),
					JsonFragmentPathSegment::ArrayItem(1),
				]
			),
			Some(8)
		);

		// missing key
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"z".into(),
					0,
					JsonObjectEntryPart::Value
				)]
			),
			None
		);

		// array index out of bounds
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[
					JsonFragmentPathSegment::ObjectEntry("b".into(), 0, JsonObjectEntryPart::Value),
					JsonFragmentPathSegment::ArrayItem(2),
				]
			),
			None
		);

		// All and Key are terminal – further segments after them return None
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[
					JsonFragmentPathSegment::ObjectEntry("b".into(), 0, JsonObjectEntryPart::All),
					JsonFragmentPathSegment::ArrayItem(0),
				]
			),
			None
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[
					JsonFragmentPathSegment::ObjectEntry("b".into(), 0, JsonObjectEntryPart::Key),
					JsonFragmentPathSegment::ArrayItem(0),
				]
			),
			None
		);
	}

	#[test]
	fn locate_duplicate_keys() {
		// { "a": 1, "a": 2 }
		// index 0: object                        (volume 7)
		// index 1: entry "a": 1, occurrence 0   (volume 3)
		// index 2: key "a",   occurrence 0       (volume 1)
		// index 3: value 1,   occurrence 0       (volume 1)
		// index 4: entry "a": 2, occurrence 1   (volume 3)
		// index 5: key "a",   occurrence 1       (volume 1)
		// index 6: value 2,   occurrence 1       (volume 1)
		let (value, code_map) = JsonValue::parse_str(r#"{ "a": 1, "a": 2 }"#).unwrap();

		// first occurrence
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					0,
					JsonObjectEntryPart::All
				)]
			),
			Some(1)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					0,
					JsonObjectEntryPart::Key
				)]
			),
			Some(2)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					0,
					JsonObjectEntryPart::Value
				)]
			),
			Some(3)
		);

		// second occurrence
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					1,
					JsonObjectEntryPart::All
				)]
			),
			Some(4)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					1,
					JsonObjectEntryPart::Key
				)]
			),
			Some(5)
		);
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					1,
					JsonObjectEntryPart::Value
				)]
			),
			Some(6)
		);

		// non-existent third occurrence
		assert_eq!(
			value.locate_fragment(
				&code_map,
				&[JsonFragmentPathSegment::ObjectEntry(
					"a".into(),
					2,
					JsonObjectEntryPart::Value
				)]
			),
			None
		);
	}
}
