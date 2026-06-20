use smallstr::SmallString;

use crate::{JsonBytes, JsonNumber, JsonNumberBuf, JsonObject, JsonValue};

pub trait JsonVisitor {
	type Output;

	fn visit_null(self) -> Self::Output;

	fn visit_bool(self, value: bool) -> Self::Output;

	fn visit_number(self, value: &JsonNumber) -> Self::Output;

	fn visit_string(self, value: &str) -> Self::Output;

	fn visit_array<I: IntoIterator<Item: VisitJson>>(self, items: I) -> Self::Output;

	fn visit_object<E: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: VisitJson>(
		self,
		entries: E,
	) -> Self::Output;
}

pub trait VisitJson {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output;
}

impl<T: VisitJson> VisitJson for &T {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		T::visit(*self, visitor)
	}
}

impl VisitJson for () {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_null()
	}
}

impl VisitJson for bool {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_bool(*self)
	}
}

impl VisitJson for str {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl VisitJson for String {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl<A: smallvec::Array<Item = u8>> VisitJson for SmallString<A> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl VisitJson for JsonNumber {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_number(self)
	}
}

impl<B: JsonBytes> VisitJson for JsonNumberBuf<B> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_number(self)
	}
}

impl<T: VisitJson> VisitJson for [T] {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_array(self)
	}
}

impl<T: VisitJson> VisitJson for Vec<T> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_array(self)
	}
}

impl VisitJson for JsonObject {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_object(self)
	}
}

impl VisitJson for JsonValue {
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
