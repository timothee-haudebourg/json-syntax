use crate::{JsonBytes, JsonNumber, JsonNumberBuf, serde::NUMBER_TOKEN};
use ser::{Serialize, Serializer};
use serde::ser;

impl Serialize for JsonNumber {
	#[inline]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		if let Some(v) = self.as_i64() {
			serializer.serialize_i64(v)
		} else if let Some(v) = self.as_u64() {
			serializer.serialize_u64(v)
		} else {
			use serde::ser::SerializeStruct;
			let mut s = serializer.serialize_struct(NUMBER_TOKEN, 1)?;
			s.serialize_field(NUMBER_TOKEN, self.as_str())?;
			s.end()
		}
	}
}

impl<B: JsonBytes> Serialize for JsonNumberBuf<B> {
	#[inline]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		self.as_number().serialize(serializer)
	}
}
