use core::fmt;
use std::{borrow::Borrow, ops::Deref};

use locspan::Span;

use crate::{
	JsonValue,
	print::{
		Indent, JsonPrintOptions,
		sizes::{JsonSize, JsonSizesVisitor, printed_string_size},
	},
	visitor::JsonVisit,
};

pub type JsonCodeMapOffset = usize;

/// Code-map.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonCodeMap(Vec<JsonCodeMapEntry>);

impl JsonCodeMap {
	/// Generates a code map for the given value.
	///
	/// The result is semantically equivalent to printing the value with the
	/// given options and parsing it back, but much cheaper.
	pub fn from_value(value: &JsonValue, options: &JsonPrintOptions) -> Self {
		// First pass: decide for each array/object whether it is expanded or inline.
		let mut sizes = Vec::new();
		value.visit(JsonSizesVisitor::new(options, &mut sizes));

		// Second pass: walk the value in traversal order, tracking the byte
		// position and building one code-map entry per fragment.
		let mut result = Self::default();
		let mut pos = 0usize;
		let mut sizes_offset = 0usize;
		result.extent_from_value(value, options, &sizes, &mut sizes_offset, &mut pos, 0);
		result
	}

	fn extent_from_value(
		&mut self,
		value: &JsonValue,
		options: &JsonPrintOptions,
		sizes: &[JsonSize],
		sizes_offset: &mut usize,
		pos: &mut usize,
		indent: usize,
	) -> usize {
		let start = *pos;
		let my_index = self.len();
		self.0.push(JsonCodeMapEntry::default()); // placeholder filled at the end

		let volume = match value {
			JsonValue::Null => {
				*pos += 4; // "null"
				1
			}
			JsonValue::Boolean(b) => {
				*pos += if *b { 4 } else { 5 }; // "true" / "false"
				1
			}
			JsonValue::Number(n) => {
				*pos += n.as_str().len();
				1
			}
			JsonValue::String(s) => {
				*pos += printed_string_size(s);
				1
			}
			JsonValue::Array(items) => {
				let size = sizes[*sizes_offset];
				*sizes_offset += 1;
				*pos += 1; // '['
				let mut volume = 1;

				match size {
					JsonSize::Expanded => {
						*pos += 1; // '\n'
						let mut first = true;
						for item in items.iter() {
							if first {
								first = false;
							} else {
								*pos += options.array_before_comma; // spaces before comma
								*pos += 2; // ',\n'
							}
							*pos += indent_bytes(options.indent, indent + 1);
							volume += self.extent_from_value(
								item,
								options,
								sizes,
								sizes_offset,
								pos,
								indent + 1,
							);
						}
						if !first {
							*pos += 1; // trailing '\n'
						}
						*pos += indent_bytes(options.indent, indent); // indent for ']'
					}
					JsonSize::Width(_) => {
						let mut first = true;
						for item in items.iter() {
							if first {
								first = false;
								*pos += options.array_begin;
							} else {
								*pos += options.array_before_comma;
								*pos += 1; // ','
								*pos += options.array_after_comma;
							}
							volume += self.extent_from_value(
								item,
								options,
								sizes,
								sizes_offset,
								pos,
								indent + 1,
							);
						}
						*pos += if first {
							options.array_empty
						} else {
							options.array_end
						};
					}
				}

				*pos += 1; // ']'
				volume
			}
			JsonValue::Object(obj) => {
				let size = sizes[*sizes_offset];
				*sizes_offset += 1;
				*pos += 1; // '{'
				let mut volume = 1;

				match size {
					JsonSize::Expanded => {
						*pos += 1; // '\n'
						let mut first = true;
						for (key, value) in obj.iter() {
							if first {
								first = false;
							} else {
								*pos += options.object_before_comma;
								*pos += 2; // ',\n'
							}
							*pos += indent_bytes(options.indent, indent + 1);
							volume += self.extend_from_entry(
								options,
								sizes,
								sizes_offset,
								pos,
								indent,
								key.as_str(),
								value,
							);
						}
						if !first {
							*pos += 1; // trailing '\n'
						}
						*pos += indent_bytes(options.indent, indent); // indent for '}'
					}
					JsonSize::Width(_) => {
						let mut first = true;
						for (key, value) in obj.iter() {
							if first {
								first = false;
								*pos += options.object_begin;
							} else {
								*pos += options.object_before_comma;
								*pos += 1; // ','
								*pos += options.object_after_comma;
							}
							volume += self.extend_from_entry(
								options,
								sizes,
								sizes_offset,
								pos,
								indent,
								key.as_str(),
								value,
							);
						}
						*pos += if first {
							options.object_empty
						} else {
							options.object_end
						};
					}
				}

				*pos += 1; // '}'
				volume
			}
		};

		self.0[my_index] = JsonCodeMapEntry::new(Span::new(start, *pos), volume);
		volume
	}

	#[allow(clippy::too_many_arguments)]
	fn extend_from_entry(
		&mut self,
		options: &JsonPrintOptions,
		sizes: &[JsonSize],
		sizes_offset: &mut usize,
		pos: &mut usize,
		indent: usize,
		key: &str,
		value: &JsonValue,
	) -> usize {
		let entry_start = *pos;
		let entry_index = self.len();
		self.0.push(JsonCodeMapEntry::default());

		// Key (always a leaf — no further recursion needed).
		let key_start = *pos;
		let key_index = self.len();
		self.0.push(JsonCodeMapEntry::default());
		*pos += printed_string_size(key);
		self.0[key_index] = JsonCodeMapEntry::new(Span::new(key_start, *pos), 1);

		*pos += options.object_before_colon;
		*pos += 1; // ':'
		*pos += options.object_after_colon;

		let value_volume =
			self.extent_from_value(value, options, sizes, sizes_offset, pos, indent + 1);

		let entry_volume = 1 + 1 + value_volume; // entry + key + value subtree
		self.0[entry_index] = JsonCodeMapEntry::new(Span::new(entry_start, *pos), entry_volume);
		entry_volume
	}

	pub fn as_slice(&self) -> &[JsonCodeMapEntry] {
		&self.0
	}

	pub(crate) fn reserve(&mut self, position: JsonCodeMapOffset) -> usize {
		let i = self.0.len();

		self.0.push(JsonCodeMapEntry {
			span: Span::new(position, position),
			volume: 0,
		});

		i
	}

	pub(crate) fn get_mut(&mut self, offset: JsonCodeMapOffset) -> Option<&mut JsonCodeMapEntry> {
		self.0.get_mut(offset)
	}

	pub fn iter(&self) -> Iter<'_> {
		self.0.iter().enumerate()
	}
}

impl Deref for JsonCodeMap {
	type Target = [JsonCodeMapEntry];

	fn deref(&self) -> &Self::Target {
		self.as_slice()
	}
}

impl AsRef<[JsonCodeMapEntry]> for JsonCodeMap {
	fn as_ref(&self) -> &[JsonCodeMapEntry] {
		self.as_slice()
	}
}

impl Borrow<[JsonCodeMapEntry]> for JsonCodeMap {
	fn borrow(&self) -> &[JsonCodeMapEntry] {
		self.as_slice()
	}
}

pub type Iter<'a> = std::iter::Enumerate<std::slice::Iter<'a, JsonCodeMapEntry>>;

pub type IntoIter = std::iter::Enumerate<std::vec::IntoIter<JsonCodeMapEntry>>;

impl<'a> IntoIterator for &'a JsonCodeMap {
	type IntoIter = Iter<'a>;
	type Item = (JsonCodeMapOffset, &'a JsonCodeMapEntry);

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl IntoIterator for JsonCodeMap {
	type IntoIter = IntoIter;
	type Item = (JsonCodeMapOffset, JsonCodeMapEntry);

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter().enumerate()
	}
}

/// Code-map entry.
///
/// Provides code-mapping metadata about a fragment of JSON value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonCodeMapEntry {
	/// Byte span of the fragment in the original source code.
	pub span: Span,

	/// Number of sub-fragment (including the fragment itself).
	pub volume: usize,
}

impl JsonCodeMapEntry {
	pub fn new(span: Span, volume: usize) -> Self {
		Self { span, volume }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonMapped<T>(pub T, pub JsonCodeMapOffset);

impl<T> JsonMapped<T> {
	pub fn offset(&self) -> JsonCodeMapOffset {
		self.1
	}

	pub fn into_offset(self) -> JsonCodeMapOffset {
		self.1
	}

	pub fn inner(&self) -> &T {
		&self.0
	}

	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T: fmt::Display> fmt::Display for JsonMapped<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

impl<T: 'static + std::error::Error> std::error::Error for JsonMapped<T> {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		Some(&self.0)
	}
}

// ── Helpers for `JsonCodeMap::from_print` ───────────────────────────────────

fn indent_bytes(indent: Indent, level: usize) -> usize {
	match indent {
		Indent::Spaces(n) => n as usize * level,
		Indent::Tabs(n) => n as usize * level, // \t is one byte
	}
}

#[cfg(test)]
mod tests {
	use super::{JsonCodeMap, JsonCodeMapEntry};
	use crate::{JsonParse, JsonPrint, JsonValue, print::JsonPrintOptions};
	use locspan::Span;

	/// Assert that `JsonCodeMap::from_print` agrees with print-then-parse
	/// for the given value and options.
	fn check(value: &JsonValue, options: &JsonPrintOptions) {
		let printed = value.print_with(options).to_string();
		let (_, expected) = JsonValue::parse_str(&printed).unwrap();
		let got = JsonCodeMap::from_value(value, options);
		assert_eq!(
			got.as_slice(),
			expected.as_slice(),
			"mismatch for printed: {printed:?}"
		);
	}

	#[test]
	fn from_value_compact() {
		// Scalars
		for src in ["null", "true", "false", "42", "-3.14", "\"hello\""] {
			let (value, _) = JsonValue::parse_str(src).unwrap();
			check(&value, &JsonPrintOptions::COMPACT);
		}

		// Nested structure
		let (value, _) = JsonValue::parse_str(r#"{"a":0,"b":[1,2]}"#).unwrap();
		check(&value, &JsonPrintOptions::COMPACT);
	}

	#[test]
	fn from_value_inline() {
		let (value, _) = JsonValue::parse_str(r#"{"a":0,"b":[1,2]}"#).unwrap();
		check(&value, &JsonPrintOptions::INLINE);
	}

	#[test]
	fn from_value_pretty() {
		let (value, _) = JsonValue::parse_str(r#"{"a":0,"b":[1,2,3,4,5,6,7,8,9,10]}"#).unwrap();
		check(&value, &JsonPrintOptions::PRETTY);
	}

	#[test]
	fn code_map_t1() {
		let (value, code_map) = JsonValue::parse_str(r#"{ "a": 0, "b": [1, 2] }"#).unwrap();
		let expected = [
			JsonCodeMapEntry::new(Span::new(0, 23), 9), // { "a": 0, "b": [1, 2] }
			JsonCodeMapEntry::new(Span::new(2, 8), 3),  // "a": 0
			JsonCodeMapEntry::new(Span::new(2, 5), 1),  // "a"
			JsonCodeMapEntry::new(Span::new(7, 8), 1),  // 0
			JsonCodeMapEntry::new(Span::new(10, 21), 5), // "b": [1, 2]
			JsonCodeMapEntry::new(Span::new(10, 13), 1), // "b"
			JsonCodeMapEntry::new(Span::new(15, 21), 3), // [1, 2]
			JsonCodeMapEntry::new(Span::new(16, 17), 1), // 1
			JsonCodeMapEntry::new(Span::new(19, 20), 1), // 2
		];

		assert_eq!(code_map.len(), expected.len());
		assert_eq!(value.traverse().count(), expected.len());
		for (i, entry) in code_map {
			assert_eq!(entry, expected[i])
		}
	}

	#[test]
	fn code_map_t2() {
		let (value, code_map) =
			JsonValue::parse_str(r#"{ "a": 0, "b": { "c": 1, "d": [2, 3] }, "e": [4, [5, 6]] }"#)
				.unwrap();
		let expected = [
			JsonCodeMapEntry::new(Span::new(0, 58), 22), // { "a": 0, "b": { "c": 1, "d": [2, 3] }, "e": [4, [5, 6]] }
			JsonCodeMapEntry::new(Span::new(2, 8), 3),   // "a": 0
			JsonCodeMapEntry::new(Span::new(2, 5), 1),   // "a"
			JsonCodeMapEntry::new(Span::new(7, 8), 1),   // 0
			JsonCodeMapEntry::new(Span::new(10, 38), 11), // "b": { "c": 1, "d": [2, 3] }
			JsonCodeMapEntry::new(Span::new(10, 13), 1), // "b"
			JsonCodeMapEntry::new(Span::new(15, 38), 9), // { "c": 1, "d": [2, 3] }
			JsonCodeMapEntry::new(Span::new(17, 23), 3), // "c": 1
			JsonCodeMapEntry::new(Span::new(17, 20), 1), // "c"
			JsonCodeMapEntry::new(Span::new(22, 23), 1), // 1
			JsonCodeMapEntry::new(Span::new(25, 36), 5), // "d": [2, 3]
			JsonCodeMapEntry::new(Span::new(25, 28), 1), // "d"
			JsonCodeMapEntry::new(Span::new(30, 36), 3), // [2, 3]
			JsonCodeMapEntry::new(Span::new(31, 32), 1), // 2
			JsonCodeMapEntry::new(Span::new(34, 35), 1), // 3
			JsonCodeMapEntry::new(Span::new(40, 56), 7), // "e": [4, [5, 6]]
			JsonCodeMapEntry::new(Span::new(40, 43), 1), // "e"
			JsonCodeMapEntry::new(Span::new(45, 56), 5), // [4, [5, 6]]
			JsonCodeMapEntry::new(Span::new(46, 47), 1), // 4
			JsonCodeMapEntry::new(Span::new(49, 55), 3), // [5, 6]
			JsonCodeMapEntry::new(Span::new(50, 51), 1), // 5
			JsonCodeMapEntry::new(Span::new(53, 54), 1), // 6
		];

		assert_eq!(code_map.len(), expected.len());
		assert_eq!(value.traverse().count(), expected.len());
		for (i, entry) in code_map {
			assert_eq!(entry, expected[i])
		}
	}
}
