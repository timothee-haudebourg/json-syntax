use crate::{code_map::Mapped, CodeMap, Value};

/// Array.
pub type Array = Vec<Value>;

/// Trait for JSON array types like `Vec<Value>` and `[Value]`.
pub trait JsonArray {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm>;
}

impl JsonArray for [Value] {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			items: self.iter(),
			code_map,
			offset: offset + 1,
		}
	}
}

impl JsonArray for Vec<Value> {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			items: self.iter(),
			code_map,
			offset: offset + 1,
		}
	}
}

pub struct IterMapped<'a, 'm> {
	items: std::slice::Iter<'a, Value>,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'a, 'm> Iterator for IterMapped<'a, 'm> {
	type Item = Mapped<&'a Value>;

	fn next(&mut self) -> Option<Self::Item> {
		self.items.next().map(|item| {
			let offset = self.offset;
			self.offset += self.code_map.get(self.offset).unwrap().volume;
			Mapped::new(offset, item)
		})
	}
}
