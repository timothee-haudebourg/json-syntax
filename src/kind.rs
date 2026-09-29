//! JSON value kinds.
use core::fmt;

/// Value kind.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum JsonKind {
	Null,
	Boolean,
	Number,
	String,
	Array,
	Object,
}

impl std::ops::BitOr for JsonKind {
	type Output = JsonKindSet;

	fn bitor(self, other: Self) -> JsonKindSet {
		JsonKindSet::from(self) | JsonKindSet::from(other)
	}
}

impl std::ops::BitOr<JsonKindSet> for JsonKind {
	type Output = JsonKindSet;

	fn bitor(self, other: JsonKindSet) -> JsonKindSet {
		JsonKindSet::from(self) | other
	}
}

impl std::ops::BitAnd for JsonKind {
	type Output = JsonKindSet;

	fn bitand(self, other: Self) -> JsonKindSet {
		JsonKindSet::from(self) & JsonKindSet::from(other)
	}
}

impl std::ops::BitAnd<JsonKindSet> for JsonKind {
	type Output = JsonKindSet;

	fn bitand(self, other: JsonKindSet) -> JsonKindSet {
		JsonKindSet::from(self) & other
	}
}

impl fmt::Display for JsonKind {
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
		/// Set of JSON value [`JsonKind`].
		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct JsonKindSet(u8);

		impl JsonKindSet {
			$(
				pub const $const: Self = Self($mask);
			)*

			pub const fn all() -> Self {
				Self($($mask)|*)
			}
		}

		impl std::ops::BitOr<JsonKind> for JsonKindSet {
			type Output = Self;

			fn bitor(self, other: JsonKind) -> Self {
				match other {
					$(
						JsonKind::$id => Self(self.0 | $mask)
					),*
				}
			}
		}

		impl std::ops::BitOrAssign<JsonKind> for JsonKindSet {
			fn bitor_assign(&mut self, other: JsonKind) {
				match other {
					$(
						JsonKind::$id => self.0 |= $mask
					),*
				}
			}
		}

		impl std::ops::BitAnd<JsonKind> for JsonKindSet {
			type Output = Self;

			fn bitand(self, other: JsonKind) -> Self {
				match other {
					$(
						JsonKind::$id => Self(self.0 & $mask)
					),*
				}
			}
		}

		impl std::ops::BitAndAssign<JsonKind> for JsonKindSet {
			fn bitand_assign(&mut self, other: JsonKind) {
				match other {
					$(
						JsonKind::$id => self.0 &= $mask
					),*
				}
			}
		}

		impl From<JsonKind> for JsonKindSet {
			fn from(value: JsonKind) -> Self {
				match value {
					$(
						JsonKind::$id => Self($mask)
					),*
				}
			}
		}

		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct JsonKindSetIter(u8);

		impl Iterator for JsonKindSetIter {
			type Item = JsonKind;

			fn size_hint(&self) -> (usize, Option<usize>) {
				let len = self.0.count_ones() as usize;
				(len, Some(len))
			}

			fn next(&mut self) -> Option<JsonKind> {
				$(
					if self.0 & $mask != 0 {
						self.0 &= !$mask;
						return Some(JsonKind::$id)
					}
				)*

				None
			}
		}

		impl DoubleEndedIterator for JsonKindSetIter {
			fn next_back(&mut self) -> Option<JsonKind> {
				let mut result = None;

				$(
					if self.0 & $mask != 0 {
						result = Some((JsonKind::$id, $mask));
					}
				)*

				result.map(|(kind, mask)| {
					self.0 &= !mask;
					kind
				})
			}
		}

		impl std::iter::FusedIterator for JsonKindSetIter {}
		impl std::iter::ExactSizeIterator for JsonKindSetIter {}
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

impl JsonKindSet {
	pub const fn none() -> Self {
		Self(0)
	}

	pub const fn len(&self) -> usize {
		self.0.count_ones() as usize
	}

	pub const fn is_empty(&self) -> bool {
		self.0 == 0
	}

	pub fn iter(&self) -> JsonKindSetIter {
		JsonKindSetIter(self.0)
	}

	/// Displays this set as a disjunction.
	///
	/// # Example
	///
	/// ```
	/// # use json_syntax::{JsonKind, JsonKindSet};
	/// let set = JsonKind::Null | JsonKind::String | JsonKind::Object;
	/// assert_eq!(set.as_disjunction().to_string(), "null, string or object");
	/// assert_eq!(JsonKindSet::ARRAY.as_disjunction().to_string(), "array");
	/// assert_eq!(JsonKindSet::all().as_disjunction().to_string(), "anything");
	/// assert_eq!(JsonKindSet::none().as_disjunction().to_string(), "nothing");
	/// ```
	pub fn as_disjunction(self) -> JsonKindSetDisjunction {
		JsonKindSetDisjunction(self)
	}

	/// Displays this set as a conjunction.
	///
	/// # Example
	///
	/// ```
	/// # use json_syntax::{JsonKind, JsonKindSet};
	/// let set = JsonKind::Null | JsonKind::String | JsonKind::Object;
	/// assert_eq!(set.as_conjunction().to_string(), "null, string and object");
	/// assert_eq!(JsonKindSet::ARRAY.as_conjunction().to_string(), "array");
	/// assert_eq!(JsonKindSet::all().as_conjunction().to_string(), "anything");
	/// assert_eq!(JsonKindSet::none().as_conjunction().to_string(), "nothing");
	/// ```
	pub fn as_conjunction(self) -> JsonKindSetConjunction {
		JsonKindSetConjunction(self)
	}
}

impl std::ops::BitOr for JsonKindSet {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		Self(self.0 | other.0)
	}
}

impl std::ops::BitOrAssign for JsonKindSet {
	fn bitor_assign(&mut self, other: Self) {
		self.0 |= other.0
	}
}

impl std::ops::BitAnd for JsonKindSet {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		Self(self.0 & other.0)
	}
}

impl std::ops::BitAndAssign for JsonKindSet {
	fn bitand_assign(&mut self, other: Self) {
		self.0 &= other.0
	}
}

impl IntoIterator for &JsonKindSet {
	type IntoIter = JsonKindSetIter;
	type Item = JsonKind;

	fn into_iter(self) -> JsonKindSetIter {
		self.iter()
	}
}

impl IntoIterator for JsonKindSet {
	type IntoIter = JsonKindSetIter;
	type Item = JsonKind;

	fn into_iter(self) -> JsonKindSetIter {
		self.iter()
	}
}

impl fmt::Display for JsonKindSet {
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

/// Displays a `JsonKindSet` as a disjunction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonKindSetDisjunction(pub JsonKindSet);

impl fmt::Display for JsonKindSetDisjunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0 == JsonKindSet::all() {
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

/// Displays a `JsonKindSet` as a conjunction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonKindSetConjunction(pub JsonKindSet);

impl fmt::Display for JsonKindSetConjunction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0 == JsonKindSet::all() {
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
