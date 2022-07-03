use locspan::Meta;
use locspan_derive::*;
use core::hash::Hash;
use crate::Value;

mod index_map;

use index_map::IndexMap;
pub use index_map::Equivalent;

/// Object key stack capacity.
///
/// If the key is longer than this value,
/// it will be stored on the heap.
pub const KEY_CAPACITY: usize = 16;

/// Object key.
pub type Key = smallstr::SmallString<[u8; KEY_CAPACITY]>;

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
#[stripped_ignore(M)]
pub struct Entry<M> {
	#[stripped_deref]
	pub key: Meta<Key, M>,
	pub value: Meta<Value<M>, M>,
}

impl<M> Entry<M> {
	pub fn new(key: Meta<Key, M>, value: Meta<Value<M>, M>) -> Self {
		Self { key, value }
	}

	pub fn map_metadata<N>(self, mut f: impl FnMut(M) -> N) -> Entry<N> {
		Entry {
			key: self.key.map_metadata(&mut f),
			value: self.value.map_metadata_recursively(f),
		}
	}

	pub fn try_map_metadata<N, E>(
		self,
		mut f: impl FnMut(M) -> Result<N, E>,
	) -> Result<Entry<N>, E> {
		Ok(Entry {
			key: self.key.try_map_metadata(&mut f)?,
			value: self.value.try_map_metadata_recursively(f)?,
		})
	}
}

/// Object.
pub struct Object<M> {
	/// The entries of the object, in order.
	entries: Vec<Entry<M>>,

	/// Maps each key to 
	indexes: IndexMap
}

impl<M> Default for Object<M> {
	fn default() -> Self {
		Self {
			entries: Vec::new(),
			indexes: IndexMap::new()
		}
	}
}

impl<M> Object<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn capacity(&self) -> usize {
		self.entries.capacity()
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	// /// Returns an iterator over the values associated to the given key.
	// /// 
	// /// For now, this runs in `O(n)`, where `n` is the number of entries in the
	// /// object.
	// pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&Values<M>> where Q: Hash + Equivalent<Key> {
	// 	todo!()
	// }

	// /// Returns an iterator over the values associated to the given key,
	// /// with the index of each matching entry in the object.
	// /// 
	// /// For now, this runs in `O(n)`, where `n` is the number of entries in the
	// /// object.
	// pub fn get_with_index<Q: ?Sized>(&self, key: &Q) -> Option<&Values<M>> where Q: Hash + Equivalent<Key> {
	// 	todo!()
	// }

	// /// Returns an iterator over the key-value pairs matching the given key.
	// /// 
	// /// For now, this runs in `O(n)`, where `n` is the number of entries in the
	// /// object.
	// pub fn get_keys_values<Q: ?Sized>(&self, key: &Q) -> Option<KeysValues<M>> where Q: Hash + Equivalent<Key> {
	// 	todo!()
	// }

	// /// Returns an iterator over the key-value pairs matching the given key,
	// /// with the index of each matching entry in the object.
	// /// 
	// /// For now, this runs in `O(n)`, where `n` is the number of entries in the
	// /// object.
	// pub fn get_keys_values_with_index<Q: ?Sized>(&self, key: &Q) -> Option<KeysValues<M>> where Q: Hash + Equivalent<Key> {
	// 	todo!()
	// }

	pub fn index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize> where Q: Hash + Equivalent<Key> {
		todo!()
	}

	// pub fn indexes_of<Q: ?Sized>(&self, key: &Q) -> Option<Indexes> where Q: Hash + Equivalent<Key> {
	// 	todo!()
	// }

	pub fn first(&self) -> Option<&Entry<M>> {
		self.entries.first()
	}

	pub fn last(&self) -> Option<&Entry<M>> {
		self.entries.last()
	}

	/// Push the given key-value pair to the end of the object.
	/// 
	/// Returns `true` if the key was not already present in the object,
	/// and `false` otherwise.
	/// Any previous entry matching the key is **not** overridden: duplicates
	/// are preserved, in order.
	/// 
	/// Runs in `O(1)`.
	pub fn push(&mut self, key: Meta<Key, M>, value: Meta<Value<M>, M>) -> bool {
		let index = self.entries.len();
		self.entries.push(Entry::new(key, value));
		self.indexes.insert(&self.entries, index)
	}

	pub fn remove_at(&mut self, index: usize) -> Option<Entry<M>> {
		todo!()
	}

	pub fn insert(&mut self, key: Meta<Key, M>, value: Meta<Value<M>, M>) -> Option<Entry<M>> {
		todo!()
	}

	/// Remove all entries associated to the given key.
	/// 
	/// Runs in `O(n)` time (average).
	pub fn remove<'q, Q: ?Sized>(&mut self, key: &'q Q) -> Option<RemovedEntries<'_, 'q, M, Q>> where Q: Hash + Equivalent<Key> {
		todo!()
	}
}

pub struct RemovedEntries<'a, 'q, M, Q: ?Sized> where Q: Hash + Equivalent<Key> {
	key: &'q Q,
	object: &'a mut Object<M>
}

impl<'a, 'q, M, Q: ?Sized> Iterator for RemovedEntries<'a, 'q, M, Q> where Q: Hash + Equivalent<Key> {
	type Item = Entry<M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.object.index_of(self.key).map(|index| {
			self.object.indexes.remove(&self.object.entries, index);
			self.object.indexes.shift(index);
			self.object.entries.remove(index)
		})
	}
}

impl<'a, 'q, M, Q: ?Sized> Drop for RemovedEntries<'a, 'q, M, Q> where Q: Hash + Equivalent<Key> {
	fn drop(&mut self) {
		self.last();
	}
}