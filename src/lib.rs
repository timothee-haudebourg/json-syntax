use generic_json::{Json, ValueRef, ValueMut};
use json_number::NumberBuf;
use std::{
	borrow::{Borrow, BorrowMut},
	ops::{Deref, DerefMut},
	cmp::Ordering,
	collections::BTreeMap
};

pub mod parse;

const SMALL_STRING_CAPACITY: usize = 16;
type SmallString = smallstr::SmallString<[u8; SMALL_STRING_CAPACITY]>;

/// JSON value.
pub struct Value<M = source_span::Span> {
	value: generic_json::Value<Self>,
	metadata: M,
}

impl<M> Value<M> {
	pub fn metadata(&self) -> &M {
		&self.metadata
	}

	pub fn metadata_mut(&mut self) -> &mut M {
		&mut self.metadata
	}
}

impl<M> Deref for Value<M> {
	type Target = generic_json::Value<Self>;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<M> DerefMut for Value<M> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<M> Borrow<generic_json::Value<Self>> for Value<M> {
	fn borrow(&self) -> &generic_json::Value<Self> {
		&self.value
	}
}

impl<M> BorrowMut<generic_json::Value<Self>> for Value<M> {
	fn borrow_mut(&mut self) -> &mut generic_json::Value<Self> {
		&mut self.value
	}
}

pub struct Key<M = source_span::Span> {
	key: SmallString,
	metadata: M
}

impl<M> Key<M> {
	pub fn metadata(&self) -> &M {
		&self.metadata
	}

	pub fn metadata_mut(&mut self) -> &mut M {
		&mut self.metadata
	}
}

impl<M> AsRef<str> for Key<M> {
	fn as_ref(&self) -> &str {
		self.key.as_ref()
	}
}

impl<M> PartialEq for Key<M> {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
	}
}

impl<M> Eq for Key<M> {}

impl<M> PartialOrd for Key<M> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.key.partial_cmp(&other.key)
	}
}

impl<M> Ord for Key<M> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.key.cmp(&other.key)
	}
}

impl<M> Borrow<str> for Key<M> {
	fn borrow(&self) -> &str {
		self.key.as_ref()
	}
}

impl<M> Json for Value<M> {
	/// Metadata type attached to each value.
	type MetaData = M;

	/// Literal number type.
	type Number = NumberBuf;

	/// String type.
	type String = SmallString;

	/// Array type.
	type Array = Vec<Self>;

	/// Object key type.
	type Key = Key<M>;

	/// Object type.
	type Object = BTreeMap<Key<M>, Self>;

	/// Creates a new "meta value" from a `Value` and its associated metadata.
	fn new(value: generic_json::Value<Self>, metadata: Self::MetaData) -> Self {
		Self {
			value,
			metadata
		}
	}

	/// Returns a reference to the actual JSON value (without the metadata).
	fn as_value_ref(&self) -> ValueRef<'_, Self> {
		self.value.as_value_ref()
	}

	/// Returns a mutable reference to the actual JSON value (without the metadata).
	fn as_value_mut(&mut self) -> ValueMut<'_, Self> {
		self.value.as_value_mut()
	}

	/// Returns a reference to the metadata associated to the JSON value.
	fn metadata(&self) -> &Self::MetaData {
		&self.metadata
	}

	/// Returns a pair containing a reference to the JSON value and a reference to its metadata.
	fn as_pair(&self) -> (ValueRef<'_, Self>, &Self::MetaData) {
		(self.value.as_value_ref(), &self.metadata)
	}

	/// Returns a pair containing a mutable reference to the JSON value and a reference to its metadata.
	fn as_pair_mut(&mut self) -> (ValueMut<'_, Self>, &Self::MetaData) {
		(self.value.as_value_mut(), &self.metadata)
	}
}
