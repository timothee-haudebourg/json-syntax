//! JSON value kinds.
use core::fmt;

/// Value kind.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Kind {
	Null,
	Boolean,
	Number,
	String,
	Array,
	Object,
}

impl std::ops::BitOr for Kind {
	type Output = KindSet;

	fn bitor(self, other: Self) -> KindSet {
		KindSet::from(self) | KindSet::from(other)
	}
}

impl std::ops::BitOr<KindSet> for Kind {
	type Output = KindSet;

	fn bitor(self, other: KindSet) -> KindSet {
		KindSet::from(self) | other
	}
}

impl std::ops::BitAnd for Kind {
	type Output = KindSet;

	fn bitand(self, other: Self) -> KindSet {
		KindSet::from(self) & KindSet::from(other)
	}
}

impl std::ops::BitAnd<KindSet> for Kind {
	type Output = KindSet;

	fn bitand(self, other: KindSet) -> KindSet {
		KindSet::from(self) & other
	}
}

impl fmt::Display for Kind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::Boolean => write!(f, "boolean"),
			Self::Number => write!(f, "number"),
			Self::String => write!(f, "string"),
			Self::Array => write!(f, "array"),
			Self::Object => write!(f, "object"),
		}
	}
}

macro_rules! kind_set {
	($($id:ident ($const:ident): $mask:literal),*) => {
		/// Set of JSON value [`Kind`].
		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct KindSet(u8);

		impl KindSet {
			$(
				pub const $const: Self = Self($mask);
			)*

			pub const fn all() -> Self {
				Self($($mask)|*)
			}
		}

		impl std::ops::BitOr<Kind> for KindSet {
			type Output = Self;

			fn bitor(self, other: Kind) -> Self {
				match other {
					$(
						Kind::$id => Self(self.0 | $mask)
					),*
				}
			}
		}

		impl std::ops::BitOrAssign<Kind> for KindSet {
			fn bitor_assign(&mut self, other: Kind) {
				match other {
					$(
						Kind::$id => self.0 |= $mask
					),*
				}
			}
		}

		impl std::ops::BitAnd<Kind> for KindSet {
			type Output = Self;

			fn bitand(self, other: Kind) -> Self {
				match other {
					$(
						Kind::$id => Self(self.0 & $mask)
					),*
				}
			}
		}

		impl std::ops::BitAndAssign<Kind> for KindSet {
			fn bitand_assign(&mut self, other: Kind) {
				match other {
					$(
						Kind::$id => self.0 &= $mask
					),*
				}
			}
		}

		impl From<Kind> for KindSet {
			fn from(value: Kind) -> Self {
				match value {
					$(
						Kind::$id => Self($mask)
					),*
				}
			}
		}

		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct KindSetIter(u8);

		impl Iterator for KindSetIter {
			type Item = Kind;

			fn size_hint(&self) -> (usize, Option<usize>) {
				let len = self.0.count_ones() as usize;
				(len, Some(len))
			}

			fn next(&mut self) -> Option<Kind> {
				$(
					if self.0 & $mask != 0 {
						self.0 &= !$mask;
						return Some(Kind::$id)
					}
				)*

				None
			}
		}

		impl DoubleEndedIterator for KindSetIter {
			fn next_back(&mut self) -> Option<Kind> {
				let mut result = None;

				$(
					if self.0 & $mask != 0 {
						result = Some((Kind::$id, $mask));
					}
				)*

				result.map(|(kind, mask)| {
					self.0 &= !mask;
					kind
				})
			}
		}

		impl std::iter::FusedIterator for KindSetIter {}
		impl std::iter::ExactSizeIterator for KindSetIter {}
	};
}

kind_set! {
	Null (NULL):       0b000001,
	Boolean (BOOLEAN): 0b000010,
	Number (NUMBER):   0b000100,
	String (STRING):   0b001000,
	Array (ARRAY):     0b010000,
	Object (OBJECT):   0b100000
}

impl KindSet {
	pub const fn none() -> Self {
		Self(0)
	}

	pub const fn len(&self) -> usize {
		self.0.count_ones() as usize
	}

	pub const fn is_empty(&self) -> bool {
		self.0 == 0
	}

	pub fn iter(&self) -> KindSetIter {
		KindSetIter(self.0)
	}

	/// Displays this set as a disjunction.
	///
	/// # Example
	///
	/// ```
	/// # use json_syntax::{Kind, KindSet};
	/// let set = Kind::Null | Kind::String | Kind::Object;
	/// assert_eq!(set.as_disjunction().to_string(), "null, string or object");
	/// assert_eq!(KindSet::ARRAY.as_disjunction().to_string(), "array");
	/// assert_eq!(KindSet::all().as_disjunction().to_string(), "anything");
	/// assert_eq!(KindSet::none().as_disjunction().to_string(), "nothing");
	/// ```
	pub fn as_disjunction(self) -> KindSetDisjunction {
		KindSetDisjunction(self)
	}

	/// Displays this set as a conjunction.
	///
	/// # Example
	///
	/// ```
	/// # use json_syntax::{Kind, KindSet};
	/// let set = Kind::Null | Kind::String | Kind::Object;
	/// assert_eq!(set.as_conjunction().to_string(), "null, string and object");
	/// assert_eq!(KindSet::ARRAY.as_conjunction().to_string(), "array");
	/// assert_eq!(KindSet::all().as_conjunction().to_string(), "anything");
	/// assert_eq!(KindSet::none().as_conjunction().to_string(), "nothing");
	/// ```
	pub fn as_conjunction(self) -> KindSetConjunction {
		KindSetConjunction(self)
	}
}

impl std::ops::BitOr for KindSet {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		Self(self.0 | other.0)
	}
}

impl std::ops::BitOrAssign for KindSet {
	fn bitor_assign(&mut self, other: Self) {
		self.0 |= other.0
	}
}

impl std::ops::BitAnd for KindSet {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		Self(self.0 & other.0)
	}
}

impl std::ops::BitAndAssign for KindSet {
	fn bitand_assign(&mut self, other: Self) {
		self.0 &= other.0
	}
}

impl<'a> IntoIterator for &'a KindSet {
	type IntoIter = KindSetIter;
	type Item = Kind;

	fn into_iter(self) -> KindSetIter {
		self.iter()
	}
}

impl IntoIterator for KindSet {
	type IntoIter = KindSetIter;
	type Item = Kind;

	fn into_iter(self) -> KindSetIter {
		self.iter()
	}
}

impl fmt::Display for KindSet {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, kind) in self.into_iter().enumerate() {
			if i > 0 {
				f.write_str(", ")?;
			}

			kind.fmt(f)?;
		}

		Ok(())
	}
}

/// Displays a `KindSet` as a disjunction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KindSetDisjunction(pub KindSet);

impl fmt::Display for KindSetDisjunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0 == KindSet::all() {
			f.write_str("anything")
		} else {
			let mut iter = self.0.into_iter();
			match iter.next_back() {
				Some(last) => {
					if let Some(first) = iter.next() {
						first.fmt(f)?;
						for k in iter {
							f.write_str(", ")?;
							k.fmt(f)?;
						}
						f.write_str(" or ")?;
					}

					last.fmt(f)
				}
				None => f.write_str("nothing"),
			}
		}
	}
}

/// Displays a `KindSet` as a conjunction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KindSetConjunction(pub KindSet);

impl fmt::Display for KindSetConjunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0 == KindSet::all() {
			f.write_str("anything")
		} else {
			let mut iter = self.0.into_iter();
			match iter.next_back() {
				Some(last) => {
					if let Some(first) = iter.next() {
						first.fmt(f)?;
						for k in iter {
							f.write_str(", ")?;
							k.fmt(f)?;
						}
						f.write_str(" and ")?;
					}

					last.fmt(f)
				}
				None => f.write_str("nothing"),
			}
		}
	}
}
