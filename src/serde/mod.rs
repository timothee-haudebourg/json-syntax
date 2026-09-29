use core::fmt;

use crate::{JsonParse, JsonValue};
use serde::{
	Serialize,
	de::{DeserializeOwned, Error},
};

mod de;
mod ser;

pub use de::*;
pub use ser::*;

const NUMBER_TOKEN: &str = "$serde_json::private::Number";

#[repr(transparent)]
pub struct JsonSerdeAdaptor<T: ?Sized>(pub T);

impl<T: ?Sized> JsonSerdeAdaptor<T> {
	pub fn new_ref(value: &T) -> &Self {
		unsafe { std::mem::transmute(value) }
	}
}

impl<T> JsonSerdeAdaptor<T> {
	pub fn unwrap(self) -> T {
		self.0
	}
}

impl<E: fmt::Debug> fmt::Debug for JsonSerdeAdaptor<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		fmt::Debug::fmt(&self.0, f)
	}
}

impl<E: fmt::Display> fmt::Display for JsonSerdeAdaptor<E> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}

impl<E: std::error::Error> std::error::Error for JsonSerdeAdaptor<E> {}

/// Serializes the given `value` into a JSON [`JsonValue`].
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use json_syntax::{json, JsonValue};
///
/// #[derive(Serialize)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// let u = User {
///   fingerprint: "0xF9BA143B95FF6D82".to_owned(),
///   location: "Menlo Park, CA".to_owned(),
/// };
///
/// let expected: JsonValue = json!({
///   "fingerprint": "0xF9BA143B95FF6D82",
///   "location": "Menlo Park, CA",
/// });
///
/// let v = json_syntax::to_value(u).unwrap();
/// assert_eq!(v, expected);
/// ```
pub fn to_value<T>(value: T) -> Result<JsonValue, SerializeError>
where
	T: Serialize,
{
	value.serialize(Serializer)
}

/// Deserializes the JSON `value` into an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use json_syntax::{json, JsonValue};
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// let j: JsonValue = json!({
///   "fingerprint": "0xF9BA143B95FF6D82",
///   "location": "Menlo Park, CA"
/// });
///
/// let u: User = json_syntax::from_value(j).unwrap();
/// println!("{:#?}", u);
/// ```
pub fn from_value<T>(value: JsonValue) -> Result<T, DeserializeError>
where
	T: DeserializeOwned,
{
	T::deserialize(value)
}

pub fn from_str<T>(s: &str) -> Result<T, DeserializeError>
where
	T: DeserializeOwned,
{
	let (json, _) = JsonValue::parse_str(s).map_err(DeserializeError::custom)?;
	from_value(json)
}

pub fn from_slice<T>(s: &[u8]) -> Result<T, DeserializeError>
where
	T: DeserializeOwned,
{
	let (json, _) = JsonValue::parse_slice(s).map_err(DeserializeError::custom)?;
	from_value(json)
}
