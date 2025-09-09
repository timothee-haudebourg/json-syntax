use json_number::Number;
use smallstr::SmallString;

use crate::{Object, Value};

pub trait JsonVisitor {
	type Output;

	fn visit_null(self) -> Self::Output;

	fn visit_bool(self, value: bool) -> Self::Output;

	fn visit_number(self, value: &Number) -> Self::Output;

	fn visit_string(self, value: &str) -> Self::Output;

	fn visit_array<I: IntoIterator<Item: JsonValue>>(self, items: I) -> Self::Output;

	fn visit_object<E: IntoIterator<Item = (K, V)>, K: AsRef<str>, V: JsonValue>(
		self,
		entries: E,
	) -> Self::Output;
}

pub trait JsonValue {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output;
}

impl<T: JsonValue> JsonValue for &T {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		T::visit(*self, visitor)
	}
}

impl JsonValue for () {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_null()
	}
}

impl JsonValue for bool {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_bool(*self)
	}
}

impl JsonValue for str {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl JsonValue for String {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl<A: smallvec::Array<Item = u8>> JsonValue for SmallString<A> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_string(self)
	}
}

impl JsonValue for Number {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_number(self)
	}
}

impl<B: json_number::Buffer> JsonValue for json_number::NumberBuf<B> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_number(self)
	}
}

impl<T: JsonValue> JsonValue for [T] {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_array(self)
	}
}

impl<T: JsonValue> JsonValue for Vec<T> {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_array(self)
	}
}

impl JsonValue for Object {
	fn visit<V: JsonVisitor>(&self, visitor: V) -> V::Output {
		visitor.visit_object(self)
	}
}

impl JsonValue for Value {
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
