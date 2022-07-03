use json_syntax::{json, Print, Value};
use locspan::Meta;

#[test]
fn print_01() {
	let value: Meta<Value<()>, ()> = json! { null };
	assert_eq!(value.pretty_print().to_string(), "null")
}

#[test]
fn print_02() {
	let value: Meta<Value<()>, ()> = json! { true };
	assert_eq!(value.pretty_print().to_string(), "true")
}

#[test]
fn print_03() {
	let value: Meta<Value<()>, ()> = json! { false };
	assert_eq!(value.pretty_print().to_string(), "false")
}

#[test]
fn print_04() {
	let value: Meta<Value<()>, ()> = json! { "foo" };
	assert_eq!(value.pretty_print().to_string(), "\"foo\"")
}

#[test]
fn print_05() {
	let value: Meta<Value<()>, ()> = json! { 1 };
	assert_eq!(value.pretty_print().to_string(), "1")
}

#[test]
fn print_06() {
	let value: Meta<Value<()>, ()> = json! { [] };
	assert_eq!(value.pretty_print().to_string(), "[]")
}

#[test]
fn print_07() {
	let value: Meta<Value<()>, ()> = json! { [ null ] };
	assert_eq!(value.pretty_print().to_string(), "[ null ]")
}

#[test]
fn print_08() {
	let value: Meta<Value<()>, ()> = json! { [ "azertyuiop" ] };
	assert_eq!(value.pretty_print().to_string(), "[ \"azertyuiop\" ]")
}

#[test]
fn print_09() {
	let value: Meta<Value<()>, ()> = json! { [ "azertyuiopq" ] };
	assert_eq!(value.pretty_print().to_string(), "[\n  \"azertyuiopq\"\n]")
}

#[test]
fn print_10() {
	let value: Meta<Value<()>, ()> = json! { [ true, false ] };
	assert_eq!(value.pretty_print().to_string(), "[\n  true,\n  false\n]")
}

#[test]
fn print_11() {
	let value: Meta<Value<()>, ()> = json! { { "a": null } };
	assert_eq!(value.pretty_print().to_string(), "{ \"a\": null }")
}

#[test]
fn print_12() {
	let value: Meta<Value<()>, ()> = json! { { "a": null, "b": 12 } };
	assert_eq!(
		value.pretty_print().to_string(),
		"{\n  \"a\": null,\n  \"b\": 12\n}"
	)
}

#[test]
fn print_13() {
	let value: Meta<Value<()>, ()> = json! { { "a": [ null ], "b": [ 13 ] } };
	assert_eq!(
		value.pretty_print().to_string(),
		"{\n  \"a\": [ null ],\n  \"b\": [ 13 ]\n}"
	)
}

#[test]
fn print_14() {
	let value: Meta<Value<()>, ()> = json! { { "a": [ null, [] ], "b": [ 14 ] } };
	assert_eq!(
		value.pretty_print().to_string(),
		"{\n  \"a\": [\n    null,\n    []\n  ],\n  \"b\": [ 14 ]\n}"
	)
}
