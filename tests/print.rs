use json_syntax::{json, Print};

#[test]
fn print_01() {
	let value = json! { null };
	assert_eq!(value.pretty_print().to_string(), "null")
}

#[test]
fn print_02() {
	let value = json! { true };
	assert_eq!(value.pretty_print().to_string(), "true")
}

#[test]
fn print_03() {
	let value = json! { false };
	assert_eq!(value.pretty_print().to_string(), "false")
}

#[test]
fn print_04() {
	let value = json! { "foo" };
	assert_eq!(value.pretty_print().to_string(), "\"foo\"")
}

#[test]
fn print_05() {
	let value = json! { 1 };
	assert_eq!(value.pretty_print().to_string(), "1")
}

#[test]
fn print_06() {
	let value = json! { [] };
	assert_eq!(value.pretty_print().to_string(), "[]")
}

#[test]
fn print_07() {
	let value = json! { [ null ] };
	assert_eq!(value.pretty_print().to_string(), "[ null ]")
}

#[test]
fn print_08() {
	let value = json! { [ "azertyuiop" ] };
	assert_eq!(value.pretty_print().to_string(), "[ \"azertyuiop\" ]")
}

#[test]
fn print_09() {
	let value = json! { [ "azertyuiopq" ] };
	assert_eq!(value.pretty_print().to_string(), "[\n  \"azertyuiopq\"\n]")
}

#[test]
fn print_10() {
	let value = json! { [ true, false ] };
	assert_eq!(value.pretty_print().to_string(), "[\n  true,\n  false\n]")
}

#[test]
fn print_11() {
	let value = json! { { "a": null } };
	assert_eq!(value.pretty_print().to_string(), "{ \"a\": null }")
}

#[test]
fn print_12() {
	let value = json! { { "a": null, "b": 12 } };
	assert_eq!(
		value.pretty_print().to_string(),
		"{\n  \"a\": null,\n  \"b\": 12\n}"
	)
}

#[test]
fn print_13() {
	let value = json! { { "a": [ null ], "b": [ 13 ] } };
	assert_eq!(
		value.pretty_print().to_string(),
		"{\n  \"a\": [ null ],\n  \"b\": [ 13 ]\n}"
	)
}

#[test]
fn print_14() {
	let value = json! { { "a": [ null, [] ], "b": [ 14 ] } };
	assert_eq!(
		value.pretty_print().to_string(),
		"{\n  \"a\": [\n    null,\n    []\n  ],\n  \"b\": [ 14 ]\n}"
	)
}
