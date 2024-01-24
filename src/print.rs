use std::fmt;

#[cfg(feature = "contextual")]
mod contextual;

#[cfg(feature = "contextual")]
pub use self::contextual::*;

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

impl<T: Print> Print for locspan::Stripped<T> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		self.0.fmt_with(f, options, indent)
	}
}

impl<T: Print, M> Print for locspan::Meta<T, M> {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		self.value().fmt_with(f, options, indent)
	}
}

impl<'a, T: Print + ?Sized> Print for &'a T {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		(**self).fmt_with(f, options, indent)
	}
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

impl<T: PrintWithSize> PrintWithSize for locspan::Stripped<T> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		self.0.fmt_with_size(f, options, indent, sizes, index)
	}
}

impl<T: PrintWithSize, M> PrintWithSize for locspan::Meta<T, M> {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		self.value().fmt_with_size(f, options, indent, sizes, index)
	}
}

impl<'a, T: PrintWithSize + ?Sized> PrintWithSize for &'a T {
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		(**self).fmt_with_size(f, options, indent, sizes, index)
	}
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

/// Formats a string literal according to [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-strings).
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

				digit(a).fmt(f)?;
				digit(b).fmt(f)?;
				digit(c).fmt(f)?;
				digit(d).fmt(f)?
			}
			_ => c.fmt(f)?,
		}
	}

	f.write_str("\"")
}

fn digit(c: u32) -> char {
	match c {
		0x0 => '0',
		0x1 => '1',
		0x2 => '2',
		0x3 => '3',
		0x4 => '4',
		0x5 => '5',
		0x6 => '6',
		0x7 => '7',
		0x8 => '8',
		0x9 => '9',
		0xa => 'a',
		0xb => 'b',
		0xc => 'c',
		0xd => 'd',
		0xe => 'e',
		0xf => 'f',
		_ => panic!("invalid input: {}", c),
	}
}

/// Returns the byte length of string literal according to [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-strings).
pub fn printed_string_size(s: &str) -> usize {
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

pub fn print_array<I: IntoIterator>(
	items: I,
	f: &mut fmt::Formatter,
	options: &Options,
	indent: usize,
	sizes: &[Size],
	index: &mut usize,
) -> fmt::Result
where
	I::IntoIter: ExactSizeIterator,
	I::Item: PrintWithSize,
{
	use fmt::Display;
	let size = sizes[*index];
	*index += 1;

	f.write_str("[")?;

	let items = items.into_iter();
	if items.len() == 0 {
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

				for (i, item) in items.enumerate() {
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
				for (i, item) in items.enumerate() {
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

impl<T: PrintWithSize> PrintWithSize for Vec<T> {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		print_array(self, f, options, indent, sizes, index)
	}
}

pub fn print_object<'a, V, I: IntoIterator<Item = (&'a str, V)>>(
	entries: I,
	f: &mut fmt::Formatter,
	options: &Options,
	indent: usize,
	sizes: &[Size],
	index: &mut usize,
) -> fmt::Result
where
	I::IntoIter: ExactSizeIterator,
	V: PrintWithSize,
{
	use fmt::Display;
	let size = sizes[*index];
	*index += 1;

	f.write_str("{")?;

	let entries = entries.into_iter();
	if entries.len() == 0 {
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

				for (i, (key, value)) in entries.enumerate() {
					if i > 0 {
						Spaces(options.object_before_comma).fmt(f)?;
						f.write_str(",\n")?
					}

					options.indent.by(indent + 1).fmt(f)?;

					string_literal(key, f)?;
					Spaces(options.object_before_colon).fmt(f)?;
					f.write_str(":")?;
					Spaces(options.object_after_colon).fmt(f)?;

					value.fmt_with_size(f, options, indent + 1, sizes, index)?
				}

				f.write_str("\n")?;
				options.indent.by(indent).fmt(f)?;
			}
			Size::Width(_) => {
				Spaces(options.object_begin).fmt(f)?;
				for (i, (key, value)) in entries.enumerate() {
					if i > 0 {
						Spaces(options.object_before_comma).fmt(f)?;
						f.write_str(",")?;
						Spaces(options.object_after_comma).fmt(f)?
					}

					string_literal(key, f)?;
					Spaces(options.object_before_colon).fmt(f)?;
					f.write_str(":")?;
					Spaces(options.object_after_colon).fmt(f)?;

					value.fmt_with_size(f, options, indent + 1, sizes, index)?
				}
				Spaces(options.object_end).fmt(f)?
			}
		}
	}

	f.write_str("}")
}

impl PrintWithSize for crate::Object {
	#[inline(always)]
	fn fmt_with_size(
		&self,
		f: &mut fmt::Formatter,
		options: &Options,
		indent: usize,
		sizes: &[Size],
		index: &mut usize,
	) -> fmt::Result {
		print_object(
			self.iter().map(|e| (e.key.as_str(), &e.value)),
			f,
			options,
			indent,
			sizes,
			index,
		)
	}
}

pub trait PrecomputeSize {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size;
}

impl PrecomputeSize for bool {
	#[inline(always)]
	fn pre_compute_size(&self, _options: &Options, _sizes: &mut Vec<Size>) -> Size {
		if *self {
			Size::Width(4)
		} else {
			Size::Width(5)
		}
	}
}

impl PrecomputeSize for crate::Value {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		match self {
			crate::Value::Null => Size::Width(4),
			crate::Value::Boolean(b) => b.pre_compute_size(options, sizes),
			crate::Value::Number(n) => Size::Width(n.as_str().len()),
			crate::Value::String(s) => Size::Width(printed_string_size(s)),
			crate::Value::Array(a) => pre_compute_array_size(a, options, sizes),
			crate::Value::Object(o) => pre_compute_object_size(
				o.iter().map(|e| (e.key.as_str(), &e.value)),
				options,
				sizes,
			),
		}
	}
}

impl<'a, T: PrecomputeSize + ?Sized> PrecomputeSize for &'a T {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		(**self).pre_compute_size(options, sizes)
	}
}

impl<T: PrecomputeSize> PrecomputeSize for locspan::Stripped<T> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.0.pre_compute_size(options, sizes)
	}
}

impl<T: PrecomputeSize, M> PrecomputeSize for locspan::Meta<T, M> {
	fn pre_compute_size(&self, options: &Options, sizes: &mut Vec<Size>) -> Size {
		self.value().pre_compute_size(options, sizes)
	}
}

pub fn pre_compute_array_size<I: IntoIterator>(
	items: I,
	options: &Options,
	sizes: &mut Vec<Size>,
) -> Size
where
	I::Item: PrecomputeSize,
{
	let index = sizes.len();
	sizes.push(Size::Width(0));

	let mut size = Size::Width(2 + options.object_begin + options.object_end);

	let mut len = 0;
	for (i, item) in items.into_iter().enumerate() {
		if i > 0 {
			size.add(Size::Width(
				1 + options.array_before_comma + options.array_after_comma,
			));
		}

		size.add(item.pre_compute_size(options, sizes));
		len += 1
	}

	let size = match size {
		Size::Expanded => Size::Expanded,
		Size::Width(width) => match options.array_limit {
			None => Size::Width(width),
			Some(Limit::Always) => Size::Expanded,
			Some(Limit::Item(i)) => {
				if len > i {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
			Some(Limit::ItemOrWidth(i, w)) => {
				if len > i || width > w {
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
	};

	sizes[index] = size;
	size
}

pub fn pre_compute_object_size<'a, V, I: IntoIterator<Item = (&'a str, V)>>(
	entries: I,
	options: &Options,
	sizes: &mut Vec<Size>,
) -> Size
where
	V: PrecomputeSize,
{
	let index = sizes.len();
	sizes.push(Size::Width(0));

	let mut size = Size::Width(2 + options.object_begin + options.object_end);

	let mut len = 0;
	for (i, (key, value)) in entries.into_iter().enumerate() {
		if i > 0 {
			size.add(Size::Width(
				1 + options.object_before_comma + options.object_after_comma,
			));
		}

		size.add(Size::Width(
			printed_string_size(key) + 1 + options.object_before_colon + options.object_after_colon,
		));
		size.add(value.pre_compute_size(options, sizes));
		len += 1;
	}

	let size = match size {
		Size::Expanded => Size::Expanded,
		Size::Width(width) => match options.object_limit {
			None => Size::Width(width),
			Some(Limit::Always) => Size::Expanded,
			Some(Limit::Item(i)) => {
				if len > i {
					Size::Expanded
				} else {
					Size::Width(width)
				}
			}
			Some(Limit::ItemOrWidth(i, w)) => {
				if len > i || width > w {
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
	};

	sizes[index] = size;
	size
}

impl Print for crate::Value {
	fn fmt_with(&self, f: &mut fmt::Formatter, options: &Options, indent: usize) -> fmt::Result {
		match self {
			Self::Null => f.write_str("null"),
			Self::Boolean(b) => b.fmt_with(f, options, indent),
			Self::Number(n) => n.fmt_with(f, options, indent),
			Self::String(s) => s.fmt_with(f, options, indent),
			Self::Array(a) => {
				let mut sizes =
					Vec::with_capacity(self.count(|_, v| v.is_array() || v.is_object()));
				self.pre_compute_size(options, &mut sizes);
				let mut index = 0;
				a.fmt_with_size(f, options, indent, &sizes, &mut index)
			}
			Self::Object(o) => {
				let mut sizes =
					Vec::with_capacity(self.count(|_, v| v.is_array() || v.is_object()));
				self.pre_compute_size(options, &mut sizes);
				let mut index = 0;
				o.fmt_with_size(f, options, indent, &sizes, &mut index)
			}
		}
	}
}

impl PrintWithSize for crate::Value {
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
