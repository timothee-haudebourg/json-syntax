use core::hash::{Hash, Hasher};

/// Wrapper around a value to consider its lexical representation without regard
/// for semantical equivalences.
#[derive(Debug)]
#[repr(transparent)]
pub struct Lexical<T: ?Sized>(pub T);

pub trait BorrowLexical {
	fn as_lexical(&self) -> &Lexical<Self>;
}

impl<T> BorrowLexical for T {
	fn as_lexical(&self) -> &Lexical<Self> {
		unsafe { core::mem::transmute(self) }
	}
}

pub trait LexicalPartialEq {
	fn lexical_eq(&self, other: &Self) -> bool;
}

impl<T: LexicalPartialEq> LexicalPartialEq for Vec<T> {
	fn lexical_eq(&self, other: &Self) -> bool {
		self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.lexical_eq(b))
	}
}

impl<T: LexicalPartialEq> PartialEq for Lexical<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.lexical_eq(&other.0)
	}
}

pub trait LexicalEq: LexicalPartialEq {}

impl<T: LexicalEq> Eq for Lexical<T> {}

pub trait LexicalHash {
	fn lexical_hash<H: Hasher>(&self, state: &mut H);
}

impl<T: LexicalHash> Hash for Lexical<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.lexical_hash(state)
	}
}
