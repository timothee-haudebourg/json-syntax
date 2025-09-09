use crate::code_map::Mapped;
use crate::lexical::{LexicalEq, LexicalHash, LexicalPartialEq};
use crate::{CodeMap, FragmentRef, Value};
use btree_indexmap::{BTreeIndexMultiMap, Comparable};
use core::fmt;
use core::hash::{Hash, Hasher};

pub use btree_indexmap::multi_map::IndexesIter;

/// Object key stack capacity.
///
/// If the key is longer than this value,
/// it will be stored on the heap.
pub const KEY_CAPACITY: usize = 16;

/// Object key.
pub type Key = smallstr::SmallString<[u8; KEY_CAPACITY]>;

/// Object entry.
pub type Entry = (Key, Value);

/// Object entry reference.
pub type EntryRef<'a> = (&'a Key, &'a Value);

fn entry_ref((k, v): &Entry) -> EntryRef {
	(k, v)
}

fn get_entry_fragment<'a>(entry: &'a Entry, index: usize) -> Result<FragmentRef<'a>, usize> {
	match index {
		0 => Ok(FragmentRef::Entry(entry_ref(entry))),
		1 => Ok(FragmentRef::Key(&entry.0)),
		_ => entry.1.get_fragment(index - 2),
	}
}

/// Object entry, with code map information.
pub type MappedEntry = Mapped<(Mapped<Key>, Mapped<Value>)>;

fn mapped_entry(
	(key, value): Entry,
	offset: usize,
	key_offset: usize,
	value_offset: usize,
) -> MappedEntry {
	Mapped(
		(Mapped(key, key_offset), Mapped(value, value_offset)),
		offset,
	)
}

/// Object entry reference, with code map information.
pub type MappedEntryRef<'a> = Mapped<(Mapped<&'a Key>, Mapped<&'a Value>)>;

fn mapped_entry_ref(
	(key, value): EntryRef,
	offset: usize,
	key_offset: usize,
	value_offset: usize,
) -> MappedEntryRef {
	Mapped(
		(Mapped(key, key_offset), Mapped(value, value_offset)),
		offset,
	)
}

pub type IndexedMappedEntry<'a> = (usize, MappedEntryRef<'a>);

pub type IndexedMappedValue<'a> = (usize, Mapped<&'a Value>);

/// Object.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Object {
	map: BTreeIndexMultiMap<Key, Value>,
}

impl Default for Object {
	fn default() -> Self {
		Self {
			map: BTreeIndexMultiMap::new(),
		}
	}
}

impl Object {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_vec(entries: Vec<(Key, Value)>) -> Self {
		Self {
			map: entries.into_iter().collect(),
		}
	}

	pub fn capacity(&self) -> usize {
		self.map.capacity()
	}

	pub fn len(&self) -> usize {
		self.map.len()
	}

	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}

	pub fn get_fragment(&self, mut index: usize) -> Result<FragmentRef, usize> {
		for e in self.map.as_entries() {
			match get_entry_fragment(e, index) {
				Ok(value) => return Ok(value),
				Err(i) => index = i,
			}
		}

		Err(index)
	}

	pub fn entries(&self) -> &[Entry] {
		self.map.as_entries()
	}

	pub fn iter(&self) -> Iter {
		self.map.iter()
	}

	pub fn iter_mut(&mut self) -> IterMut {
		self.map.iter_mut()
	}

	pub fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			entries: self.map.iter(),
			code_map,
			offset: offset + 1,
		}
	}

	pub fn into_iter_mapped<'m>(self, code_map: &'m CodeMap, offset: usize) -> IntoIterMapped<'m> {
		IntoIterMapped {
			entries: self.map.into_iter(),
			code_map,
			offset: offset + 1,
		}
	}

	/// Checks if this object contains the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn contains_key<Q>(&self, key: &Q) -> bool
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.contains_key(key)
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get<Q>(&self, key: &Q) -> Get
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get(key)
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_mut<Q>(&mut self, key: &Q) -> GetMut
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get_mut(key)
	}

	/// Returns the unique entry value matching the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_unique<Q>(&self, key: &Q) -> Result<Option<&Value>, DuplicateEntry<EntryRef>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.map.get_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry.1)),
			},
			None => Ok(None),
		}
	}

	/// Returns the unique entry value matching the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_unique_mut<Q>(
		&mut self,
		key: &Q,
	) -> Result<Option<&mut Value>, DuplicateEntry<EntryRef>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.map.get_entries_mut(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => {
					let entry = (entry.0, &*entry.1);
					let duplicate = (duplicate.0, &*duplicate.1);
					Err(DuplicateEntry(entry, duplicate))
				}
				None => Ok(Some(entry.1)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_entries<Q>(&self, key: &Q) -> GetEntries
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get_entries(key)
	}

	/// Returns the unique entry matching the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_unique_entry<Q>(&self, key: &Q) -> Result<Option<EntryRef>, DuplicateEntry<EntryRef>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_indexed<Q>(&self, key: &Q) -> GetIndexed
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get_indexed(key)
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_indexed_entries<Q>(&self, key: &Q) -> GetIndexedEntries
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get_indexed_entries(key)
	}

	/// Returns the (first) value associated to `key`, or insert a `key`-`value`
	/// entry where `value` is returned by the given function `f`.
	pub fn get_or_insert_with<Q>(&mut self, key: impl Into<Key>, f: impl FnOnce() -> Value) -> Get {
		self.map.get_or_insert_with(key.into(), f)
	}

	/// Returns a mutable reference to the (first) value associated to `key`, or
	/// insert a `key`-`value` entry where `value` is returned by the given
	/// function `f`.
	pub fn get_or_insert_mut_with<Q>(
		&mut self,
		key: impl Into<Key>,
		f: impl FnOnce() -> Value,
	) -> GetMut {
		self.map.get_or_insert_mut_with(key.into(), f)
	}

	pub fn index_of<Q>(&self, key: &Q) -> Option<usize>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.index_of(key)
	}

	pub fn indexes_of<Q>(&self, key: &Q) -> IndexesIter
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.indexes_of(key)
	}

	pub fn redundant_index_of<Q>(&self, key: &Q) -> Option<usize>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.indexes_of(key).nth(1)
	}

	/// Returns an iterator over the mapped entries matching the given key.
	///
	/// Runs in `O(n)` (average). `O(log(n))` to find the entry, `O(n)` to
	/// compute the entry fragment offset.
	pub fn get_mapped_entries<'m, Q>(
		&self,
		code_map: &'m CodeMap,
		offset: usize,
		key: &Q,
	) -> GetMappedEntries<'_, 'm>
	where
		Q: ?Sized + Comparable<Key>,
	{
		GetMappedEntries {
			indexes: self.map.indexes_of(key),
			object: self,
			code_map,
			offset: offset + 1,
			last_index: 0,
		}
	}

	/// Returns the unique mapped entry matching the given key.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_unique_mapped_entry<Q>(
		&self,
		code_map: &CodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<MappedEntryRef>, DuplicateEntry<MappedEntryRef>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_mapped_entries(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the mapped entries matching the given key, with
	/// their index.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_mapped_entries_with_index<'m, Q>(
		&self,
		code_map: &'m CodeMap,
		offset: usize,
		key: &Q,
	) -> GetIndexedMappedEntries<'_, 'm>
	where
		Q: ?Sized + Comparable<Key>,
	{
		GetIndexedMappedEntries {
			indexes: self.map.indexes_of(key),
			object: self,
			code_map,
			offset: offset + 1,
			last_index: 0,
		}
	}

	/// Returns the unique mapped entry matching the given key, with its index.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_unique_mapped_entry_with_index<Q>(
		&self,
		code_map: &CodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<IndexedMappedEntry>, DuplicateEntry<IndexedMappedEntry>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_mapped_entries_with_index(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the mapped values matching the given key.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_mapped<'m, Q>(
		&self,
		code_map: &'m CodeMap,
		offset: usize,
		key: &Q,
	) -> GetMapped<'_, 'm>
	where
		Q: ?Sized + Comparable<Key>,
	{
		GetMapped {
			indexes: self.map.indexes_of(key),
			object: self,
			code_map,
			offset: offset + 1,
			last_index: 0,
		}
	}

	/// Returns the unique mapped values matching the given key.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_unique_mapped<Q>(
		&self,
		code_map: &CodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<Mapped<&Value>>, DuplicateEntry<Mapped<&Value>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_mapped(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the mapped values matching the given key, with
	/// their index.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_indexed_mapped<'m, Q>(
		&self,
		code_map: &'m CodeMap,
		offset: usize,
		key: &Q,
	) -> GetIndexedMapped<'_, 'm>
	where
		Q: ?Sized + Comparable<Key>,
	{
		GetIndexedMapped {
			indexes: self.map.indexes_of(key),
			object: self,
			code_map,
			offset: offset + 1,
			last_index: 0,
		}
	}

	/// Returns the unique mapped values matching the given key, with its index.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_unique_mapped_with_index<Q>(
		&self,
		code_map: &CodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<IndexedMappedValue>, DuplicateEntry<IndexedMappedValue>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_indexed_mapped(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	pub fn first(&self) -> Option<EntryRef> {
		self.map.first()
	}

	pub fn last(&self) -> Option<EntryRef> {
		self.map.last()
	}

	/// Push the given key-value pair to the end of the object.
	///
	/// Returns `true` if the key was not already present in the object,
	/// and `false` otherwise.
	/// Any previous entry matching the key is **not** overridden: duplicates
	/// are preserved, in order.
	///
	/// Runs in `O(1)`.
	pub fn push_back(&mut self, key: impl Into<Key>, value: Value) -> (usize, bool) {
		self.map.push_back(key.into(), value)
	}

	pub fn push_entry_back(&mut self, (key, value): Entry) -> (usize, bool) {
		self.map.push_back(key, value)
	}

	/// Push the given key-value pair to the top of the object.
	///
	/// Returns `true` if the key was not already present in the object,
	/// and `false` otherwise.
	/// Any previous entry matching the key is **not** overridden: duplicates
	/// are preserved, in order.
	///
	/// Runs in `O(n)`.
	pub fn push_front(&mut self, key: impl Into<Key>, value: Value) -> bool {
		self.map.push_front(key.into(), value)
	}

	pub fn push_entry_front(&mut self, (key, value): Entry) -> bool {
		self.map.push_front(key, value)
	}

	/// Removes the entry at the given index `i`.
	pub fn swap_remove_at(&mut self, i: usize) -> Option<Entry> {
		self.map.swap_remove_at(i)
	}

	/// Removes the entry at the given index `i`.
	pub fn shift_remove_at(&mut self, i: usize) -> Option<Entry> {
		self.map.shift_remove_at(i)
	}

	/// Inserts the given key-value pair.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	/// Otherwise, `None` is returned.
	pub fn shift_insert(&mut self, key: impl Into<Key>, value: Value) -> ShiftInsert {
		self.map.shift_insert(key.into(), value)
	}

	pub fn swap_insert(&mut self, key: impl Into<Key>, value: Value) -> SwapInsert {
		self.map.swap_insert(key.into(), value)
	}

	/// Inserts the given key-value pair on top of the object.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	pub fn shift_insert_front(&mut self, key: impl Into<Key>, value: Value) -> ShiftInsertFront {
		self.map.shift_insert_front(key.into(), value)
	}

	pub fn swap_insert_front(&mut self, key: impl Into<Key>, value: Value) -> SwapInsertFront {
		self.map.swap_insert_front(key.into(), value)
	}

	pub fn shift_insert_back_full(
		&mut self,
		key: impl Into<Key>,
		value: Value,
	) -> (usize, ShiftInsertBack) {
		self.map.shift_insert_back_full(key.into(), value)
	}

	pub fn shift_insert_back(&mut self, key: impl Into<Key>, value: Value) -> ShiftInsertBack {
		self.map.shift_insert_back(key.into(), value)
	}

	/// Alias of [`Self::shift_insert_back`].
	pub fn insert(&mut self, key: impl Into<Key>, value: Value) -> ShiftInsertBack {
		self.shift_insert_back(key, value)
	}

	pub fn swap_insert_back_full(
		&mut self,
		key: impl Into<Key>,
		value: Value,
	) -> (usize, SwapInsertBack) {
		self.map.swap_insert_back_full(key.into(), value)
	}

	pub fn swap_insert_back(&mut self, key: impl Into<Key>, value: Value) -> SwapInsertBack {
		self.map.swap_insert_back(key.into(), value)
	}

	pub fn shift_remove_indexed_entries<'q, Q>(&mut self, key: &'q Q) -> ShiftRemoveIndexedEntries
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.shift_remove_indexed_entries(key)
	}

	/// Remove all entries associated to the given key.
	pub fn shift_remove_entries<'q, Q>(&mut self, key: &'q Q) -> ShiftRemoveEntries
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.shift_remove_entries(key)
	}

	/// Remove all entries associated to the given key.
	pub fn shift_remove<'q, Q>(&mut self, key: &'q Q) -> ShiftRemove
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.shift_remove(key)
	}

	/// Alias of [`Self::shift_remove`].
	pub fn remove<'q, Q>(&mut self, key: &'q Q) -> ShiftRemove
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.shift_remove(key)
	}

	/// Alias of [`Self::shift_remove_unique`].
	pub fn remove_unique<'q, Q>(
		&mut self,
		key: &'q Q,
	) -> Result<Option<Entry>, DuplicateEntry<Entry>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.shift_remove_unique(key)
	}

	pub fn swap_remove_indexed_entries<'q, Q>(&mut self, key: &'q Q) -> SwapRemoveIndexedEntries
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.swap_remove_indexed_entries(key)
	}

	pub fn swap_remove_entries<'q, Q>(&mut self, key: &'q Q) -> SwapRemoveEntries
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.swap_remove_entries(key)
	}

	pub fn swap_remove<'q, Q>(&mut self, key: &'q Q) -> SwapRemove
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.swap_remove(key)
	}

	/// Remove the unique entry associated to the given key.
	///
	/// Returns an error if multiple entries match the key.
	pub fn swap_remove_unique<Q>(&mut self, key: &Q) -> Result<Option<Entry>, DuplicateEntry<Entry>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.swap_remove_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Remove the unique entry associated to the given key.
	///
	/// Returns an error if multiple entries match the key.
	pub fn shift_remove_unique<Q>(
		&mut self,
		key: &Q,
	) -> Result<Option<Entry>, DuplicateEntry<Entry>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.shift_remove_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Sort the entries by key name.
	///
	/// The relative order of entries with the same key is unchanged.
	pub fn sort(&mut self) {
		self.map.sort();
	}

	/// Puts this JSON object in canonical form according to
	/// [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785#name-generation-of-canonical-jso).
	///
	/// This will canonicalize the entries and sort them by key.
	/// The relative order of entries with the same key is unchanged.
	#[cfg(feature = "canonicalize")]
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		for (_, item) in self.iter_mut() {
			item.canonicalize_with(buffer);
		}

		self.sort()
	}

	/// Puts this JSON object in canonical form according to
	/// [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785#name-generation-of-canonical-jso).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalize(&mut self) {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalize_with(&mut buffer)
	}
}

pub type Iter<'a> = btree_indexmap::multi_map::Iter<'a, Key, Value>;

pub type IterMut<'a> = btree_indexmap::multi_map::IterMut<'a, Key, Value>;

pub type IntoIter = btree_indexmap::multi_map::IntoIter<Key, Value>;

pub type Get<'a> = btree_indexmap::multi_map::Get<'a, Key, Value>;

pub type GetIndexed<'a> = btree_indexmap::multi_map::GetIndexed<'a, Key, Value>;

pub type GetEntries<'a> = btree_indexmap::multi_map::GetEntries<'a, Key, Value>;

pub type GetIndexedEntries<'a> = btree_indexmap::multi_map::GetIndexedEntries<'a, Key, Value>;

pub type GetMut<'a> = btree_indexmap::multi_map::GetMut<'a, Key, Value>;

pub type ShiftInsert<'a> = btree_indexmap::multi_map::ShiftInsert<'a, Key, Value>;

pub type SwapInsert<'a> = btree_indexmap::multi_map::SwapInsert<'a, Key, Value>;

pub type ShiftInsertBack<'a> = btree_indexmap::multi_map::ShiftInsertBack<'a, Key, Value>;

pub type SwapInsertBack<'a> = btree_indexmap::multi_map::SwapInsertBack<'a, Key, Value>;

pub type ShiftInsertFront<'a> = btree_indexmap::multi_map::ShiftInsertFront<'a, Key, Value>;

pub type SwapInsertFront<'a> = btree_indexmap::multi_map::SwapInsertFront<'a, Key, Value>;

pub type ShiftRemoveIndexedEntries<'a> =
	btree_indexmap::multi_map::ShiftRemoveIndexedEntries<'a, Key, Value>;

pub type ShiftRemoveEntries<'a> = btree_indexmap::multi_map::ShiftRemoveEntries<'a, Key, Value>;

pub type ShiftRemove<'a> = btree_indexmap::multi_map::ShiftRemove<'a, Key, Value>;

pub type SwapRemoveIndexedEntries<'a> =
	btree_indexmap::multi_map::SwapRemoveIndexedEntries<'a, Key, Value>;

pub type SwapRemoveEntries<'a> = btree_indexmap::multi_map::SwapRemoveEntries<'a, Key, Value>;

pub type SwapRemove<'a> = btree_indexmap::multi_map::SwapRemove<'a, Key, Value>;

pub struct IterMapped<'a, 'm> {
	entries: Iter<'a>,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'a, 'm> Iterator for IterMapped<'a, 'm> {
	type Item = MappedEntryRef<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.entries.next().map(|e| {
			let offset = self.offset;
			self.offset += 2 + self.code_map.get(self.offset + 2).unwrap().volume;
			mapped_entry_ref(e, offset, offset + 1, offset + 2)
		})
	}
}

pub struct IntoIterMapped<'m> {
	entries: IntoIter,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'m> Iterator for IntoIterMapped<'m> {
	type Item = MappedEntry;

	fn next(&mut self) -> Option<Self::Item> {
		self.entries.next().map(|e| {
			let offset = self.offset;
			self.offset += 2 + self.code_map.get(self.offset + 2).unwrap().volume;
			mapped_entry(e, offset, offset + 1, offset + 2)
		})
	}
}

macro_rules! mapped_entries_iter {
	($($id:ident <$lft:lifetime> {
		type Item = $item:ty ;

		fn next(&mut $self:ident, $index:ident) { $e:expr }
	})*) => {
		$(
			pub struct $id<$lft, 'm> {
				indexes: IndexesIter<$lft>,
				object: &$lft Object,
				code_map: &'m CodeMap,
				offset: usize,
				last_index: usize
			}

			impl<$lft, 'm> Iterator for $id<$lft, 'm> {
				type Item = $item;

				fn next(&mut $self) -> Option<Self::Item> {
					$self.indexes.next().map(|$index| {
						while $self.last_index < $index {
							$self.last_index += 1;
							$self.offset += 2 + $self.code_map.get($self.offset+2).unwrap().volume;
						}

						$e
					})
				}
			}
		)*
	};
}

mapped_entries_iter! {
	GetMappedEntries<'a> {
		type Item = MappedEntryRef<'a>;

		fn next(&mut self, index) {
			mapped_entry_ref(entry_ref(&self.object.entries()[index]), self.offset, self.offset+1, self.offset+2)
		}
	}

	GetIndexedMappedEntries<'a> {
		type Item = (usize, MappedEntryRef<'a>);

		fn next(&mut self, index) {
			(
				index,
				mapped_entry_ref(entry_ref(&self.object.entries()[index]), self.offset, self.offset+1, self.offset+2)
			)
		}
	}

	GetMapped<'a> {
		type Item = Mapped<&'a Value>;

		fn next(&mut self, index) {
			Mapped(
				&self.object.entries()[index].1,
				self.offset+2,
			)
		}
	}

	GetIndexedMapped<'a> {
		type Item = (usize, Mapped<&'a Value>);

		fn next(&mut self, index) {
			(
				index,
				Mapped(
					&self.object.entries()[index].1,
					self.offset+2
				)
			)
		}
	}
}

impl LexicalPartialEq for Object {
	fn lexical_eq(&self, other: &Self) -> bool {
		self.map.as_entries().eq(other.map.as_entries())
	}
}

impl LexicalEq for Object {}

impl Hash for Object {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.map.hash(state);
	}
}

impl LexicalHash for Object {
	fn lexical_hash<H: Hasher>(&self, state: &mut H) {
		self.map.as_entries().hash(state)
	}
}

impl fmt::Debug for Object {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map().entries(self.map.iter()).finish()
	}
}

impl From<Vec<Entry>> for Object {
	fn from(entries: Vec<Entry>) -> Self {
		Self::from_vec(entries)
	}
}

impl<'a> IntoIterator for &'a Object {
	type Item = EntryRef<'a>;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut Object {
	type Item = (&'a Key, &'a mut Value);
	type IntoIter = IterMut<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl IntoIterator for Object {
	type Item = Entry;
	type IntoIter = std::vec::IntoIter<Entry>;

	fn into_iter(self) -> Self::IntoIter {
		self.map.into_iter()
	}
}

impl Extend<Entry> for Object {
	fn extend<I: IntoIterator<Item = Entry>>(&mut self, iter: I) {
		for entry in iter {
			self.push_entry_back(entry);
		}
	}
}

impl FromIterator<Entry> for Object {
	fn from_iter<I: IntoIterator<Item = Entry>>(iter: I) -> Self {
		let mut object = Object::default();
		object.extend(iter);
		object
	}
}

/// Duplicate entry error.
#[derive(Debug)]
pub struct DuplicateEntry<T = Entry>(pub T, pub T);

impl fmt::Display for DuplicateEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "duplicate entry `{}`", self.0 .0)
	}
}

pub type DuplicateEntryRef<'a> = DuplicateEntry<EntryRef<'a>>;

impl std::error::Error for DuplicateEntry {}

#[cfg(test)]
mod tests {
	use crate::lexical::BorrowLexical;

	use super::*;

	#[test]
	fn remove() {
		let mut object = Object::new();
		object.insert("a", Value::Null);

		object.remove("a");
		object.remove("a");
	}

	#[test]
	fn unordered_eq1() {
		let mut a = Object::new();
		a.push_back("a", Value::Null);
		a.push_back("b", Value::Null);

		let mut b = Object::new();
		b.push_back("b", Value::Null);
		b.push_back("a", Value::Null);

		assert_ne!(a, b);
		assert_eq!(a.as_lexical(), b.as_lexical())
	}

	#[test]
	fn unordered_eq2() {
		let mut a = Object::new();
		a.push_back("a", Value::Null);
		a.push_back("a", Value::Null);

		let mut b = Object::new();
		b.push_back("a", Value::Null);
		b.push_back("a", Value::Null);

		assert_eq!(a, b);
		assert_eq!(a.as_lexical(), b.as_lexical())
	}

	#[test]
	fn insert_front1() {
		let mut a = Object::new();
		a.push_back("a", Value::Null);
		a.push_back("b", Value::Null);
		a.push_back("c", Value::Null);
		a.shift_insert_front("b", Value::Null);

		let mut b = Object::new();
		b.push_back("b", Value::Null);
		b.push_back("a", Value::Null);
		b.push_back("c", Value::Null);

		assert_eq!(a, b);
	}

	#[test]
	fn insert_front2() {
		let mut a = Object::new();
		a.push_back("a", Value::Null);
		a.push_back("a", Value::Null);
		a.push_back("c", Value::Null);
		a.shift_insert_front("a", Value::Null);

		let mut b = Object::new();
		b.push_back("a", Value::Null);
		b.push_back("c", Value::Null);

		assert_eq!(a, b);
	}

	#[test]
	fn mapped_entries() {
		use crate::Parse;
		let (json, code_map) = crate::Value::parse_str(
			r#"{ "0": [null, null], "1": { "foo": 0, "bar": 1 }, "0": null }"#,
		)
		.unwrap();
		let object = json.into_object().unwrap();

		let offsets: Vec<_> = object
			.get_mapped_entries(&code_map, 0, "0")
			.map(|Mapped((key, value), offset)| (offset, key.offset(), value.offset()))
			.collect();

		assert_eq!(offsets, [(1, 2, 3), (15, 16, 17)]);

		let offsets: Vec<_> = object
			.get_mapped_entries(&code_map, 0, "1")
			.map(|Mapped((key, value), offset)| (offset, key.offset(), value.offset()))
			.collect();

		assert_eq!(offsets, [(6, 7, 8)]);

		let offsets: Vec<_> = object
			.iter_mapped(&code_map, 0)
			.map(|Mapped((key, value), offset)| (offset, key.offset(), value.offset()))
			.collect();

		assert_eq!(offsets, [(1, 2, 3), (6, 7, 8), (15, 16, 17)]);
	}
}
