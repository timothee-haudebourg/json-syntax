//! This example shows how to serialize and deserialize `json_syntax::Value`
//! using the `serde` crate. This will not allow you to attach metadata to each
//! value fragment (the `M` type will be unit `()`).
use json_syntax::Print;

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct MyType {
	foo: String,
	bar: Vec<u32>,
}

fn main() {
	// Instantiate our type.
	let a = MyType {
		foo: "Hello World!".to_string(),
		bar: vec![1, 2, 3],
	};

	// Serialize `a` into a JSON value.
	let json = json_syntax::to_value(&a).expect("serialization failed");

	// Print the value.
	println!("{}", json.pretty_print());

	// Deserialize JSON back into `MyType`.
	let b: MyType = json_syntax::from_value(json).expect("deserialization failed");

	// The round-trip should not change the actual data.
	assert_eq!(a, b)
}
