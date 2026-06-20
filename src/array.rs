use crate::{code_map::Mapped, CodeMap, JsonFragment, JsonValue};

/// JSON array slice.
pub type JsonArray = [JsonValue];

/// Owned JSON array.
pub type JsonArrayBuf = Vec<JsonValue>;

/// Trait for JSON array types like `Vec<Value>` and `[Value]`.
pub trait JsonArrayExt {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm>;

	fn get_fragment(&self, index: usize) -> Result<JsonFragment<'_>, usize>;
}

/// Trait for owned JSON array types like `Vec<Value>`.
pub trait JsonArrayBufExt: Sized + JsonArrayExt {
	fn into_iter_mapped<'m>(self, code_map: &'m CodeMap, offset: usize) -> IntoIterMapped<'m>;
}

impl JsonArrayExt for JsonArray {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			items: self.iter(),
			code_map,
			offset: offset + 1,
		}
	}

	fn get_fragment(&self, mut index: usize) -> Result<JsonFragment<'_>, usize> {
		for v in self {
			match v.get_fragment(index) {
				Ok(value) => return Ok(value),
				Err(i) => index = i,
			}
		}

		Err(index)
	}
}

impl JsonArrayExt for JsonArrayBuf {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		self.as_slice().iter_mapped(code_map, offset)
	}

	fn get_fragment(&self, index: usize) -> Result<JsonFragment<'_>, usize> {
		self.as_slice().get_fragment(index)
	}
}

impl JsonArrayBufExt for JsonArrayBuf {
	fn into_iter_mapped<'m>(self, code_map: &'m CodeMap, offset: usize) -> IntoIterMapped<'m> {
		IntoIterMapped {
			items: self.into_iter(),
			code_map,
			offset: offset + 1,
		}
	}
}

pub struct IterMapped<'a, 'm> {
	items: std::slice::Iter<'a, JsonValue>,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'a, 'm> Iterator for IterMapped<'a, 'm> {
	type Item = Mapped<&'a JsonValue>;

	fn next(&mut self) -> Option<Self::Item> {
		self.items.next().map(|item| {
			let offset = self.offset;
			self.offset += self.code_map.get(self.offset).unwrap().volume;
			Mapped(item, offset)
		})
	}
}

pub struct IntoIterMapped<'m> {
	items: std::vec::IntoIter<JsonValue>,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'m> Iterator for IntoIterMapped<'m> {
	type Item = Mapped<JsonValue>;

	fn next(&mut self) -> Option<Self::Item> {
		self.items.next().map(|item| {
			let offset = self.offset;
			self.offset += self.code_map.get(self.offset).unwrap().volume;
			Mapped(item, offset)
		})
	}
}
