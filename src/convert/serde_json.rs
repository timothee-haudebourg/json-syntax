use locspan::Meta;

use crate::{object::Entry, Value};

/// A fragment of a [`serde_json::Value`].
///
/// Used by the [`Value::from_serde_json`] to annotate the output value.
pub enum SerdeJsonFragment<'a> {
	Key(&'a str),
	Value(&'a serde_json::Value),
}

impl<M> Value<M> {
	/// Converts a [`serde_json::Value`] into a `Value`.
	///
	/// The function `f` is used to annotate the output each sub value
	/// (passed as parameter).
	pub fn from_serde_json(
		value: serde_json::Value,
		f: impl Clone + Fn(SerdeJsonFragment) -> M,
	) -> Meta<Self, M> {
		let meta = f(SerdeJsonFragment::Value(&value));

		let v = match value {
			serde_json::Value::Null => Self::Null,
			serde_json::Value::Bool(b) => Self::Boolean(b),
			serde_json::Value::Number(n) => Self::Number(n.into()),
			serde_json::Value::String(s) => Self::String(s.into()),
			serde_json::Value::Array(a) => Self::Array(
				a.into_iter()
					.map(|i| Self::from_serde_json(i, f.clone()))
					.collect(),
			),
			serde_json::Value::Object(o) => Self::Object(
				o.into_iter()
					.map(|(k, v)| {
						let k_meta = f(SerdeJsonFragment::Key(&k));
						Entry::new(Meta(k.into(), k_meta), Self::from_serde_json(v, f.clone()))
					})
					.collect(),
			),
		};

		Meta(v, meta)
	}

	/// Converts a `Value` into a [`serde_json::Value`], stripping the metadata.
	pub fn into_serde_json(Meta(this, _): Meta<Self, M>) -> serde_json::Value {
		this.into()
	}
}

impl<M: Default> From<serde_json::Value> for Value<M> {
	#[inline(always)]
	fn from(value: serde_json::Value) -> Self {
		Value::from_serde_json(value, |_| M::default()).into_value()
	}
}

impl<M> From<Value<M>> for serde_json::Value {
	fn from(value: Value<M>) -> Self {
		match value {
			Value::Null => Self::Null,
			Value::Boolean(b) => Self::Bool(b),
			Value::Number(n) => Self::Number(n.into()),
			Value::String(s) => Self::String(s.into_string()),
			Value::Array(a) => Self::Array(a.into_iter().map(Value::into_serde_json).collect()),
			Value::Object(o) => Self::Object(
				o.into_iter()
					.map(
						|Entry {
						     key: Meta(key, _),
						     value,
						 }| { (key.into_string(), Value::into_serde_json(value)) },
					)
					.collect(),
			),
		}
	}
}
