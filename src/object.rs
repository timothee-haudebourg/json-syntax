use crate::Value;
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use locspan::Meta;
use locspan_derive::*;

mod index_map;

pub use index_map::Equivalent;
use index_map::IndexMap;

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

	pub fn as_pair(&self) -> (&Meta<Key, M>, &Meta<Value<M>, M>) {
		(&self.key, &self.value)
	}
}

/// Object.
#[derive(Clone, StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd, StrippedHash)]
#[stripped_ignore(M)]
pub struct Object<M> {
	/// The entries of the object, in order.
	entries: Vec<Entry<M>>,

	/// Maps each key to
	#[stripped_ignore]
	indexes: IndexMap,
}

impl<M> Default for Object<M> {
	fn default() -> Self {
		Self {
			entries: Vec::new(),
			indexes: IndexMap::new(),
		}
	}
}

impl<M> Object<M> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self { entries: Vec::with_capacity(cap), indexes: IndexMap::with_capacity(cap) }
	}

	pub fn from_vec(entries: Vec<Entry<M>>) -> Self {
		let mut indexes = IndexMap::new();
		for i in 0..entries.len() {
			indexes.insert(&entries, i);
		}

		Self { entries, indexes }
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

	pub fn entries(&self) -> &[Entry<M>] {
		&self.entries
	}

	pub fn iter(&self) -> core::slice::Iter<Entry<M>> {
		self.entries.iter()
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get<Q: ?Sized>(&self, key: &Q) -> Values<M>
	where
		Q: Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		Values {
			indexes,
			object: self,
		}
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_entries<Q: ?Sized>(&self, key: &Q) -> Entries<M>
	where
		Q: Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		Entries {
			indexes,
			object: self,
		}
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_with_index<Q: ?Sized>(&self, key: &Q) -> ValuesWithIndex<M>
	where
		Q: Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		ValuesWithIndex {
			indexes,
			object: self,
		}
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_entries_with_index<Q: ?Sized>(&self, key: &Q) -> EntriesWithIndex<M>
	where
		Q: Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		EntriesWithIndex {
			indexes,
			object: self,
		}
	}

	pub fn index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.indexes
			.get(&self.entries, key)
			.map(index_map::Indexes::first)
	}

	pub fn redundant_index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
	where
		Q: Hash + Equivalent<Key>,
	{
		self.indexes
			.get(&self.entries, key)
			.and_then(index_map::Indexes::redundant)
	}

	pub fn indexes_of<Q: ?Sized>(&self, key: &Q) -> Indexes
	where
		Q: Hash + Equivalent<Key>,
	{
		self.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default()
	}

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
		self.push_entry(Entry::new(key, value))
	}

	pub fn push_entry(&mut self, entry: Entry<M>) -> bool {
		let index = self.entries.len();
		self.entries.push(entry);
		self.indexes.insert(&self.entries, index)
	}

	/// Removes the entry at the given index.
	pub fn remove_at(&mut self, index: usize) -> Option<Entry<M>> {
		if index < self.entries.len() {
			self.indexes.remove(&self.entries, index);
			self.indexes.shift(index);
			Some(self.entries.remove(index))
		} else {
			None
		}
	}

	/// Inserts the given key-value pair.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	/// Otherwise, `None` is returned.
	pub fn insert(
		&mut self,
		key: Meta<Key, M>,
		value: Meta<Value<M>, M>,
	) -> Option<RemovedByInsertion<M>> {
		match self.index_of(key.value()) {
			Some(index) => {
				let mut entry = Entry::new(key, value);
				core::mem::swap(&mut entry, &mut self.entries[index]);
				Some(RemovedByInsertion {
					index,
					first: Some(entry),
					object: self,
				})
			}
			None => {
				self.push(key, value);
				None
			}
		}
	}

	/// Remove all entries associated to the given key.
	///
	/// Runs in `O(n)` time (average).
	pub fn remove<'q, Q: ?Sized>(&mut self, key: &'q Q) -> RemovedEntries<'_, 'q, M, Q>
	where
		Q: Hash + Equivalent<Key>,
	{
		RemovedEntries { key, object: self }
	}

	/// Recursively maps the metadata inside the object.
	pub fn map_metadata<N>(self, mut f: impl FnMut(M) -> N) -> Object<N> {
		let entries = self
			.entries
			.into_iter()
			.map(|entry| entry.map_metadata(&mut f))
			.collect();

		Object {
			entries,
			indexes: self.indexes,
		}
	}

	/// Tries to recursively maps the metadata inside the object.
	pub fn try_map_metadata<N, E>(
		self,
		mut f: impl FnMut(M) -> Result<N, E>,
	) -> Result<Object<N>, E> {
		let mut entries = Vec::with_capacity(self.len());
		for entry in self.entries {
			entries.push(entry.try_map_metadata(&mut f)?)
		}

		Ok(Object {
			entries,
			indexes: self.indexes,
		})
	}
}

impl<M: PartialEq> PartialEq for Object<M> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<M: Eq> Eq for Object<M> {}

impl<M: PartialOrd> PartialOrd for Object<M> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

impl<M: Ord> Ord for Object<M> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl<M: Hash> Hash for Object<M> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl<M: fmt::Debug> fmt::Debug for Object<M> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map()
			.entries(self.entries.iter().map(Entry::as_pair))
			.finish()
	}
}

impl<M> From<Vec<Entry<M>>> for Object<M> {
	fn from(entries: Vec<Entry<M>>) -> Self {
		Self::from_vec(entries)
	}
}

impl<'a, M> IntoIterator for &'a Object<M> {
	type Item = &'a Entry<M>;
	type IntoIter = core::slice::Iter<'a, Entry<M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<M> IntoIterator for Object<M> {
	type Item = Entry<M>;
	type IntoIter = std::vec::IntoIter<Entry<M>>;

	fn into_iter(self) -> Self::IntoIter {
		self.entries.into_iter()
	}
}

impl<M> Extend<Entry<M>> for Object<M> {
	fn extend<I: IntoIterator<Item = Entry<M>>>(&mut self, iter: I) {
		for entry in iter {
			self.push_entry(entry);
		}
	}
}

impl<M> FromIterator<Entry<M>> for Object<M> {
	fn from_iter<I: IntoIterator<Item = Entry<M>>>(iter: I) -> Self {
		let mut object = Object::default();
		object.extend(iter);
		object
	}
}

impl<M> Extend<(Meta<Key, M>, Meta<Value<M>, M>)> for Object<M> {
	fn extend<I: IntoIterator<Item = (Meta<Key, M>, Meta<Value<M>, M>)>>(&mut self, iter: I) {
		for (key, value) in iter {
			self.push(key, value);
		}
	}
}

impl<M> FromIterator<(Meta<Key, M>, Meta<Value<M>, M>)> for Object<M> {
	fn from_iter<I: IntoIterator<Item = (Meta<Key, M>, Meta<Value<M>, M>)>>(iter: I) -> Self {
		let mut object = Object::default();
		object.extend(iter);
		object
	}
}

pub enum Indexes<'a> {
	Some {
		first: Option<usize>,
		other: core::slice::Iter<'a, usize>,
	},
	None,
}

impl<'a> Default for Indexes<'a> {
	fn default() -> Self {
		Self::None
	}
}

impl<'a> Iterator for Indexes<'a> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			Self::Some { first, other } => match first.take() {
				Some(index) => Some(index),
				None => other.next().cloned(),
			},
			Self::None => None,
		}
	}
}

macro_rules! entries_iter {
	($($id:ident <$lft:lifetime> {
		type Item = $item:ty ;

		fn next(&mut $self:ident, $index:ident) { $e:expr }
	})*) => {
		$(
			pub struct $id<$lft, M> {
				indexes: Indexes<$lft>,
				object: &$lft Object<M>
			}

			impl<$lft, M> Iterator for $id<$lft, M> {
				type Item = $item;

				fn next(&mut $self) -> Option<Self::Item> {
					$self.indexes.next().map(|$index| $e)
				}
			}
		)*
	};
}

entries_iter! {
	Values<'a> {
		type Item = &'a Meta<Value<M>, M>;

		fn next(&mut self, index) { &self.object.entries[index].value }
	}

	ValuesWithIndex<'a> {
		type Item = (usize, &'a Meta<Value<M>, M>);

		fn next(&mut self, index) { (index, &self.object.entries[index].value) }
	}

	Entries<'a> {
		type Item = &'a Entry<M>;

		fn next(&mut self, index) { &self.object.entries[index] }
	}

	EntriesWithIndex<'a> {
		type Item = (usize, &'a Entry<M>);

		fn next(&mut self, index) { (index, &self.object.entries[index]) }
	}
}

pub struct RemovedByInsertion<'a, M> {
	index: usize,
	first: Option<Entry<M>>,
	object: &'a mut Object<M>,
}

impl<'a, M> Iterator for RemovedByInsertion<'a, M> {
	type Item = Entry<M>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.first.take() {
			Some(entry) => Some(entry),
			None => {
				let key = self.object.entries[self.index].key.value();
				self.object
					.redundant_index_of(key)
					.and_then(|index| self.object.remove_at(index))
			}
		}
	}
}

impl<'a, M> Drop for RemovedByInsertion<'a, M> {
	fn drop(&mut self) {
		self.last();
	}
}

pub struct RemovedEntries<'a, 'q, M, Q: ?Sized>
where
	Q: Hash + Equivalent<Key>,
{
	key: &'q Q,
	object: &'a mut Object<M>,
}

impl<'a, 'q, M, Q: ?Sized> Iterator for RemovedEntries<'a, 'q, M, Q>
where
	Q: Hash + Equivalent<Key>,
{
	type Item = Entry<M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.object
			.index_of(self.key)
			.and_then(|index| self.object.remove_at(index))
	}
}

impl<'a, 'q, M, Q: ?Sized> Drop for RemovedEntries<'a, 'q, M, Q>
where
	Q: Hash + Equivalent<Key>,
{
	fn drop(&mut self) {
		self.last();
	}
}
