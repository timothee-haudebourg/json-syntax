use crate::{
	JsonNumber,
	visitor::{JsonVisit, JsonVisitor},
};

use super::{JsonPrintOptions, Limit};

/// The size of a value.
#[derive(Clone, Copy)]
pub enum JsonSize {
	/// The value (array or object) is expanded on multiple lines.
	Expanded,

	/// The value is formatted in a single line with the given character width.
	Width(usize),
}

impl JsonSize {
	pub fn add(&mut self, other: Self) {
		*self = match (*self, other) {
			(Self::Width(a), Self::Width(b)) => Self::Width(a + b),
			_ => Self::Expanded,
		}
	}
}

pub struct JsonSizesVisitor<'a> {
	// Formatting options.
	options: &'a JsonPrintOptions,

	// Stores the size of expandable values (arrays and objects).
	sizes: &'a mut Vec<JsonSize>,
}

impl<'a> JsonSizesVisitor<'a> {
	pub fn new(options: &'a JsonPrintOptions, sizes: &'a mut Vec<JsonSize>) -> Self {
		Self { options, sizes }
	}
}

impl<'a> JsonVisitor for JsonSizesVisitor<'a> {
	type Output = JsonSize;

	fn visit_null(self) -> Self::Output {
		JsonSize::Width(4)
	}

	fn visit_bool(self, value: bool) -> Self::Output {
		if value {
			JsonSize::Width(4)
		} else {
			JsonSize::Width(5)
		}
	}

	fn visit_number(self, value: &JsonNumber) -> Self::Output {
		JsonSize::Width(value.as_str().len())
	}

	fn visit_string(self, value: &str) -> Self::Output {
		JsonSize::Width(printed_string_size(value))
	}

	fn visit_array<I: IntoIterator<Item: JsonVisit>>(self, items: I) -> Self::Output {
		let index = self.sizes.len();
		self.sizes.push(JsonSize::Width(0));

		let mut size = JsonSize::Width(2 + self.options.object_begin + self.options.object_end);

		let mut len = 0;
		for (i, item) in items.into_iter().enumerate() {
			if i > 0 {
				size.add(JsonSize::Width(
					1 + self.options.array_before_comma + self.options.array_after_comma,
				));
			}

			let item_visitor = JsonSizesVisitor {
				options: self.options,
				sizes: &mut *self.sizes,
			};

			size.add(item.visit(item_visitor));

			len += 1
		}

		let size = match size {
			JsonSize::Expanded => JsonSize::Expanded,
			JsonSize::Width(width) => match self.options.array_limit {
				None => JsonSize::Width(width),
				Some(Limit::Always) => JsonSize::Expanded,
				Some(Limit::Item(i)) => {
					if len > i {
						JsonSize::Expanded
					} else {
						JsonSize::Width(width)
					}
				}
				Some(Limit::ItemOrWidth(i, w)) => {
					if len > i || width > w {
						JsonSize::Expanded
					} else {
						JsonSize::Width(width)
					}
				}
				Some(Limit::Width(w)) => {
					if width > w {
						JsonSize::Expanded
					} else {
						JsonSize::Width(width)
					}
				}
			},
		};

		self.sizes[index] = size;
		size
	}

	fn visit_object<E: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: JsonVisit>(
		self,
		entries: E,
	) -> Self::Output {
		let index = self.sizes.len();
		self.sizes.push(JsonSize::Width(0));

		let mut size = JsonSize::Width(2 + self.options.object_begin + self.options.object_end);

		let mut len = 0;
		for (i, (key, value)) in entries.into_iter().enumerate() {
			if i > 0 {
				size.add(JsonSize::Width(
					1 + self.options.object_before_comma + self.options.object_after_comma,
				));
			}

			size.add(JsonSize::Width(
				printed_string_size(key.as_ref())
					+ 1 + self.options.object_before_colon
					+ self.options.object_after_colon,
			));

			let value_visitor = JsonSizesVisitor {
				options: self.options,
				sizes: &mut *self.sizes,
			};

			size.add(value.visit(value_visitor));
			len += 1;
		}

		let size = match size {
			JsonSize::Expanded => JsonSize::Expanded,
			JsonSize::Width(width) => match self.options.object_limit {
				None => JsonSize::Width(width),
				Some(Limit::Always) => JsonSize::Expanded,
				Some(Limit::Item(i)) => {
					if len > i {
						JsonSize::Expanded
					} else {
						JsonSize::Width(width)
					}
				}
				Some(Limit::ItemOrWidth(i, w)) => {
					if len > i || width > w {
						JsonSize::Expanded
					} else {
						JsonSize::Width(width)
					}
				}
				Some(Limit::Width(w)) => {
					if width > w {
						JsonSize::Expanded
					} else {
						JsonSize::Width(width)
					}
				}
			},
		};

		self.sizes[index] = size;
		size
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
