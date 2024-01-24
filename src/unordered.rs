use core::hash::{Hash, Hasher};

use locspan::Meta;

/// Wrapper to view a value without considering the order of
/// the objects entries.
#[derive(Debug)]
#[repr(transparent)]
pub struct Unordered<T: ?Sized>(pub T);

pub trait BorrowUnordered {
	fn as_unordered(&self) -> &Unordered<Self>;
}

impl<T> BorrowUnordered for T {
	fn as_unordered(&self) -> &Unordered<Self> {
		unsafe { core::mem::transmute(self) }
	}
}

pub trait UnorderedPartialEq {
	fn unordered_eq(&self, other: &Self) -> bool;
}

impl<T: UnorderedPartialEq, M: PartialEq> UnorderedPartialEq for Meta<T, M> {
	fn unordered_eq(&self, other: &Self) -> bool {
		self.metadata() == other.metadata() && self.value().unordered_eq(other.value())
	}
}

impl<T: UnorderedPartialEq> UnorderedPartialEq for Vec<T> {
	fn unordered_eq(&self, other: &Self) -> bool {
		self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.unordered_eq(b))
	}
}

impl<T: UnorderedPartialEq> PartialEq for Unordered<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.unordered_eq(&other.0)
	}
}

pub trait UnorderedEq: UnorderedPartialEq {}

impl<T: UnorderedEq, M: Eq> UnorderedEq for Meta<T, M> {}

impl<T: UnorderedEq> Eq for Unordered<T> {}

pub trait UnorderedHash {
	fn unordered_hash<H: Hasher>(&self, state: &mut H);
}

impl<T: UnorderedHash> Hash for Unordered<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.unordered_hash(state)
	}
}
