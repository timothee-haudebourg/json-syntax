use json_syntax::{json, object::Key, JsonObject, JsonValue};

#[test]
fn macro_01() {
	let value = json! {
		null
	};

	assert_eq!(value, JsonValue::Null)
}

#[test]
fn macro_02() {
	let value = json! {
		true
	};

	assert_eq!(value, JsonValue::Boolean(true))
}

#[test]
fn macro_03() {
	let value = json! {
		false
	};

	assert_eq!(value, JsonValue::Boolean(false))
}

#[test]
fn macro_04() {
	let value = json! {
		[]
	};

	assert_eq!(value, JsonValue::Array(vec![]))
}

#[test]
fn macro_05() {
	let value = json! {
		{}
	};

	assert_eq!(value, JsonValue::Object(JsonObject::default()))
}

#[test]
fn macro_06() {
	let value = json! {
		[ null ]
	};

	assert_eq!(value, JsonValue::Array(vec![JsonValue::Null]))
}

#[test]
fn macro_07() {
	let value = json! {
		{ "foo": null }
	};

	assert_eq!(
		value,
		JsonValue::Object(vec![("foo".into(), JsonValue::Null)].into())
	)
}

#[test]
fn macro_08() {
	let item = json! { null };
	let value = json! {
		[ item ]
	};

	assert_eq!(value, JsonValue::Array(vec![JsonValue::Null]))
}

#[test]
fn macro_09() {
	let value = json! {
		[ [ null ], true, false ]
	};

	assert_eq!(
		value,
		JsonValue::Array(vec![
			JsonValue::Array(vec![JsonValue::Null]),
			JsonValue::Boolean(true),
			JsonValue::Boolean(false)
		])
	)
}

#[test]
fn macro_10() {
	let value = json! {
		{ "a": true, "b": false }
	};

	assert_eq!(
		value,
		JsonValue::Object(JsonObject::from_vec(vec![
			("a".into(), JsonValue::Boolean(true)),
			("b".into(), JsonValue::Boolean(false))
		]))
	)
}

#[test]
fn macro_11() {
	let key = Key::from("a");
	let t = json! { true };

	let value = json! {
		{ key: t, "b": false }
	};

	assert_eq!(
		value,
		JsonValue::Object(JsonObject::from_vec(vec![
			("a".into(), JsonValue::Boolean(true)),
			("b".into(), JsonValue::Boolean(false))
		]))
	)
}

#[test]
fn macro_12() {
	let keys = [Key::from("a"), Key::from("c")];
	let values = [json! { true }, json! { false }];

	let value = json! {
		{ keys[0].clone(): values[0].clone(), "b": {}, keys[1].clone(): values[1].clone() }
	};

	assert_eq!(
		value,
		JsonValue::Object(JsonObject::from_vec(vec![
			("a".into(), JsonValue::Boolean(true)),
			("b".into(), JsonValue::Object(JsonObject::default())),
			("c".into(), JsonValue::Boolean(false))
		]))
	)
}

#[test]
fn macro_13() {
	let keys = [Key::from("a"), Key::from("c")];
	let values = [json! { true }, json! { false }];

	let value = json! {
		{ keys[0].clone(): values[0].clone(), ("b"): {}, keys[1].clone(): values[1].clone() }
	};

	assert_eq!(
		value,
		JsonValue::Object(JsonObject::from_vec(vec![
			("a".into(), JsonValue::Boolean(true)),
			("b".into(), JsonValue::Object(JsonObject::default())),
			("c".into(), JsonValue::Boolean(false))
		]))
	)
}

#[test]
fn macro_14() {
	let value = json! {
		{ "a": 0.1f32, "b": 1.1e10f32 }
	};

	assert_eq!(
		value,
		JsonValue::Object(JsonObject::from_vec(vec![
			("a".into(), JsonValue::Number(0.1f32.try_into().unwrap())),
			("b".into(), JsonValue::Number(1.1e10f32.try_into().unwrap()))
		]))
	)
}
