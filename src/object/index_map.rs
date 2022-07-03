use core::hash::{Hash, BuildHasher};
use hashbrown::raw::RawTable;
use hashbrown::hash_map::DefaultHashBuilder;
use super::{Key, Entry};

pub trait Equivalent<Q: ?Sized> {
	fn equivalent(&self, q: &Q) -> bool;
}

impl<T: ?Sized + Eq, Q: ?Sized> Equivalent<Q> for T where Q: std::borrow::Borrow<T> {
	fn equivalent(&self, q: &Q) -> bool {
		self == q.borrow()
	}
}

fn make_insert_hash<K, S>(hash_builder: &S, val: &K) -> u64
where
	K: Hash,
	S: BuildHasher,
{
	use core::hash::Hasher;
	let mut state = hash_builder.build_hasher();
	val.hash(&mut state);
	state.finish()
}

fn equivalent_key<'a, M, Q>(entries: &'a [Entry<M>], k: &'a Q) -> impl 'a + Fn(&Indexes) -> bool
where
	Q: ?Sized + Eq + Equivalent<Key>
{
	move |indexes| k.equivalent(entries[indexes.rep].key.value())
}

fn make_hasher<'a, M, S>(entries: &'a [Entry<M>], hash_builder: &'a S) -> impl 'a + Fn(&Indexes) -> u64
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

struct Indexes {
	/// Index of the first entry with the considered key (the representative).
	rep: usize,

	/// Other indexes with this key.
	other: Vec<usize>
}

impl Indexes {
	fn new(rep: usize) -> Self {
		Self {
			rep,
			other: Vec::new()
		}
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
}

pub struct IndexMap<S = DefaultHashBuilder> {
	hash_builder: S,
	table: RawTable<Indexes>
}

impl<S: Default> IndexMap<S> {
	fn default() -> Self {
		Self {
			hash_builder: S::default(),
			table: RawTable::default()
		}
	}
}

impl<S> IndexMap<S> {
	pub fn new() -> Self where S: Default {
		Self::default()
	}
}

impl<S: BuildHasher> IndexMap<S> {
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
			},
			None => {
				self.table.insert(hash, Indexes::new(index), make_hasher::<M, S>(entries, &self.hash_builder));
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