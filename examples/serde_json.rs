//! This example shows how to convert a `serde_json::Value` into/from a
//! `json_syntax::Value` using the `serde_json` feature.

fn main() {
	// First we create a `serde_json` value.
	let a = serde_json::json!({
		"foo": 1,
		"bar": [2, 3]
	});

	// We convert the `serde_json` value into a `json_syntax` value.
	let b = json_syntax::Value::from_serde_json(a);

	// We convert it back into a `serde_json` value.
	let _ = json_syntax::Value::into_serde_json(b);
}
