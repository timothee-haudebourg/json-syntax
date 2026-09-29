use core::hash::{Hash, Hasher};

use crate::JsonValue;

/// Wrapper around a JSON value to consider its lexical representation without
/// regard for semantical equivalences.
#[derive(Debug)]
#[repr(transparent)]
pub struct JsonLexical<T: ?Sized = JsonValue>(pub T);

pub trait JsonBorrowLexical {
	fn as_lexical(&self) -> &JsonLexical<Self> {
		unsafe { core::mem::transmute(self) }
	}
}

pub trait JsonLexicalPartialEq {
	fn lexical_eq(&self, other: &Self) -> bool;
}

impl<T: JsonLexicalPartialEq> JsonLexicalPartialEq for Vec<T> {
	fn lexical_eq(&self, other: &Self) -> bool {
		self.len() == other.len() && self.iter().zip(other).all(|(a, b)| a.lexical_eq(b))
	}
}

impl<T: JsonLexicalPartialEq> PartialEq for JsonLexical<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0.lexical_eq(&other.0)
	}
}

pub trait JsonLexicalEq: JsonLexicalPartialEq {}

impl<T: JsonLexicalEq> Eq for JsonLexical<T> {}

pub trait JsonLexicalHash {
	fn lexical_hash<H: Hasher>(&self, state: &mut H);
}

impl<T: JsonLexicalHash> Hash for JsonLexical<T> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.lexical_hash(state)
	}
}
