use core::fmt;
use std::error::Error;

use into_owned_trait::IntoOwned;

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
			ObjectEntryPart::All,
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
			ObjectEntryPart::All,
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
			ObjectEntryPart::Key,
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
			ObjectEntryPart::Key,
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
			ObjectEntryPart::Value,
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
			ObjectEntryPart::Value,
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
	ObjectEntry(String, usize, ObjectEntryPart),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectEntryPart {
	All,
	Key,
	Value,
}

#[derive(Debug, Clone, Copy)]
pub enum JsonFragmentPathItemRef<'a> {
	ArrayItem(usize),
	ObjectEntry(&'a str, usize, ObjectEntryPart),
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
