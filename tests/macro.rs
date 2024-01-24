use json_syntax::{
	json,
	object::{Entry, Key},
	Object, Value,
};

#[test]
fn macro_01() {
	let value = json! {
		null
	};

	assert_eq!(value, Value::Null)
}

#[test]
fn macro_02() {
	let value = json! {
		true
	};

	assert_eq!(value, Value::Boolean(true))
}

#[test]
fn macro_03() {
	let value = json! {
		false
	};

	assert_eq!(value, Value::Boolean(false))
}

#[test]
fn macro_04() {
	let value = json! {
		[]
	};

	assert_eq!(value, Value::Array(vec![]))
}

#[test]
fn macro_05() {
	let value = json! {
		{}
	};

	assert_eq!(value, Value::Object(Object::default()))
}

#[test]
fn macro_06() {
	let value = json! {
		[ null ]
	};

	assert_eq!(value, Value::Array(vec![Value::Null]))
}

#[test]
fn macro_07() {
	let value = json! {
		{ "foo": null }
	};

	assert_eq!(
		value,
		Value::Object(vec![Entry::new("foo".into(), Value::Null)].into())
	)
}

#[test]
fn macro_08() {
	let item = json! { null };
	let value = json! {
		[ item ]
	};

	assert_eq!(value, Value::Array(vec![Value::Null]))
}

#[test]
fn macro_09() {
	let value = json! {
		[ [ null ], true, false ]
	};

	assert_eq!(
		value,
		Value::Array(vec![
			Value::Array(vec![Value::Null]),
			Value::Boolean(true),
			Value::Boolean(false)
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
		Value::Object(Object::from_vec(vec![
			Entry::new("a".into(), Value::Boolean(true)),
			Entry::new("b".into(), Value::Boolean(false))
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
		Value::Object(Object::from_vec(vec![
			Entry::new("a".into(), Value::Boolean(true)),
			Entry::new("b".into(), Value::Boolean(false))
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
		Value::Object(Object::from_vec(vec![
			Entry::new("a".into(), Value::Boolean(true)),
			Entry::new("b".into(), Value::Object(Object::default())),
			Entry::new("c".into(), Value::Boolean(false))
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
		Value::Object(Object::from_vec(vec![
			Entry::new("a".into(), Value::Boolean(true)),
			Entry::new("b".into(), Value::Object(Object::default())),
			Entry::new("c".into(), Value::Boolean(false))
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
		Value::Object(Object::from_vec(vec![
			Entry::new("a".into(), Value::Number(0.1f32.try_into().unwrap())),
			Entry::new("b".into(), Value::Number(1.1e10f32.try_into().unwrap()))
		]))
	)
}
