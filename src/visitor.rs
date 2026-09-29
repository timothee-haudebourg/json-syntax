use smallstr::SmallString;

use crate::{JsonNumber, JsonNumberBuf, JsonObject, JsonValue};

pub trait JsonVisitor {
	type Output;

	fn visit_null(self) -> Self::Output;

	fn visit_bool(self, value: bool) -> Self::Output;

	fn visit_number(self, value: &JsonNumber) -> Self::Output;

	fn visit_string(self, value: &str) -> Self::Output;

	fn visit_array<I: IntoIterator<Item: JsonVisit>>(self, items: I) -> Self::Output;

	fn visit_object<E: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: JsonVisit>(
		self,
		entries: E,
	) -> Self::Output;
}

pub trait JsonVisit {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output;
}

impl<T: JsonVisit> JsonVisit for &T {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		T::visit(*self, visitor)
	}
}

impl JsonVisit for () {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_null()
	}
}

impl JsonVisit for bool {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_bool(*self)
	}
}

impl JsonVisit for str {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl JsonVisit for String {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl<A: smallvec::Array<Item = u8>> JsonVisit for SmallString<A> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl JsonVisit for JsonNumber {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_number(self)
	}
}

impl JsonVisit for JsonNumberBuf {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_number(self)
	}
}

impl<T: JsonVisit> JsonVisit for [T] {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_array(self)
	}
}

impl<T: JsonVisit> JsonVisit for Vec<T> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_array(self)
	}
}

impl JsonVisit for JsonObject {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_object(self)
	}
}

impl JsonVisit for JsonValue {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		match self {
			Self::Null => ().visit(visitor),
			Self::Boolean(b) => b.visit(visitor),
			Self::Number(n) => n.visit(visitor),
			Self::String(s) => s.visit(visitor),
			Self::Array(a) => a.visit(visitor),
			Self::Object(o) => o.visit(visitor),
		}
	}
}
