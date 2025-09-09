use core::fmt;
use printer::Printer;
use sizes::SizesVisitor;

use crate::visitor::JsonValue;

mod printer;
mod sizes;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Indent {
	Spaces(u8),
	Tabs(u8),
}

impl Indent {
	pub fn by(self, n: usize) -> IndentBy {
		IndentBy(self, n)
	}
}

impl fmt::Display for Indent {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Spaces(n) => {
				for _ in 0..*n {
					f.write_str(" ")?
				}
			}
			Self::Tabs(n) => {
				for _ in 0..*n {
					f.write_str("\t")?
				}
			}
		}

		Ok(())
	}
}

pub struct IndentBy(Indent, usize);

impl fmt::Display for IndentBy {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for _ in 0..self.1 {
			self.0.fmt(f)?
		}

		Ok(())
	}
}

pub struct Spaces(pub usize);

impl fmt::Display for Spaces {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for _ in 0..self.0 {
			f.write_str(" ")?
		}

		Ok(())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Padding {
	Spaces(u8),
	NewLine,
}

impl fmt::Display for Padding {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Spaces(n) => {
				for _ in 0..*n {
					f.write_str(" ")?
				}
			}
			Self::NewLine => f.write_str("\n")?,
		}

		Ok(())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Limit {
	/// Always expanded, even if empty.
	Always,

	/// Expanded if the array/object has more than the given number of items.
	Item(usize),

	/// Expanded if the representation of the array/object is more than the
	/// given number of characters long.
	Width(usize),

	/// Expanded if the array/object has more than the given number of items
	/// (first argument), or if its the representation is more than the
	/// given number of characters long (second argument).
	ItemOrWidth(usize, usize),
}

/// Print options.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub struct Options {
	/// Indentation string.
	pub indent: Indent,

	/// String added after `[`.
	pub array_begin: usize,

	/// String added before `]`.
	pub array_end: usize,

	/// Number of spaces inside an inlined empty array.
	pub array_empty: usize,

	/// Number of spaces before a comma in an array.
	pub array_before_comma: usize,

	/// Number of spaces after a comma in an array.
	pub array_after_comma: usize,

	/// Limit after which an array is expanded.
	pub array_limit: Option<Limit>,

	/// String added after `{`.
	pub object_begin: usize,

	/// String added before `}`.
	pub object_end: usize,

	/// Number of spaces inside an inlined empty object.
	pub object_empty: usize,

	/// Number of spaces before a comma in an object.
	pub object_before_comma: usize,

	/// Number of spaces after a comma in an object.
	pub object_after_comma: usize,

	/// Number of spaces before a colon in an object.
	pub object_before_colon: usize,

	/// Number of spaces after a colon in an object.
	pub object_after_colon: usize,

	/// Limit after which an array is expanded.
	pub object_limit: Option<Limit>,
}

impl Options {
	/// Pretty print options.
	#[inline(always)]
	pub fn pretty() -> Self {
		Self {
			indent: Indent::Spaces(2),
			array_begin: 1,
			array_end: 1,
			array_empty: 0,
			array_before_comma: 0,
			array_after_comma: 1,
			array_limit: Some(Limit::ItemOrWidth(1, 16)),
			object_begin: 1,
			object_end: 1,
			object_empty: 0,
			object_before_comma: 0,
			object_after_comma: 1,
			object_before_colon: 0,
			object_after_colon: 1,
			object_limit: Some(Limit::ItemOrWidth(1, 16)),
		}
	}

	/// Compact print options.
	///
	/// Values will be formatted on a single line without spaces.
	#[inline(always)]
	pub fn compact() -> Self {
		Self {
			indent: Indent::Spaces(0),
			array_begin: 0,
			array_end: 0,
			array_empty: 0,
			array_before_comma: 0,
			array_after_comma: 0,
			array_limit: None,
			object_begin: 0,
			object_end: 0,
			object_empty: 0,
			object_before_comma: 0,
			object_after_comma: 0,
			object_before_colon: 0,
			object_after_colon: 0,
			object_limit: None,
		}
	}

	/// Inline print options.
	///
	/// Values will be formatted on a single line with some spaces.
	#[inline(always)]
	pub fn inline() -> Self {
		Self {
			indent: Indent::Spaces(0),
			array_begin: 1,
			array_end: 1,
			array_empty: 0,
			array_before_comma: 0,
			array_after_comma: 1,
			array_limit: None,
			object_begin: 1,
			object_end: 1,
			object_empty: 0,
			object_before_comma: 0,
			object_after_comma: 1,
			object_before_colon: 0,
			object_after_colon: 1,
			object_limit: None,
		}
	}
}

/// Print methods.
pub trait Print {
	/// Print the value with `Options::pretty` options.
	#[inline(always)]
	fn pretty_print(&self) -> Printed<'_, Self> {
		self.print_with(Options::pretty())
	}

	/// Print the value with `Options::compact` options.
	#[inline(always)]
	fn compact_print(&self) -> Printed<'_, Self> {
		self.print_with(Options::compact())
	}

	/// Print the value with `Options::inline` options.
	#[inline(always)]
	fn inline_print(&self) -> Printed<'_, Self> {
		self.print_with(Options::inline())
	}

	/// Print the value with the given options.
	#[inline(always)]
	fn print_with(&self, options: Options) -> Printed<'_, Self> {
		Printed(self, options, 0)
	}

	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result;
}

impl<T: JsonValue> Print for T {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		let mut sizes = Vec::new();
		self.visit(SizesVisitor::new(options, &mut sizes));
		let mut offset = 0;
		self.visit(Printer::new(options, &sizes, &mut offset, indent, f))
	}
}

/// Printed value.
pub struct Printed<'t, T: ?Sized>(&'t T, Options, usize);

impl<T: Print> fmt::Display for Printed<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt_with(f, &self.1, self.2)
	}
}
