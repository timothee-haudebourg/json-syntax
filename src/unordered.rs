use core::hash::{Hash, Hasher};

/// Wrapper to view a value without considering the order of
/// the objects entries.
#[repr(transparent)]
pub struct Unordered<T: ?Sized>(T);

pub trait BorrowUnordered {
	fn unordered(&self) -> &Unordered<Self>;
}

impl<T> BorrowUnordered for T {
	fn unordered(&self) -> &Unordered<Self> {
		unsafe { core::mem::transmute(self) }
	}
}

pub trait UnorderedPartialEq {
	fn unordered_eq(&self, other: &Self) -> bool;
}

impl<T: UnorderedPartialEq> PartialEq for Unordered<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.unordered_eq(&other.0)
	}
}

pub trait UnorderedEq: UnorderedPartialEq {}

impl<T: UnorderedEq> Eq for Unordered<T> {}

pub trait UnorderedHash {
	fn unordered_hash<H: Hasher>(&self, state: &mut H);
}

impl<T: UnorderedHash> Hash for Unordered<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.unordered_hash(state)
	}
}