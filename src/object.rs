use crate::code_map::{JsonCodeMapOffset, JsonMapped};
use crate::lexical::{BorrowJsonLexical, JsonLexicalEq, JsonLexicalHash, JsonLexicalPartialEq};
use crate::{JsonCodeMap, JsonFragment, JsonValue};
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
pub type Entry = (Key, JsonValue);

/// Object entry reference.
pub type EntryRef<'a> = (&'a Key, &'a JsonValue);

fn entry_ref((k, v): &Entry) -> EntryRef<'_> {
	(k, v)
}

fn get_entry_fragment<'a>(entry: &'a Entry, index: usize) -> Result<JsonFragment<'a>, usize> {
	match index {
		0 => Ok(JsonFragment::Entry(entry_ref(entry))),
		1 => Ok(JsonFragment::Key(&entry.0)),
		_ => entry.1.get_fragment(index - 2),
	}
}

/// Object entry, with code map information.
pub type MappedEntry = JsonMapped<(JsonMapped<Key>, JsonMapped<JsonValue>)>;

fn mapped_entry(
	(key, value): Entry,
	offset: usize,
	key_offset: usize,
	value_offset: usize,
) -> MappedEntry {
	JsonMapped(
		(JsonMapped(key, key_offset), JsonMapped(value, value_offset)),
		offset,
	)
}

/// Object entry reference, with code map information.
pub type MappedEntryRef<'a> = JsonMapped<(JsonMapped<&'a Key>, JsonMapped<&'a JsonValue>)>;

fn mapped_entry_ref(
	(key, value): EntryRef,
	offset: usize,
	key_offset: usize,
	value_offset: usize,
) -> MappedEntryRef {
	JsonMapped(
		(JsonMapped(key, key_offset), JsonMapped(value, value_offset)),
		offset,
	)
}

pub type IndexedMappedEntry<'a> = (usize, MappedEntryRef<'a>);

pub type IndexedMappedValue<'a> = (usize, JsonMapped<&'a JsonValue>);

/// JSON object.
///
/// # Comparison
///
/// The `PartialEq` and `PartialOrd` implementations will compare the JSON
/// objects semantically, without regard for the entries indexes. The only
/// exception is when one key has multiple values, in which cases the values are
/// ordered by index. It is possible to compare the lexical representation of
/// objects (where the entries indexes matter) by using the
/// [`BorrowJsonLexical::as_lexical`] method on both ends and comparing the
/// results.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonObject {
	map: BTreeIndexMultiMap<Key, JsonValue>,
}

impl Default for JsonObject {
	fn default() -> Self {
		Self {
			map: BTreeIndexMultiMap::new(),
		}
	}
}

impl JsonObject {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_vec(entries: Vec<(Key, JsonValue)>) -> Self {
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

	pub fn get_fragment(&self, mut index: usize) -> Result<JsonFragment<'_>, usize> {
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

	pub fn iter(&self) -> Iter<'_> {
		self.map.iter()
	}

	pub fn iter_mut(&mut self) -> IterMut<'_> {
		self.map.iter_mut()
	}

	pub fn iter_mapped<'m>(&self, code_map: &'m JsonCodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			entries: self.map.iter(),
			code_map,
			offset: offset + 1,
		}
	}

	pub fn into_iter_mapped<'m>(
		self,
		code_map: &'m JsonCodeMap,
		offset: usize,
	) -> IntoIterMapped<'m> {
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
	pub fn get<Q>(&self, key: &Q) -> Get<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get(key)
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_mut<Q>(&mut self, key: &Q) -> GetMut<'_>
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
	pub fn get_unique<Q>(&self, key: &Q) -> Result<Option<&JsonValue>, DuplicateEntry<EntryRef<'_>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.map.get_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
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
	) -> Result<Option<&mut JsonValue>, DuplicateEntry<EntryRef<'_>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.map.get_entries_mut(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => {
					let entry = (entry.0, &*entry.1);
					let duplicate = (duplicate.0, &*duplicate.1);
					Err(DuplicateEntry::new(entry, duplicate))
				}
				None => Ok(Some(entry.1)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_entries<Q>(&self, key: &Q) -> GetEntries<'_>
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
	pub fn get_unique_entry<Q>(
		&self,
		key: &Q,
	) -> Result<Option<EntryRef<'_>>, DuplicateEntry<EntryRef<'_>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_indexed<Q>(&self, key: &Q) -> GetIndexed<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get_indexed(key)
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(log(n))` (average).
	pub fn get_indexed_entries<Q>(&self, key: &Q) -> GetIndexedEntries<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.get_indexed_entries(key)
	}

	/// Returns the (first) value associated to `key`, or insert a `key`-`value`
	/// entry where `value` is returned by the given function `f`.
	pub fn get_or_insert_with<Q>(
		&mut self,
		key: impl Into<Key>,
		f: impl FnOnce() -> JsonValue,
	) -> Get<'_> {
		self.map.get_or_insert_with(key.into(), f)
	}

	/// Returns a mutable reference to the (first) value associated to `key`, or
	/// insert a `key`-`value` entry where `value` is returned by the given
	/// function `f`.
	pub fn get_or_insert_mut_with<Q>(
		&mut self,
		key: impl Into<Key>,
		f: impl FnOnce() -> JsonValue,
	) -> GetMut<'_> {
		self.map.get_or_insert_mut_with(key.into(), f)
	}

	pub fn index_of<Q>(&self, key: &Q) -> Option<usize>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.index_of(key)
	}

	pub fn indexes_of<Q>(&self, key: &Q) -> IndexesIter<'_>
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
		code_map: &'m JsonCodeMap,
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
		code_map: &JsonCodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<MappedEntryRef<'_>>, DuplicateEntry<MappedEntryRef<'_>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_mapped_entries(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
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
		code_map: &'m JsonCodeMap,
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
		code_map: &JsonCodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<IndexedMappedEntry<'_>>, DuplicateEntry<IndexedMappedEntry<'_>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_mapped_entries_with_index(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
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
		code_map: &'m JsonCodeMap,
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
		code_map: &JsonCodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<JsonMapped<&JsonValue>>, DuplicateEntry<JsonMapped<&JsonValue>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_mapped(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
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
		code_map: &'m JsonCodeMap,
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
		code_map: &JsonCodeMap,
		offset: usize,
		key: &Q,
	) -> Result<Option<IndexedMappedValue<'_>>, DuplicateEntry<IndexedMappedValue<'_>>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		let mut entries = self.get_indexed_mapped(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	pub fn first(&self) -> Option<EntryRef<'_>> {
		self.map.first()
	}

	pub fn last(&self) -> Option<EntryRef<'_>> {
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
	pub fn push_back(&mut self, key: impl Into<Key>, value: JsonValue) -> (usize, bool) {
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
	pub fn push_front(&mut self, key: impl Into<Key>, value: JsonValue) -> bool {
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
	pub fn shift_insert(&mut self, key: impl Into<Key>, value: JsonValue) -> ShiftInsert<'_> {
		self.map.shift_insert(key.into(), value)
	}

	pub fn swap_insert(&mut self, key: impl Into<Key>, value: JsonValue) -> SwapInsert<'_> {
		self.map.swap_insert(key.into(), value)
	}

	/// Inserts the given key-value pair on top of the object.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	pub fn shift_insert_front(
		&mut self,
		key: impl Into<Key>,
		value: JsonValue,
	) -> ShiftInsertFront<'_> {
		self.map.shift_insert_front(key.into(), value)
	}

	pub fn swap_insert_front(
		&mut self,
		key: impl Into<Key>,
		value: JsonValue,
	) -> SwapInsertFront<'_> {
		self.map.swap_insert_front(key.into(), value)
	}

	pub fn shift_insert_back_full(
		&mut self,
		key: impl Into<Key>,
		value: JsonValue,
	) -> (usize, ShiftInsertBack<'_>) {
		self.map.shift_insert_back_full(key.into(), value)
	}

	pub fn shift_insert_back(
		&mut self,
		key: impl Into<Key>,
		value: JsonValue,
	) -> ShiftInsertBack<'_> {
		self.map.shift_insert_back(key.into(), value)
	}

	/// Alias of [`Self::shift_insert_back`].
	pub fn insert(&mut self, key: impl Into<Key>, value: JsonValue) -> ShiftInsertBack<'_> {
		self.shift_insert_back(key, value)
	}

	pub fn swap_insert_back_full(
		&mut self,
		key: impl Into<Key>,
		value: JsonValue,
	) -> (usize, SwapInsertBack<'_>) {
		self.map.swap_insert_back_full(key.into(), value)
	}

	pub fn swap_insert_back(
		&mut self,
		key: impl Into<Key>,
		value: JsonValue,
	) -> SwapInsertBack<'_> {
		self.map.swap_insert_back(key.into(), value)
	}

	pub fn shift_remove_indexed_entries<Q>(&mut self, key: &Q) -> ShiftRemoveIndexedEntries<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.shift_remove_indexed_entries(key)
	}

	/// Remove all entries associated to the given key.
	pub fn shift_remove_entries<Q>(&mut self, key: &Q) -> ShiftRemoveEntries<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.shift_remove_entries(key)
	}

	/// Remove all entries associated to the given key.
	pub fn shift_remove<Q>(&mut self, key: &Q) -> ShiftRemove<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.shift_remove(key)
	}

	/// Alias of [`Self::shift_remove`].
	pub fn remove<Q>(&mut self, key: &Q) -> ShiftRemove<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.shift_remove(key)
	}

	/// Alias of [`Self::shift_remove_unique`].
	pub fn remove_unique<Q>(&mut self, key: &Q) -> Result<Option<Entry>, DuplicateEntry<Entry>>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.shift_remove_unique(key)
	}

	pub fn swap_remove_indexed_entries<Q>(&mut self, key: &Q) -> SwapRemoveIndexedEntries<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.swap_remove_indexed_entries(key)
	}

	pub fn swap_remove_entries<Q>(&mut self, key: &Q) -> SwapRemoveEntries<'_>
	where
		Q: ?Sized + Comparable<Key>,
	{
		self.map.swap_remove_entries(key)
	}

	pub fn swap_remove<Q>(&mut self, key: &Q) -> SwapRemove<'_>
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
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
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
				Some(duplicate) => Err(DuplicateEntry::new(entry, duplicate)),
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

pub type Iter<'a> = btree_indexmap::multi_map::Iter<'a, Key, JsonValue>;

pub type IterMut<'a> = btree_indexmap::multi_map::IterMut<'a, Key, JsonValue>;

pub type IntoIter = btree_indexmap::multi_map::IntoIter<Key, JsonValue>;

pub type Get<'a> = btree_indexmap::multi_map::Get<'a, Key, JsonValue>;

pub type GetIndexed<'a> = btree_indexmap::multi_map::GetIndexed<'a, Key, JsonValue>;

pub type GetEntries<'a> = btree_indexmap::multi_map::GetEntries<'a, Key, JsonValue>;

pub type GetIndexedEntries<'a> = btree_indexmap::multi_map::GetIndexedEntries<'a, Key, JsonValue>;

pub type GetMut<'a> = btree_indexmap::multi_map::GetMut<'a, Key, JsonValue>;

pub type ShiftInsert<'a> = btree_indexmap::multi_map::ShiftInsert<'a, Key, JsonValue>;

pub type SwapInsert<'a> = btree_indexmap::multi_map::SwapInsert<'a, Key, JsonValue>;

pub type ShiftInsertBack<'a> = btree_indexmap::multi_map::ShiftInsertBack<'a, Key, JsonValue>;

pub type SwapInsertBack<'a> = btree_indexmap::multi_map::SwapInsertBack<'a, Key, JsonValue>;

pub type ShiftInsertFront<'a> = btree_indexmap::multi_map::ShiftInsertFront<'a, Key, JsonValue>;

pub type SwapInsertFront<'a> = btree_indexmap::multi_map::SwapInsertFront<'a, Key, JsonValue>;

pub type ShiftRemoveIndexedEntries<'a> =
	btree_indexmap::multi_map::ShiftRemoveIndexedEntries<'a, Key, JsonValue>;

pub type ShiftRemoveEntries<'a> = btree_indexmap::multi_map::ShiftRemoveEntries<'a, Key, JsonValue>;

pub type ShiftRemove<'a> = btree_indexmap::multi_map::ShiftRemove<'a, Key, JsonValue>;

pub type SwapRemoveIndexedEntries<'a> =
	btree_indexmap::multi_map::SwapRemoveIndexedEntries<'a, Key, JsonValue>;

pub type SwapRemoveEntries<'a> = btree_indexmap::multi_map::SwapRemoveEntries<'a, Key, JsonValue>;

pub type SwapRemove<'a> = btree_indexmap::multi_map::SwapRemove<'a, Key, JsonValue>;

pub struct IterMapped<'a, 'm> {
	entries: Iter<'a>,
	code_map: &'m JsonCodeMap,
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
	code_map: &'m JsonCodeMap,
	offset: JsonCodeMapOffset,
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
				object: &$lft JsonObject,
				code_map: &'m JsonCodeMap,
				offset: JsonCodeMapOffset,
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
		type Item = JsonMapped<&'a JsonValue>;

		fn next(&mut self, index) {
			JsonMapped(
				&self.object.entries()[index].1,
				self.offset+2,
			)
		}
	}

	GetIndexedMapped<'a> {
		type Item = (usize, JsonMapped<&'a JsonValue>);

		fn next(&mut self, index) {
			(
				index,
				JsonMapped(
					&self.object.entries()[index].1,
					self.offset+2
				)
			)
		}
	}
}

impl BorrowJsonLexical for JsonObject {}

impl JsonLexicalPartialEq for JsonObject {
	fn lexical_eq(&self, other: &Self) -> bool {
		self.map.as_entries().eq(other.map.as_entries())
	}
}

impl JsonLexicalEq for JsonObject {}

impl JsonLexicalHash for JsonObject {
	fn lexical_hash<H: Hasher>(&self, state: &mut H) {
		self.map.as_entries().hash(state)
	}
}

impl fmt::Debug for JsonObject {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map().entries(self.map.iter()).finish()
	}
}

impl From<Vec<Entry>> for JsonObject {
	fn from(entries: Vec<Entry>) -> Self {
		Self::from_vec(entries)
	}
}

impl<'a> IntoIterator for &'a JsonObject {
	type Item = EntryRef<'a>;
	type IntoIter = Iter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a> IntoIterator for &'a mut JsonObject {
	type Item = (&'a Key, &'a mut JsonValue);
	type IntoIter = IterMut<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl IntoIterator for JsonObject {
	type Item = Entry;
	type IntoIter = std::vec::IntoIter<Entry>;

	fn into_iter(self) -> Self::IntoIter {
		self.map.into_iter()
	}
}

impl Extend<Entry> for JsonObject {
	fn extend<I: IntoIterator<Item = Entry>>(&mut self, iter: I) {
		for entry in iter {
			self.push_entry_back(entry);
		}
	}
}

impl FromIterator<Entry> for JsonObject {
	fn from_iter<I: IntoIterator<Item = Entry>>(iter: I) -> Self {
		let mut object = JsonObject::default();
		object.extend(iter);
		object
	}
}

/// Duplicate entry error.
#[derive(Debug)]
pub struct DuplicateEntry<T = Entry>(pub Box<T>, pub Box<T>);

impl<T> DuplicateEntry<T> {
	pub fn new(a: T, b: T) -> Self {
		Self(Box::new(a), Box::new(b))
	}
}

impl fmt::Display for DuplicateEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "duplicate entry `{}`", self.0.0)
	}
}

pub type DuplicateEntryRef<'a> = DuplicateEntry<EntryRef<'a>>;

impl std::error::Error for DuplicateEntry {}

#[cfg(test)]
mod tests {
	use crate::lexical::BorrowJsonLexical;

	use super::*;

	#[test]
	fn remove() {
		let mut object = JsonObject::new();
		object.insert("a", JsonValue::Null);

		object.remove("a");
		object.remove("a");
	}

	#[test]
	fn unordered_eq1() {
		let mut a = JsonObject::new();
		a.push_back("a", JsonValue::Null);
		a.push_back("b", JsonValue::Null);

		let mut b = JsonObject::new();
		b.push_back("b", JsonValue::Null);
		b.push_back("a", JsonValue::Null);

		assert_eq!(a, b);
		assert_ne!(a.as_lexical(), b.as_lexical())
	}

	#[test]
	fn unordered_eq2() {
		let mut a = JsonObject::new();
		a.push_back("a", JsonValue::Null);
		a.push_back("a", JsonValue::Null);

		let mut b = JsonObject::new();
		b.push_back("a", JsonValue::Null);
		b.push_back("a", JsonValue::Null);

		assert_eq!(a, b);
		assert_eq!(a.as_lexical(), b.as_lexical())
	}

	#[test]
	fn insert_front1() {
		let mut a = JsonObject::new();
		a.push_back("a", JsonValue::Null);
		a.push_back("b", JsonValue::Null);
		a.push_back("c", JsonValue::Null);
		a.shift_insert_front("b", JsonValue::Null);

		let mut b = JsonObject::new();
		b.push_back("b", JsonValue::Null);
		b.push_back("a", JsonValue::Null);
		b.push_back("c", JsonValue::Null);

		assert_eq!(a, b);
	}

	#[test]
	fn insert_front2() {
		let mut a = JsonObject::new();
		a.push_back("a", JsonValue::Null);
		a.push_back("a", JsonValue::Null);
		a.push_back("c", JsonValue::Null);
		a.shift_insert_front("a", JsonValue::Null);

		let mut b = JsonObject::new();
		b.push_back("a", JsonValue::Null);
		b.push_back("c", JsonValue::Null);

		assert_eq!(a, b);
	}

	#[test]
	fn mapped_entries() {
		use crate::ParseJson;
		let (json, code_map) = crate::JsonValue::parse_str(
			r#"{ "0": [null, null], "1": { "foo": 0, "bar": 1 }, "0": null }"#,
		)
		.unwrap();
		let object = json.into_object().unwrap();

		let offsets: Vec<_> = object
			.get_mapped_entries(&code_map, 0, "0")
			.map(|JsonMapped((key, value), offset)| (offset, key.offset(), value.offset()))
			.collect();

		assert_eq!(offsets, [(1, 2, 3), (15, 16, 17)]);

		let offsets: Vec<_> = object
			.get_mapped_entries(&code_map, 0, "1")
			.map(|JsonMapped((key, value), offset)| (offset, key.offset(), value.offset()))
			.collect();

		assert_eq!(offsets, [(6, 7, 8)]);

		let offsets: Vec<_> = object
			.iter_mapped(&code_map, 0)
			.map(|JsonMapped((key, value), offset)| (offset, key.offset(), value.offset()))
			.collect();

		assert_eq!(offsets, [(1, 2, 3), (6, 7, 8), (15, 16, 17)]);
	}
}
