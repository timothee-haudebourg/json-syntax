use super::{Entry, Key};
use core::hash::{BuildHasher, Hash};
use hashbrown::hash_map::DefaultHashBuilder;
use hashbrown::raw::RawTable;

pub trait Equivalent<K: ?Sized> {
	fn equivalent(&self, key: &K) -> bool;
}

impl<Q: ?Sized + Eq, K: ?Sized> Equivalent<K> for Q
where
	K: std::borrow::Borrow<Q>,
{
	fn equivalent(&self, key: &K) -> bool {
		self == key.borrow()
	}
}

fn make_insert_hash<K, S>(hash_builder: &S, val: &K) -> u64
where
	K: ?Sized + Hash,
	S: BuildHasher,
{
	use core::hash::Hasher;
	let mut state = hash_builder.build_hasher();
	val.hash(&mut state);
	state.finish()
}

fn equivalent_key<'a, M, Q>(entries: &'a [Entry<M>], k: &'a Q) -> impl 'a + Fn(&Indexes) -> bool
where
	Q: ?Sized + Equivalent<Key>,
{
	move |indexes| k.equivalent(entries[indexes.rep].key.value())
}

fn make_hasher<'a, M, S>(
	entries: &'a [Entry<M>],
	hash_builder: &'a S,
) -> impl 'a + Fn(&Indexes) -> u64
where
	S: BuildHasher,
{
	move |indexes| make_hash::<S>(hash_builder, entries[indexes.rep].key.value())
}

fn make_hash<S>(hash_builder: &S, val: &Key) -> u64
where
	S: BuildHasher,
{
	use core::hash::Hasher;
	let mut state = hash_builder.build_hasher();
	val.hash(&mut state);
	state.finish()
}

#[derive(Clone)]
pub struct Indexes {
	/// Index of the first entry with the considered key (the representative).
	rep: usize,

	/// Other indexes with this key.
	other: Vec<usize>,
}

impl Indexes {
	fn new(rep: usize) -> Self {
		Self {
			rep,
			other: Vec::new(),
		}
	}

	pub fn first(&self) -> usize {
		self.rep
	}

	pub fn redundant(&self) -> Option<usize> {
		self.other.first().cloned()
	}

	pub fn redundants(&self) -> &[usize] {
		&self.other
	}

	fn insert(&mut self, mut index: usize) {
		if index != self.rep {
			if index < self.rep {
				core::mem::swap(&mut index, &mut self.rep);
			}

			if let Err(i) = self.other.binary_search(&index) {
				self.other.insert(i, index)
			}
		}
	}

	/// Removes the given index, unless it is the last remaining index.
	///
	/// Returns `true` if the index has been removed or not in the list,
	/// and `false` if it was the last index (and hence not removed).
	fn remove(&mut self, index: usize) -> bool {
		if self.rep == index {
			if self.other.is_empty() {
				false
			} else {
				self.rep = self.other.remove(1);
				true
			}
		} else {
			if let Ok(i) = self.other.binary_search(&index) {
				self.other.remove(i);
			}

			true
		}
	}

	/// Decreases all index greater than `index` by one.
	pub fn shift(&mut self, index: usize) {
		if self.rep > index {
			self.rep -= 1
		}

		for i in &mut self.other {
			if *i > index {
				*i -= 1
			}
		}
	}

	pub fn iter(&self) -> super::Indexes {
		super::Indexes::Some {
			first: Some(self.rep),
			other: self.other.iter(),
		}
	}
}

impl<'a> IntoIterator for &'a Indexes {
	type Item = usize;
	type IntoIter = super::Indexes<'a>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[derive(Clone)]
pub struct IndexMap<S = DefaultHashBuilder> {
	hash_builder: S,
	table: RawTable<Indexes>,
}

impl<S: Default> IndexMap<S> {
	fn default() -> Self {
		Self {
			hash_builder: S::default(),
			table: RawTable::default(),
		}
	}
}

impl<S: Default> IndexMap<S> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self { hash_builder: S::default(), table: RawTable::with_capacity(cap) }
	}
}

impl<S: BuildHasher> IndexMap<S> {
	pub fn get<M, Q: ?Sized>(&self, entries: &[Entry<M>], key: &Q) -> Option<&Indexes>
	where
		Q: Hash + Equivalent<Key>,
	{
		let hash = make_insert_hash(&self.hash_builder, key);
		self.table.get(hash, equivalent_key(entries, key))
	}

	/// Associates the given `key` to `index`.
	///
	/// Returns `true` if no index was already associated to the key.
	pub fn insert<M>(&mut self, entries: &[Entry<M>], index: usize) -> bool {
		let key = entries[index].key.value();
		let hash = make_insert_hash(&self.hash_builder, key);
		match self.table.get_mut(hash, equivalent_key(entries, key)) {
			Some(indexes) => {
				indexes.insert(index);
				false
			}
			None => {
				self.table.insert(
					hash,
					Indexes::new(index),
					make_hasher::<M, S>(entries, &self.hash_builder),
				);
				true
			}
		}
	}

	/// Removes the association between the given key and index.
	pub fn remove<M>(&mut self, entries: &[Entry<M>], index: usize) {
		let key = entries[index].key.value();
		let hash = make_insert_hash(&self.hash_builder, key);
		if let Some(indexes) = self.table.get_mut(hash, equivalent_key(entries, key)) {
			indexes.remove(index);
		}
	}

	/// Decreases all index greater than `index` by one everywhere in the table.
	pub fn shift(&mut self, index: usize) {
		unsafe {
			for bucket in self.table.iter() {
				let indexes = bucket.as_mut();
				indexes.shift(index)
			}
		}
	}
}
