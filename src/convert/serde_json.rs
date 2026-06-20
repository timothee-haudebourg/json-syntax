use crate::{JsonBytes, JsonNumber, JsonNumberBuf, JsonValue};

impl JsonValue {
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
	/// let b = JsonValue::from_serde_json(a);
	///
	/// // We convert it back into a `serde_json` value.
	/// let _ = JsonValue::into_serde_json(b);
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
					.map(|(k, v)| (k.into(), Self::from_serde_json(v)))
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
	/// let b = JsonValue::from_serde_json(a);
	///
	/// // We convert it back into a `serde_json` value.
	/// let _ = JsonValue::into_serde_json(b);
	/// ```
	pub fn into_serde_json(self) -> serde_json::Value {
		match self {
			Self::Null => serde_json::Value::Null,
			Self::Boolean(b) => serde_json::Value::Bool(b),
			Self::Number(n) => serde_json::Value::Number(n.into()),
			Self::String(s) => serde_json::Value::String(s.into_string()),
			Self::Array(a) => {
				serde_json::Value::Array(a.into_iter().map(JsonValue::into_serde_json).collect())
			}
			Self::Object(o) => serde_json::Value::Object(
				o.into_iter()
					.map(|(key, value)| (key.into_string(), JsonValue::into_serde_json(value)))
					.collect(),
			),
		}
	}
}

impl From<serde_json::Value> for JsonValue {
	#[inline(always)]
	fn from(value: serde_json::Value) -> Self {
		Self::from_serde_json(value)
	}
}

impl From<JsonValue> for serde_json::Value {
	fn from(value: JsonValue) -> Self {
		value.into_serde_json()
	}
}

impl<B: JsonBytes> From<serde_json::Number> for JsonNumberBuf<B> {
	#[inline(always)]
	fn from(n: serde_json::Number) -> Self {
		JsonNumberBuf::new(B::from_vec(n.to_string().into_bytes()))
			.ok()
			.expect("invalid `serde_json::Number`")
	}
}

impl<B: JsonBytes> From<JsonNumberBuf<B>> for serde_json::Number {
	#[inline(always)]
	fn from(n: JsonNumberBuf<B>) -> Self {
		Self::from(n.as_number())
	}
}

impl<'n> From<&'n JsonNumber> for serde_json::Number {
	fn from(n: &'n JsonNumber) -> Self {
		if let Some(u) = n.as_u64() {
			u.into()
		} else if let Some(i) = n.as_i64() {
			i.into()
		} else {
			match n.as_str().parse() {
				Ok(n) => n,
				Err(_) => Self::from_f64(n.as_f64_lossy()).unwrap(),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::JsonNumberBuf;

	#[test]
	fn serde_json_arbitrary_number_precision_compatibility() {
		let n = JsonNumberBuf::new("1.1".to_owned().into_bytes()).unwrap();
		let serde_json::Value::Number(serde_json_n) = serde_json::to_value(n.clone()).unwrap()
		else {
			panic!("not a number")
		};

		let m: JsonNumberBuf = serde_json_n.into();
		assert_eq!(n, m)
	}
}
