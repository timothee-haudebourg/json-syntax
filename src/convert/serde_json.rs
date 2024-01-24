use crate::{object::Entry, Value};

impl Value {
	/// Converts a [`serde_json::Value`] into a `Value`.
	///
	/// # Example
	///
	/// ```
	/// // First we create a `serde_json` value.
	/// let a = serde_json::json!({
	///   "foo": 1,
	///   "bar": [2, 3]
	/// });
	///
	/// // We convert the `serde_json` value into a `json_syntax` value.
	/// let b = json_syntax::Value::from_serde_json(a);
	///
	/// // We convert it back into a `serde_json` value.
	/// let _ = json_syntax::Value::into_serde_json(b);
	/// ```
	pub fn from_serde_json(value: serde_json::Value) -> Self {
		match value {
			serde_json::Value::Null => Self::Null,
			serde_json::Value::Bool(b) => Self::Boolean(b),
			serde_json::Value::Number(n) => Self::Number(n.into()),
			serde_json::Value::String(s) => Self::String(s.into()),
			serde_json::Value::Array(a) => {
				Self::Array(a.into_iter().map(Self::from_serde_json).collect())
			}
			serde_json::Value::Object(o) => Self::Object(
				o.into_iter()
					.map(|(k, v)| Entry::new(k.into(), Self::from_serde_json(v)))
					.collect(),
			),
		}
	}

	/// Converts a `Value` into a [`serde_json::Value`].
	///
	/// # Example
	///
	/// ```
	/// // First we create a `serde_json` value.
	/// let a = serde_json::json!({
	///   "foo": 1,
	///   "bar": [2, 3]
	/// });
	///
	/// // We convert the `serde_json` value into a `json_syntax` value.
	/// let b = json_syntax::Value::from_serde_json(a);
	///
	/// // We convert it back into a `serde_json` value.
	/// let _ = json_syntax::Value::into_serde_json(b);
	/// ```
	pub fn into_serde_json(self) -> serde_json::Value {
		match self {
			Self::Null => serde_json::Value::Null,
			Self::Boolean(b) => serde_json::Value::Bool(b),
			Self::Number(n) => serde_json::Value::Number(n.into()),
			Self::String(s) => serde_json::Value::String(s.into_string()),
			Self::Array(a) => {
				serde_json::Value::Array(a.into_iter().map(Value::into_serde_json).collect())
			}
			Self::Object(o) => serde_json::Value::Object(
				o.into_iter()
					.map(|Entry { key, value }| (key.into_string(), Value::into_serde_json(value)))
					.collect(),
			),
		}
	}
}

impl From<serde_json::Value> for Value {
	#[inline(always)]
	fn from(value: serde_json::Value) -> Self {
		Self::from_serde_json(value)
	}
}

impl From<Value> for serde_json::Value {
	fn from(value: Value) -> Self {
		value.into_serde_json()
	}
}
