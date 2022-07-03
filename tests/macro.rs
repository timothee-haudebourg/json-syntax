use json_syntax::{json, Entry, Key, Value};
use locspan::Meta;

#[test]
fn macro_01() {
	let value: Meta<Value<()>, ()> = json! {
		null
	};

	assert_eq!(value, Meta(Value::Null, ()))
}

#[test]
fn macro_02() {
	let value: Meta<Value<()>, ()> = json! {
		true
	};

	assert_eq!(value, Meta(Value::Boolean(true), ()))
}

#[test]
fn macro_03() {
	let value: Meta<Value<()>, ()> = json! {
		false
	};

	assert_eq!(value, Meta(Value::Boolean(false), ()))
}

#[test]
fn macro_04() {
	let value: Meta<Value<()>, ()> = json! {
		[]
	};

	assert_eq!(value, Meta(Value::Array(vec![]), ()))
}

#[test]
fn macro_05() {
	let value: Meta<Value<()>, ()> = json! {
		{}
	};

	assert_eq!(value, Meta(Value::Object(vec![]), ()))
}

#[test]
fn macro_06() {
	let value: Meta<Value<()>, ()> = json! {
		[ null ]
	};

	assert_eq!(value, Meta(Value::Array(vec![Meta(Value::Null, ())]), ()))
}

#[test]
fn macro_07() {
	let value: Meta<Value<()>, ()> = json! {
		{ "foo": null }
	};

	assert_eq!(
		value,
		Meta(
			Value::Object(vec![Entry::new(
				Meta("foo".into(), ()),
				Meta(Value::Null, ())
			)]),
			()
		)
	)
}

#[test]
fn macro_08() {
	let item = json! { null };
	let value: Meta<Value<()>, ()> = json! {
		[ item ]
	};

	assert_eq!(value, Meta(Value::Array(vec![Meta(Value::Null, ())]), ()))
}

#[test]
fn macro_09() {
	let value: Meta<Value<()>, ()> = json! {
		[ [ null ], true, false ]
	};

	assert_eq!(
		value,
		Meta(
			Value::Array(vec![
				Meta(Value::Array(vec![Meta(Value::Null, ())]), ()),
				Meta(Value::Boolean(true), ()),
				Meta(Value::Boolean(false), ())
			]),
			()
		)
	)
}

#[test]
fn macro_10() {
	let value: Meta<Value<()>, ()> = json! {
		{ "a": true, "b": false }
	};

	assert_eq!(
		value,
		Meta(
			Value::Object(vec![
				Entry::new(Meta("a".into(), ()), Meta(Value::Boolean(true), ())),
				Entry::new(Meta("b".into(), ()), Meta(Value::Boolean(false), ()))
			]),
			()
		)
	)
}

#[test]
fn macro_11() {
	let key = Meta(Key::from("a"), ());
	let t = json! { true };

	let value: Meta<Value<()>, ()> = json! {
		{ key: t, "b": false }
	};

	assert_eq!(
		value,
		Meta(
			Value::Object(vec![
				Entry::new(Meta("a".into(), ()), Meta(Value::Boolean(true), ())),
				Entry::new(Meta("b".into(), ()), Meta(Value::Boolean(false), ()))
			]),
			()
		)
	)
}

#[test]
fn macro_12() {
	let keys = [Meta(Key::from("a"), ()), Meta(Key::from("c"), ())];
	let values = [json! { true }, json! { false }];

	let value: Meta<Value<()>, ()> = json! {
		{ keys[0].clone(): values[0].clone(), "b": {}, keys[1].clone(): values[1].clone() }
	};

	assert_eq!(
		value,
		Meta(
			Value::Object(vec![
				Entry::new(Meta("a".into(), ()), Meta(Value::Boolean(true), ())),
				Entry::new(Meta("b".into(), ()), Meta(Value::Object(vec![]), ())),
				Entry::new(Meta("c".into(), ()), Meta(Value::Boolean(false), ()))
			]),
			()
		)
	)
}

#[test]
fn macro_13() {
	let keys = [Meta(Key::from("a"), 1), Meta(Key::from("c"), 5)];
	let values = [json! { true @ 2 }, json! { false @ 6 }];

	let value: Meta<Value<u32>, u32> = json! {
		{ keys[0].clone(): values[0].clone(), ("b" @ 3): {} @ 4, keys[1].clone(): values[1].clone() } @ 7
	};

	assert_eq!(
		value,
		Meta(
			Value::Object(vec![
				Entry::new(Meta("a".into(), 1), Meta(Value::Boolean(true), 2)),
				Entry::new(Meta("b".into(), 3), Meta(Value::Object(vec![]), 4)),
				Entry::new(Meta("c".into(), 5), Meta(Value::Boolean(false), 6))
			]),
			7
		)
	)
}

#[test]
fn macro_14() {
	let value: Meta<Value<()>, ()> = json! {
		{ "a": 0.1, "b": 1.1e10 }
	};

	assert_eq!(
		value,
		Meta(
			Value::Object(vec![
				Entry::new(Meta("a".into(), ()), Meta(Value::Number(0.1.try_into().unwrap()), ())),
				Entry::new(Meta("b".into(), ()), Meta(Value::Number(1.1e10.try_into().unwrap()), ()))
			]),
			()
		)
	)
}