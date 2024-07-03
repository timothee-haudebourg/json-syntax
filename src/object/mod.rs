use crate::code_map::Mapped;
use crate::{CodeMap, FragmentRef, UnorderedEq, UnorderedPartialEq, Value};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Entry<K = Key, V = Value> {
	pub key: K,
	pub value: V,
}

impl<K, V> Entry<K, V> {
	pub fn new(key: K, value: V) -> Self {
		Self { key, value }
	}

	pub fn as_key(&self) -> &K {
		&self.key
	}

	pub fn as_value(&self) -> &V {
		&self.value
	}

	pub fn into_key(self) -> K {
		self.key
	}

	pub fn into_value(self) -> V {
		self.value
	}

	pub fn as_pair(&self) -> (&K, &V) {
		(&self.key, &self.value)
	}

	pub fn into_pair(self) -> (K, V) {
		(self.key, self.value)
	}

	pub fn as_ref(&self) -> Entry<&K, &V> {
		Entry::new(&self.key, &self.value)
	}

	pub fn into_mapped(
		self,
		key_offset: usize,
		value_offset: usize,
	) -> Entry<Mapped<K>, Mapped<V>> {
		Entry::new(
			Mapped::new(key_offset, self.key),
			Mapped::new(value_offset, self.value),
		)
	}
}

impl Entry {
	pub fn get_fragment(&self, index: usize) -> Result<FragmentRef, usize> {
		match index {
			0 => Ok(FragmentRef::Entry(self)),
			1 => Ok(FragmentRef::Key(&self.key)),
			_ => self.value.get_fragment(index - 2),
		}
	}
}

pub type MappedEntry<'a> = Mapped<Entry<Mapped<&'a Key>, Mapped<&'a Value>>>;

pub type IndexedMappedEntry<'a> = (usize, MappedEntry<'a>);

pub type IndexedMappedValue<'a> = (usize, Mapped<&'a Value>);

/// Object.
#[derive(Clone)]
pub struct Object {
	/// The entries of the object, in order.
	entries: Vec<Entry>,

	/// Maps each key to an entry index.
	indexes: IndexMap,
}

impl Default for Object {
	fn default() -> Self {
		Self {
			entries: Vec::new(),
			indexes: IndexMap::new(),
		}
	}
}

impl Object {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from_vec(entries: Vec<Entry>) -> Self {
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

	pub fn get_fragment(&self, mut index: usize) -> Result<FragmentRef, usize> {
		for e in &self.entries {
			match e.get_fragment(index) {
				Ok(value) => return Ok(value),
				Err(i) => index = i,
			}
		}

		Err(index)
	}

	pub fn entries(&self) -> &[Entry] {
		&self.entries
	}

	pub fn iter(&self) -> Iter {
		self.entries.iter()
	}

	pub fn iter_mut(&mut self) -> IterMut {
		IterMut(self.entries.iter_mut())
	}

	pub fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			entries: self.entries.iter(),
			code_map,
			offset: offset + 1,
		}
	}

	/// Checks if this object contains the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn contains_key<Q>(&self, key: &Q) -> bool
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		self.indexes.get(&self.entries, key).is_some()
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get<Q>(&self, key: &Q) -> Values
	where
		Q: ?Sized + Hash + Equivalent<Key>,
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

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_mut<Q>(&mut self, key: &Q) -> ValuesMut
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		ValuesMut {
			indexes,
			entries: &mut self.entries,
		}
	}

	/// Returns the unique entry value matching the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_unique<Q>(&self, key: &Q) -> Result<Option<&Value>, Duplicate<&Entry>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.get_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
				None => Ok(Some(&entry.value)),
			},
			None => Ok(None),
		}
	}

	/// Returns the unique entry value matching the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_unique_mut<Q>(&mut self, key: &Q) -> Result<Option<&mut Value>, Duplicate<&Entry>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let index = {
			let mut entries = self.get_entries_with_index(key);
			match entries.next() {
				Some((i, _)) => match entries.next() {
					Some((j, _)) => Err(Duplicate(i, j)),
					None => Ok(Some(i)),
				},
				None => Ok(None),
			}
		};

		match index {
			Ok(Some(i)) => Ok(Some(&mut self.entries[i].value)),
			Ok(None) => Ok(None),
			Err(Duplicate(i, j)) => Err(Duplicate(&self.entries[i], &self.entries[j])),
		}
	}

	/// Returns an iterator over the entries matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_entries<Q>(&self, key: &Q) -> Entries
	where
		Q: ?Sized + Hash + Equivalent<Key>,
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

	/// Returns the unique entry matching the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_unique_entry<Q>(&self, key: &Q) -> Result<Option<&Entry>, Duplicate<&Entry>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.get_entries(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Returns an iterator over the values matching the given key.
	///
	/// Runs in `O(1)` (average).
	pub fn get_with_index<Q>(&self, key: &Q) -> ValuesWithIndex
	where
		Q: ?Sized + Hash + Equivalent<Key>,
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
	pub fn get_entries_with_index<Q>(&self, key: &Q) -> EntriesWithIndex
	where
		Q: ?Sized + Hash + Equivalent<Key>,
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

	/// Returns the (first) value associated to `key`, or insert a `key`-`value`
	/// entry where `value` is returned by the given function `f`.
	pub fn get_or_insert_with<Q>(&mut self, key: &Q, f: impl FnOnce() -> Value) -> &Value
	where
		Q: ?Sized + Hash + Equivalent<Key> + ToOwned,
		Q::Owned: Into<Key>,
	{
		let index = match self.index_of(key) {
			Some(index) => index,
			None => {
				let index = self.entries.len();
				self.push(key.to_owned().into(), f());
				index
			}
		};

		&self.entries[index].value
	}

	/// Returns a mutable reference to the (first) value associated to `key`, or
	/// insert a `key`-`value` entry where `value` is returned by the given
	/// function `f`.
	pub fn get_mut_or_insert_with<Q>(&mut self, key: &Q, f: impl FnOnce() -> Value) -> &mut Value
	where
		Q: ?Sized + Hash + Equivalent<Key> + ToOwned,
		Q::Owned: Into<Key>,
	{
		let index = match self.index_of(key) {
			Some(index) => index,
			None => {
				let index = self.entries.len();
				self.push(key.to_owned().into(), f());
				index
			}
		};

		&mut self.entries[index].value
	}

	pub fn index_of<Q>(&self, key: &Q) -> Option<usize>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		self.indexes
			.get(&self.entries, key)
			.map(index_map::Indexes::first)
	}

	pub fn redundant_index_of<Q>(&self, key: &Q) -> Option<usize>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		self.indexes
			.get(&self.entries, key)
			.and_then(index_map::Indexes::redundant)
	}

	pub fn indexes_of<Q>(&self, key: &Q) -> Indexes
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		self.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default()
	}

	/// Returns an iterator over the mapped entries matching the given key.
	///
	/// Runs in `O(n)` (average). `O(1)` to find the entry, `O(n)` to compute
	/// the entry fragment offset.
	pub fn get_mapped_entries<'m, Q>(
		&self,
		code_map: &'m CodeMap,
		offset: usize,
		key: &Q,
	) -> MappedEntries<'_, 'm>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		MappedEntries {
			indexes,
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
	) -> Result<Option<MappedEntry>, Duplicate<MappedEntry>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.get_mapped_entries(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
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
	) -> MappedEntriesWithIndex<'_, 'm>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		MappedEntriesWithIndex {
			indexes,
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
	) -> Result<Option<IndexedMappedEntry>, Duplicate<IndexedMappedEntry>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.get_mapped_entries_with_index(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
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
	) -> MappedValues<'_, 'm>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		MappedValues {
			indexes,
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
	) -> Result<Option<Mapped<&Value>>, Duplicate<Mapped<&Value>>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.get_mapped(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
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
	pub fn get_mapped_with_index<'m, Q>(
		&self,
		code_map: &'m CodeMap,
		offset: usize,
		key: &Q,
	) -> MappedValuesWithIndex<'_, 'm>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let indexes = self
			.indexes
			.get(&self.entries, key)
			.map(IntoIterator::into_iter)
			.unwrap_or_default();
		MappedValuesWithIndex {
			indexes,
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
	) -> Result<Option<IndexedMappedValue>, Duplicate<IndexedMappedValue>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.get_mapped_with_index(code_map, offset, key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	pub fn first(&self) -> Option<&Entry> {
		self.entries.first()
	}

	pub fn last(&self) -> Option<&Entry> {
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
	pub fn push(&mut self, key: Key, value: Value) -> bool {
		self.push_entry(Entry::new(key, value))
	}

	pub fn push_entry(&mut self, entry: Entry) -> bool {
		let index = self.entries.len();
		self.entries.push(entry);
		self.indexes.insert(&self.entries, index)
	}

	/// Push the given key-value pair to the top of the object.
	///
	/// Returns `true` if the key was not already present in the object,
	/// and `false` otherwise.
	/// Any previous entry matching the key is **not** overridden: duplicates
	/// are preserved, in order.
	///
	/// Runs in `O(n)`.
	pub fn push_front(&mut self, key: Key, value: Value) -> bool {
		self.push_entry_front(Entry::new(key, value))
	}

	pub fn push_entry_front(&mut self, entry: Entry) -> bool {
		self.entries.insert(0, entry);
		self.indexes.shift_up(0);
		self.indexes.insert(&self.entries, 0)
	}

	/// Removes the entry at the given index.
	pub fn remove_at(&mut self, index: usize) -> Option<Entry> {
		if index < self.entries.len() {
			self.indexes.remove(&self.entries, index);
			self.indexes.shift_down(index);
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
	pub fn insert(&mut self, key: Key, value: Value) -> Option<RemovedByInsertion> {
		match self.index_of(&key) {
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

	/// Inserts the given key-value pair on top of the object.
	///
	/// If one or more entries are already matching the given key,
	/// all of them are removed and returned in the resulting iterator.
	pub fn insert_front(&mut self, key: Key, value: Value) -> RemovedByInsertFront {
		if let Some(first) = self.entries.first_mut() {
			if first.key == key {
				let first = core::mem::replace(first, Entry::new(key, value));
				return RemovedByInsertFront {
					first: Some(first),
					object: self,
				};
			}
		}

		self.push_front(key, value);
		RemovedByInsertFront {
			first: None,
			object: self,
		}
	}

	/// Remove all entries associated to the given key.
	///
	/// Runs in `O(n)` time (average).
	pub fn remove<'q, Q>(&mut self, key: &'q Q) -> RemovedEntries<'_, 'q, Q>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		RemovedEntries { key, object: self }
	}

	/// Remove the unique entry associated to the given key.
	///
	/// Returns an error if multiple entries match the key.
	///
	/// Runs in `O(n)` time (average).
	pub fn remove_unique<Q>(&mut self, key: &Q) -> Result<Option<Entry>, Duplicate<Entry>>
	where
		Q: ?Sized + Hash + Equivalent<Key>,
	{
		let mut entries = self.remove(key);

		match entries.next() {
			Some(entry) => match entries.next() {
				Some(duplicate) => Err(Duplicate(entry, duplicate)),
				None => Ok(Some(entry)),
			},
			None => Ok(None),
		}
	}

	/// Sort the entries by key name.
	///
	/// Entries with the same key are sorted by value.
	pub fn sort(&mut self) {
		use locspan::BorrowStripped;
		self.entries.sort_by(|a, b| a.stripped().cmp(b.stripped()));
		self.indexes.clear();

		for i in 0..self.entries.len() {
			self.indexes.insert(&self.entries, i);
		}
	}

	/// Puts this JSON object in canonical form according to
	/// [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785#name-generation-of-canonical-jso).
	///
	/// This will canonicalize the entries and sort them by key.
	/// Entries with the same key are sorted by value.
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

pub type Iter<'a> = core::slice::Iter<'a, Entry>;

pub struct IterMut<'a>(std::slice::IterMut<'a, Entry>);

impl<'a> Iterator for IterMut<'a> {
	type Item = (&'a Key, &'a mut Value);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|entry| (&entry.key, &mut entry.value))
	}
}

pub struct IterMapped<'a, 'm> {
	entries: std::slice::Iter<'a, Entry>,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'a, 'm> Iterator for IterMapped<'a, 'm> {
	type Item = MappedEntry<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.entries.next().map(|e| {
			let offset = self.offset;
			self.offset += 2 + self.code_map.get(self.offset + 2).unwrap().volume;
			Mapped::new(offset, e.as_ref().into_mapped(offset + 1, offset + 2))
		})
	}
}

impl PartialEq for Object {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl Eq for Object {}

impl UnorderedPartialEq for Object {
	fn unordered_eq(&self, other: &Self) -> bool {
		if self.entries.len() != other.entries.len() {
			return false;
		}

		if !self.iter().all(|Entry { key, value: a }| {
			other
				.get_entries(key)
				.any(|Entry { value: b, .. }| a.unordered_eq(b))
		}) {
			return false;
		}

		if self.indexes.contains_duplicate_keys()
			&& !other.iter().all(
				|Entry {
				     key: other_key,
				     value: b,
				 }| {
					self.get_entries(other_key)
						.any(|Entry { value: a, .. }| a.unordered_eq(b))
				},
			) {
			return false;
		}

		true
	}
}

impl UnorderedEq for Object {}

impl PartialOrd for Object {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.entries.cmp(&other.entries))
	}
}

impl Ord for Object {
	fn cmp(&self, other: &Self) -> Ordering {
		self.entries.cmp(&other.entries)
	}
}

impl Hash for Object {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.entries.hash(state)
	}
}

impl fmt::Debug for Object {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_map()
			.entries(self.entries.iter().map(Entry::as_pair))
			.finish()
	}
}

impl From<Vec<Entry>> for Object {
	fn from(entries: Vec<Entry>) -> Self {
		Self::from_vec(entries)
	}
}

impl<'a> IntoIterator for &'a Object {
	type Item = &'a Entry;
	type IntoIter = core::slice::Iter<'a, Entry>;

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
		self.entries.into_iter()
	}
}

impl Extend<Entry> for Object {
	fn extend<I: IntoIterator<Item = Entry>>(&mut self, iter: I) {
		for entry in iter {
			self.push_entry(entry);
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

impl Extend<(Key, Value)> for Object {
	fn extend<I: IntoIterator<Item = (Key, Value)>>(&mut self, iter: I) {
		for (key, value) in iter {
			self.push(key, value);
		}
	}
}

impl FromIterator<(Key, Value)> for Object {
	fn from_iter<I: IntoIterator<Item = (Key, Value)>>(iter: I) -> Self {
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
			pub struct $id<$lft> {
				indexes: Indexes<$lft>,
				object: &$lft Object
			}

			impl<$lft> Iterator for $id<$lft> {
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
		type Item = &'a Value;

		fn next(&mut self, index) { &self.object.entries[index].value }
	}

	ValuesWithIndex<'a> {
		type Item = (usize, &'a Value);

		fn next(&mut self, index) { (index, &self.object.entries[index].value) }
	}

	Entries<'a> {
		type Item = &'a Entry;

		fn next(&mut self, index) { &self.object.entries[index] }
	}

	EntriesWithIndex<'a> {
		type Item = (usize, &'a Entry);

		fn next(&mut self, index) { (index, &self.object.entries[index]) }
	}
}

macro_rules! entries_iter_mut {
	($($id:ident <$lft:lifetime> {
		type Item = $item:ty ;

		fn next(&mut $self:ident, $index:ident) { $e:expr }
	})*) => {
		$(
			pub struct $id<$lft> {
				indexes: Indexes<$lft>,
				entries: &$lft mut [Entry]
			}

			impl<$lft> Iterator for $id<$lft> {
				type Item = $item;

				fn next(&mut $self) -> Option<Self::Item> {
					$self.indexes.next().map(|$index| $e)
				}
			}
		)*
	};
}

entries_iter_mut! {
	ValuesMut<'a> {
		type Item = &'a mut Value;

		fn next(&mut self, index) {
			// This is safe because there is no aliasing between the values.
			unsafe { core::mem::transmute(&mut self.entries[index].value) }
		}
	}

	ValuesMutWithIndex<'a> {
		type Item = (usize, &'a mut Value);

		fn next(&mut self, index) {
			// This is safe because there is no aliasing between the values.
			unsafe { (index, core::mem::transmute::<&mut Value, &'a mut Value>(&mut self.entries[index].value)) }
		}
	}
}

macro_rules! mapped_entries_iter {
	($($id:ident <$lft:lifetime> {
		type Item = $item:ty ;

		fn next(&mut $self:ident, $index:ident) { $e:expr }
	})*) => {
		$(
			pub struct $id<$lft, 'm> {
				indexes: Indexes<$lft>,
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
	MappedEntries<'a> {
		type Item = MappedEntry<'a>;

		fn next(&mut self, index) {
			Mapped::new(
				self.offset,
				self.object.entries[index].as_ref().into_mapped(
					self.offset+1,
					self.offset+2
				)
			)
		}
	}

	MappedEntriesWithIndex<'a> {
		type Item = (usize, MappedEntry<'a>);

		fn next(&mut self, index) {
			(
				index,
				Mapped::new(
					self.offset,
					self.object.entries[index].as_ref().into_mapped(
						self.offset+1,
						self.offset+2
					)
				)
			)
		}
	}

	MappedValues<'a> {
		type Item = Mapped<&'a Value>;

		fn next(&mut self, index) {
			Mapped::new(
				self.offset+2,
				&self.object.entries[index].value
			)
		}
	}

	MappedValuesWithIndex<'a> {
		type Item = (usize, Mapped<&'a Value>);

		fn next(&mut self, index) {
			(
				index,
				Mapped::new(
					self.offset+2,
					&self.object.entries[index].value
				)
			)
		}
	}
}

pub struct RemovedByInsertion<'a> {
	index: usize,
	first: Option<Entry>,
	object: &'a mut Object,
}

impl<'a> Iterator for RemovedByInsertion<'a> {
	type Item = Entry;

	fn next(&mut self) -> Option<Self::Item> {
		match self.first.take() {
			Some(entry) => Some(entry),
			None => {
				let key = &self.object.entries[self.index].key;
				self.object
					.redundant_index_of(key)
					.and_then(|index| self.object.remove_at(index))
			}
		}
	}
}

impl<'a> Drop for RemovedByInsertion<'a> {
	fn drop(&mut self) {
		self.last();
	}
}

pub struct RemovedByInsertFront<'a> {
	first: Option<Entry>,
	object: &'a mut Object,
}

impl<'a> Iterator for RemovedByInsertFront<'a> {
	type Item = Entry;

	fn next(&mut self) -> Option<Self::Item> {
		match self.first.take() {
			Some(entry) => Some(entry),
			None => {
				let key = &self.object.entries[0].key;
				self.object
					.redundant_index_of(key)
					.and_then(|index| self.object.remove_at(index))
			}
		}
	}
}

impl<'a> Drop for RemovedByInsertFront<'a> {
	fn drop(&mut self) {
		self.last();
	}
}

pub struct RemovedEntries<'a, 'q, Q: ?Sized>
where
	Q: Hash + Equivalent<Key>,
{
	key: &'q Q,
	object: &'a mut Object,
}

impl<'a, 'q, Q: ?Sized> Iterator for RemovedEntries<'a, 'q, Q>
where
	Q: Hash + Equivalent<Key>,
{
	type Item = Entry;

	fn next(&mut self) -> Option<Self::Item> {
		self.object
			.index_of(self.key)
			.and_then(|index| self.object.remove_at(index))
	}
}

impl<'a, 'q, Q: ?Sized> Drop for RemovedEntries<'a, 'q, Q>
where
	Q: Hash + Equivalent<Key>,
{
	fn drop(&mut self) {
		self.last();
	}
}

#[derive(Debug)]
pub struct Duplicate<T>(pub T, pub T);

/// Duplicate entry error.
pub type DuplicateEntry = Duplicate<Entry>;

impl fmt::Display for DuplicateEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "duplicate entry `{}`", self.0.key)
	}
}

impl std::error::Error for DuplicateEntry {}

#[cfg(test)]
mod tests {
	use crate::BorrowUnordered;

	use super::*;

	#[test]
	fn remove() {
		let mut object = Object::new();
		object.insert("a".into(), Value::Null);

		object.remove("a");
		object.remove("a");
	}

	#[test]
	fn unordered_eq1() {
		let mut a = Object::new();
		a.push("a".into(), Value::Null);
		a.push("b".into(), Value::Null);

		let mut b = Object::new();
		b.push("b".into(), Value::Null);
		b.push("a".into(), Value::Null);

		assert_ne!(a, b);
		assert_eq!(a.as_unordered(), b.as_unordered())
	}

	#[test]
	fn unordered_eq2() {
		let mut a = Object::new();
		a.push("a".into(), Value::Null);
		a.push("a".into(), Value::Null);

		let mut b = Object::new();
		b.push("a".into(), Value::Null);
		b.push("a".into(), Value::Null);

		assert_eq!(a, b);
		assert_eq!(a.as_unordered(), b.as_unordered())
	}

	#[test]
	fn insert_front1() {
		let mut a = Object::new();
		a.push("a".into(), Value::Null);
		a.push("b".into(), Value::Null);
		a.push("c".into(), Value::Null);
		a.insert_front("b".into(), Value::Null);

		let mut b = Object::new();
		b.push("b".into(), Value::Null);
		b.push("a".into(), Value::Null);
		b.push("c".into(), Value::Null);

		assert_eq!(a, b);
	}

	#[test]
	fn insert_front2() {
		let mut a = Object::new();
		a.push("a".into(), Value::Null);
		a.push("a".into(), Value::Null);
		a.push("c".into(), Value::Null);
		a.insert_front("a".into(), Value::Null);

		let mut b = Object::new();
		b.push("a".into(), Value::Null);
		b.push("c".into(), Value::Null);

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
			.map(|e| (e.offset, e.value.key.offset, e.value.value.offset))
			.collect();

		assert_eq!(offsets, [(1, 2, 3), (15, 16, 17)]);

		let offsets: Vec<_> = object
			.get_mapped_entries(&code_map, 0, "1")
			.map(|e| (e.offset, e.value.key.offset, e.value.value.offset))
			.collect();

		assert_eq!(offsets, [(6, 7, 8)]);

		let offsets: Vec<_> = object
			.iter_mapped(&code_map, 0)
			.map(|e| (e.offset, e.value.key.offset, e.value.value.offset))
			.collect();

		assert_eq!(offsets, [(1, 2, 3), (6, 7, 8), (15, 16, 17)]);
	}
}
