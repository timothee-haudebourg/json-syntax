//! This example shows how to convert a `serde_json::Value` into/from a
//! `json_syntax::Value` using the `serde_json` feature.

fn main() {
	// First we create a `serde_json` value.
	let a = serde_json::json!({
		"foo": 1,
		"bar": [2, 3]
	});

	// We convert the `serde_json` value into a `json_syntax` value.
	let b = json_syntax::Value::from_serde_json(a, |fragment| {
		// `json_syntax` keeps metadata information attached to every value
		// fragment, which `serde_json` does not. This is why this closure is
		// necessary. It is called for every `serde_json` fragment to let you
		// choose the metadata you want to associate to the fragment.
		// This is intended to store code mapping information, but you can store
		// any information. Here we store a text description of the fragment.
		match fragment {
			json_syntax::SerdeJsonFragment::Key(key) => {
				format!("I'm an object key `{key}`")
			}
			json_syntax::SerdeJsonFragment::Value(value) => match value {
				serde_json::Value::Null => "I'm the `null` value".to_string(),
				serde_json::Value::Bool(b) => format!("I'm the boolean `{b:?}`"),
				serde_json::Value::Number(n) => format!("I'm the number {n}"),
				serde_json::Value::String(s) => format!("I'm the string {s:?}"),
				serde_json::Value::Array(a) => format!("I'm an array of {} elements", a.len()),
				serde_json::Value::Object(o) => format!("I'm an object of {} entries", o.len()),
			},
		}
	});

	// We convert it back into a `serde_json` value.
	let _ = json_syntax::Value::into_serde_json(b);
}
