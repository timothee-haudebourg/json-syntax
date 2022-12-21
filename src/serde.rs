use crate::Value;
use locspan::Meta;
use serde::{de::DeserializeOwned, Serialize};

mod de;
mod ser;

pub use de::*;
pub use ser::*;

/// Serializes the given `value` into a JSON [`Value`], with `()` as metadata.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use json_syntax::{json, Value};
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
/// let expected: Value = json!({
///   "fingerprint": "0xF9BA143B95FF6D82",
///   "location": "Menlo Park, CA",
/// }).into_value();
///
/// let v = json_syntax::to_value(u).unwrap();
/// assert_eq!(v, expected);
/// ```
pub fn to_value<T>(value: T) -> Result<Value, SerializeError>
where
	T: Serialize,
{
	Ok(value.serialize(Serializer::new(|| ()))?.into_value())
}

/// Serializes the given `value` into a JSON [`Value<M>`] using the input
/// `metadata` function to annotate the output value.
pub fn to_value_with<T, M>(
	value: T,
	metadata: impl Clone + Fn() -> M,
) -> Result<Meta<Value<M>, M>, SerializeError>
where
	T: Serialize,
{
	value.serialize(Serializer::new(metadata))
}

/// Deserializes the JSON `value` into an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use json_syntax::{json, Value};
///
/// #[derive(Deserialize, Debug)]
/// struct User {
///     fingerprint: String,
///     location: String,
/// }
///
/// let j: Value = json!({
///   "fingerprint": "0xF9BA143B95FF6D82",
///   "location": "Menlo Park, CA"
/// }).into_value();
///
/// let u: User = json_syntax::from_value(j).unwrap();
/// println!("{:#?}", u);
/// ```
pub fn from_value<T, M>(value: Value<M>) -> Result<T, DeserializeError>
where
	T: DeserializeOwned,
{
	T::deserialize(value)
}
