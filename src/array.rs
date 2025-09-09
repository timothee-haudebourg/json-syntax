use crate::{code_map::Mapped, CodeMap, Value};

/// Array.
pub type Array = Vec<Value>;

/// Trait for JSON array types like `Vec<Value>` and `[Value]`.
pub trait JsonSlice {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm>;
}

/// Trait for owned JSON array types like `Vec<Value>`.
pub trait JsonArray: Sized + JsonSlice {
	fn into_iter_mapped<'m>(self, code_map: &'m CodeMap, offset: usize) -> IntoIterMapped<'m>;
}

impl JsonSlice for [Value] {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			items: self.iter(),
			code_map,
			offset: offset + 1,
		}
	}
}

impl JsonSlice for Vec<Value> {
	fn iter_mapped<'m>(&self, code_map: &'m CodeMap, offset: usize) -> IterMapped<'_, 'm> {
		IterMapped {
			items: self.iter(),
			code_map,
			offset: offset + 1,
		}
	}
}

impl JsonArray for Vec<Value> {
	fn into_iter_mapped<'m>(self, code_map: &'m CodeMap, offset: usize) -> IntoIterMapped<'m> {
		IntoIterMapped {
			items: self.into_iter(),
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
			Mapped(item, offset)
		})
	}
}

pub struct IntoIterMapped<'m> {
	items: std::vec::IntoIter<Value>,
	code_map: &'m CodeMap,
	offset: usize,
}

impl<'m> Iterator for IntoIterMapped<'m> {
	type Item = Mapped<Value>;

	fn next(&mut self) -> Option<Self::Item> {
		self.items.next().map(|item| {
			let offset = self.offset;
			self.offset += self.code_map.get(self.offset).unwrap().volume;
			Mapped(item, offset)
		})
	}
}
