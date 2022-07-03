use std::fmt;

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

/// The size of a value.
#[derive(Clone, Copy)]
pub enum Size {
	/// The value (array or object) is expanded on multiple lines.
	Expanded,

	/// The value is formatted in a single line with the given character width.
	Width(usize),
}

impl Size {
	pub fn add(&mut self, other: Self) {
		*self = match (*self, other) {
			(Self::Width(a), Self::Width(b)) => Self::Width(a + b),
			_ => Self::Expanded,
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

pub trait PrintWithSize {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result;
}

/// Printed value.
pub struct Printed<'t, T: ?Sized>(&'t T, Options, usize);

impl<'t, T: Print> fmt::Display for Printed<'t, T> {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.0.fmt_with(f, &self.1, self.2)
	}
}

impl Print for bool {
	#[inline(always)]
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		if *self {
			f.write_str("true")
		} else {
			f.write_str("false")
		}
	}
}

impl Print for crate::NumberBuf {
	#[inline(always)]
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

pub fn string_literal(s: &str, f: &mut fmt::Formatter) -> fmt::Result {
	use fmt::Display;
	f.write_str("\"")?;

	for c in s.chars() {
		match c {
			'\\' => f.write_str("\\\\")?,
			'\"' => f.write_str("\\\"")?,
			'\u{0008}' => f.write_str("\\b")?,
			'\u{0009}' => f.write_str("\\t")?,
			'\u{000a}' => f.write_str("\\n")?,
			'\u{000c}' => f.write_str("\\f")?,
			'\u{000d}' => f.write_str("\\r")?,
			'\u{0000}'..='\u{001f}' => {
				f.write_str("\\u")?;

				let codepoint = c as u32;
				let d = codepoint & 0x000f;
				let c = (codepoint & 0x00f0) >> 4;
				let b = (codepoint & 0x0f00) >> 8;
				let a = (codepoint & 0xf000) >> 12;

				char::from_u32(b'0' as u32 + a).unwrap().fmt(f)?;
				char::from_u32(b'0' as u32 + b).unwrap().fmt(f)?;
				char::from_u32(b'0' as u32 + c).unwrap().fmt(f)?;
				char::from_u32(b'0' as u32 + d).unwrap().fmt(f)?
			}
			_ => c.fmt(f)?,
		}
	}

	f.write_str("\"")
}

fn printed_string_size(s: &str) -> usize {
	let mut width = 2;

	for c in s.chars() {
		width += match c {
			'\\' | '\"' | '\u{0008}' | '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' => 2,
			'\u{0000}'..='\u{001f}' => 6,
			_ => 1,
		}
	}

	width
}

impl Print for crate::String {
	#[inline(always)]
	fn fmt_with(&self, f: &mut fmt::Formatter, _options: &Options, _indent: usize) -> fmt::Result {
		string_literal(self, f)
	}
}

impl<M> PrintWithSize for crate::Array<M> {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		use fmt::Display;
		let size = sizes[*index];
		*index += 1;

		f.write_str("[")?;

		if self.is_empty() {
			match size {
				Size::Expanded => {
					f.write_str("\n")?;
					options.indent.by(indent).fmt(f)?;
				}
				Size::Width(_) => Spaces(options.array_empty).fmt(f)?,
			}
		} else {
			match size {
				Size::Expanded => {
					f.write_str("\n")?;

					for (i, item) in self.iter().enumerate() {
						if i > 0 {
							Spaces(options.array_before_comma).fmt(f)?;
							f.write_str(",\n")?
						}

						options.indent.by(indent + 1).fmt(f)?;
						item.fmt_with_size(f, options, indent + 1, sizes, index)?
					}

					f.write_str("\n")?;
					options.indent.by(indent).fmt(f)?;
				}
				Size::Width(_) => {
					Spaces(options.array_begin).fmt(f)?;
					for (i, item) in self.iter().enumerate() {
						if i > 0 {
							Spaces(options.array_before_comma).fmt(f)?;
							f.write_str(",")?;
							Spaces(options.array_after_comma).fmt(f)?
						}

						item.fmt_with_size(f, options, indent + 1, sizes, index)?
					}
					Spaces(options.array_end).fmt(f)?
				}
			}
		}

		f.write_str("]")
	}
}

impl<M> PrintWithSize for crate::Object<M> {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		use fmt::Display;
		let size = sizes[*index];
		*index += 1;

		f.write_str("{")?;

		if self.is_empty() {
			match size {
				Size::Expanded => {
					f.write_str("\n")?;
					options.indent.by(indent).fmt(f)?;
				}
				Size::Width(_) => Spaces(options.object_empty).fmt(f)?,
			}
		} else {
			match size {
				Size::Expanded => {
					f.write_str("\n")?;

					for (i, entry) in self.iter().enumerate() {
						if i > 0 {
							Spaces(options.object_before_comma).fmt(f)?;
							f.write_str(",\n")?
						}

						options.indent.by(indent + 1).fmt(f)?;

						string_literal(&entry.key, f)?;
						Spaces(options.object_before_colon).fmt(f)?;
						f.write_str(":")?;
						Spaces(options.object_after_colon).fmt(f)?;

						entry
							.value
							.fmt_with_size(f, options, indent + 1, sizes, index)?
					}

					f.write_str("\n")?;
					options.indent.by(indent).fmt(f)?;
				}
				Size::Width(_) => {
					Spaces(options.object_begin).fmt(f)?;
					for (i, entry) in self.iter().enumerate() {
						if i > 0 {
							Spaces(options.object_before_comma).fmt(f)?;
							f.write_str(",")?;
							Spaces(options.object_after_comma).fmt(f)?
						}

						string_literal(&entry.key, f)?;
						Spaces(options.object_before_colon).fmt(f)?;
						f.write_str(":")?;
						Spaces(options.object_after_colon).fmt(f)?;

						entry
							.value
							.fmt_with_size(f, options, indent + 1, sizes, index)?
					}
					Spaces(options.object_end).fmt(f)?
				}
			}
		}

		f.write_str("}")
	}
}

fn pre_compute_size<M>(value: &crate::Value<M>, options: &Options, sizes: &mut Vec<Size>) -> Size {
	match value {
		crate::Value::Null => Size::Width(4),
		crate::Value::Boolean(true) => Size::Width(4),
		crate::Value::Boolean(false) => Size::Width(5),
		crate::Value::Number(n) => Size::Width(n.as_str().len()),
		crate::Value::String(s) => Size::Width(printed_string_size(s)),
		crate::Value::Array(a) => {
			let index = sizes.len();
			sizes.push(Size::Width(0));
			let size = pre_compute_array_size(a, options, sizes);
			sizes[index] = size;
			size
		}
		crate::Value::Object(o) => {
			let index = sizes.len();
			sizes.push(Size::Width(0));
			let size = pre_compute_object_size(o, options, sizes);
			sizes[index] = size;
			size
		}
	}
}

fn pre_compute_array_size<M>(
	array: &crate::Array<M>,
	options: &Options,
	sizes: &mut Vec<Size>,
) -> Size {
	let mut size = Size::Width(2 + options.object_begin + options.object_end);

	for (i, item) in array.iter().enumerate() {
		if i > 0 {
			size.add(Size::Width(
				1 + options.array_before_comma + options.array_after_comma,
			));
		}

		size.add(pre_compute_size(item, options, sizes));
	}

	match size {
		Size::Expanded => Size::Expanded,
		Size::Width(width) => match options.array_limit {
			None => Size::Width(width),
			Some(Limit::Always) => Size::Expanded,
			Some(Limit::Item(i)) => {
				if array.len() > i {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
			Some(Limit::ItemOrWidth(i, w)) => {
				if array.len() > i || width > w {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
			Some(Limit::Width(w)) => {
				if width > w {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
		},
	}
}

fn pre_compute_object_size<M>(
	object: &crate::Object<M>,
	options: &Options,
	sizes: &mut Vec<Size>,
) -> Size {
	let mut size = Size::Width(2 + options.object_begin + options.object_end);

	for (i, entry) in object.iter().enumerate() {
		if i > 0 {
			size.add(Size::Width(
				1 + options.object_before_comma + options.object_after_comma,
			));
		}

		size.add(Size::Width(
			printed_string_size(&entry.key)
				+ 1 + options.object_before_colon
				+ options.object_after_colon,
		));
		size.add(pre_compute_size(&entry.value, options, sizes));
	}

	match size {
		Size::Expanded => Size::Expanded,
		Size::Width(width) => match options.object_limit {
			None => Size::Width(width),
			Some(Limit::Always) => Size::Expanded,
			Some(Limit::Item(i)) => {
				if object.len() > i {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
			Some(Limit::ItemOrWidth(i, w)) => {
				if object.len() > i || width > w {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
			Some(Limit::Width(w)) => {
				if width > w {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
		},
	}
}

impl<M> Print for crate::Value<M> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => f.write_str("null"),
			Self::Boolean(b) => b.fmt_with(f, options, indent),
			Self::Number(n) => n.fmt_with(f, options, indent),
			Self::String(s) => s.fmt_with(f, options, indent),
			Self::Array(a) => {
				let mut sizes = Vec::with_capacity(self.count(|v| v.is_array() || v.is_object()));
				pre_compute_size(self, options, &mut sizes);
				let mut index = 0;
				a.fmt_with_size(f, options, indent, &sizes, &mut index)
			}
			Self::Object(o) => {
				let mut sizes = Vec::with_capacity(self.count(|v| v.is_array() || v.is_object()));
				pre_compute_size(self, options, &mut sizes);
				let mut index = 0;
				o.fmt_with_size(f, options, indent, &sizes, &mut index)
			}
		}
	}
}

impl<M> PrintWithSize for crate::Value<M> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		match self {
			Self::Null => f.write_str("null"),
			Self::Boolean(b) => b.fmt_with(f, options, indent),
			Self::Number(n) => n.fmt_with(f, options, indent),
			Self::String(s) => s.fmt_with(f, options, indent),
			Self::Array(a) => a.fmt_with_size(f, options, indent, sizes, index),
			Self::Object(o) => o.fmt_with_size(f, options, indent, sizes, index),
		}
	}
}
