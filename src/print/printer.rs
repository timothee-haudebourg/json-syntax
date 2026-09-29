use core::fmt;
use std::fmt::Display;

use crate::{
	JsonNumber,
	visitor::{JsonVisit, JsonVisitor},
};

use super::{JsonPrintOptions, Spaces, sizes::JsonSize};

pub struct JsonPrinter<'a, 'f> {
	options: &'a JsonPrintOptions,
	sizes: &'a [JsonSize],
	offset: &'a mut usize,
	indent: usize,
	formatter: &'a mut fmt::Formatter<'f>,
}

impl<'a, 'f> JsonPrinter<'a, 'f> {
	pub fn new(
		options: &'a JsonPrintOptions,
		sizes: &'a [JsonSize],
		offset: &'a mut usize,
		indent: usize,
		formatter: &'a mut fmt::Formatter<'f>,
	) -> Self {
		Self {
			options,
			sizes,
			offset,
			indent,
			formatter,
		}
	}
}

impl<'a, 'f> JsonVisitor for JsonPrinter<'a, 'f> {
	type Output = fmt::Result;

	fn visit_null(self) -> Self::Output {
		self.formatter.write_str("null")
	}

	fn visit_bool(self, value: bool) -> Self::Output {
		if value {
			self.formatter.write_str("true")
		} else {
			self.formatter.write_str("false")
		}
	}

	fn visit_number(self, value: &JsonNumber) -> Self::Output {
		self.formatter.write_str(value.as_str())
	}

	fn visit_string(self, value: &str) -> Self::Output {
		string_literal(value, self.formatter)
	}

	fn visit_array<I: IntoIterator<Item: JsonVisit>>(self, items: I) -> Self::Output {
		let size = self.sizes[*self.offset];
		*self.offset += 1;

		self.formatter.write_str("[")?;

		match size {
			JsonSize::Expanded => {
				self.formatter.write_str("\n")?;

				let mut empty = true;
				for item in items {
					if empty {
						empty = false;
					} else {
						Spaces(self.options.array_before_comma).fmt(self.formatter)?;
						self.formatter.write_str(",\n")?
					}

					self.options
						.indent
						.by(self.indent + 1)
						.fmt(self.formatter)?;

					let item_printer = JsonPrinter {
						options: self.options,
						sizes: self.sizes,
						offset: self.offset,
						formatter: self.formatter,
						indent: self.indent + 1,
					};

					item.visit(item_printer)?
				}

				if !empty {
					self.formatter.write_str("\n")?;
				}

				self.options.indent.by(self.indent).fmt(self.formatter)?;
			}
			JsonSize::Width(_) => {
				let mut empty = true;
				for item in items {
					if empty {
						empty = false;
						Spaces(self.options.array_begin).fmt(self.formatter)?;
					} else {
						Spaces(self.options.array_before_comma).fmt(self.formatter)?;
						self.formatter.write_str(",")?;
						Spaces(self.options.array_after_comma).fmt(self.formatter)?
					}

					let item_printer = JsonPrinter {
						options: self.options,
						sizes: self.sizes,
						offset: self.offset,
						formatter: self.formatter,
						indent: self.indent + 1,
					};

					item.visit(item_printer)?
				}

				if empty {
					Spaces(self.options.array_empty).fmt(self.formatter)?
				} else {
					Spaces(self.options.array_end).fmt(self.formatter)?
				}
			}
		}

		self.formatter.write_str("]")
	}

	fn visit_object<E: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: JsonVisit>(
		self,
		entries: E,
	) -> Self::Output {
		let size = self.sizes[*self.offset];
		*self.offset += 1;

		self.formatter.write_str("{")?;

		match size {
			JsonSize::Expanded => {
				self.formatter.write_str("\n")?;

				let mut empty = true;
				for (key, value) in entries {
					if empty {
						empty = false;
					} else {
						Spaces(self.options.object_before_comma).fmt(self.formatter)?;
						self.formatter.write_str(",\n")?
					}

					self.options
						.indent
						.by(self.indent + 1)
						.fmt(self.formatter)?;

					string_literal(key.as_ref(), self.formatter)?;

					Spaces(self.options.object_before_colon).fmt(self.formatter)?;
					self.formatter.write_str(":")?;
					Spaces(self.options.object_after_colon).fmt(self.formatter)?;

					let value_printer = JsonPrinter {
						options: self.options,
						sizes: self.sizes,
						offset: self.offset,
						formatter: self.formatter,
						indent: self.indent + 1,
					};

					value.visit(value_printer)?
				}

				if !empty {
					self.formatter.write_str("\n")?;
				}

				self.options.indent.by(self.indent).fmt(self.formatter)?;
			}
			JsonSize::Width(_) => {
				let mut empty = true;
				for (key, value) in entries {
					if empty {
						empty = false;
						Spaces(self.options.object_begin).fmt(self.formatter)?;
					} else {
						Spaces(self.options.object_before_comma).fmt(self.formatter)?;
						self.formatter.write_str(",")?;
						Spaces(self.options.object_after_comma).fmt(self.formatter)?
					}

					string_literal(key.as_ref(), self.formatter)?;
					Spaces(self.options.object_before_colon).fmt(self.formatter)?;
					self.formatter.write_str(":")?;
					Spaces(self.options.object_after_colon).fmt(self.formatter)?;

					let value_printer = JsonPrinter {
						options: self.options,
						sizes: self.sizes,
						offset: self.offset,
						formatter: self.formatter,
						indent: self.indent + 1,
					};

					value.visit(value_printer)?
				}

				if empty {
					Spaces(self.options.object_empty).fmt(self.formatter)?
				} else {
					Spaces(self.options.object_end).fmt(self.formatter)?
				}
			}
		}

		self.formatter.write_str("}")
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
