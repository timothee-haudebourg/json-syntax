/// Constructs a `Loc<json_syntax::Value, (), ()>` from a JSON literal.
///
/// ```
/// # use json_syntax::{Value, json};
/// # use locspan::Meta;
/// let value: Meta<Value<()>, ()> = json!({
///     "code": 200,
///     "success": true,
///     "payload": {
///         "features": [
///             "json",
///             "syntax"
///         ]
///     }
/// });
/// ```
///
/// Variables or expressions can be interpolated into the JSON literal.
///
/// ```
/// # use json_syntax::{Value, Key, json};
/// # use locspan::Meta;
/// let code = 200;
/// let features = vec!["json", "syntax"];
///
/// let value: Meta<Value<()>, ()> = json!({
///     "code": Meta(Value::from(code), ()),
///     "success": Meta(Value::from(code == 200), ()),
///     "payload": {
///         Meta(Key::from(features[0]), ()): Meta(Value::from(features[1]), ())
///     }
/// });
/// ```
///
/// Trailing commas are allowed inside both arrays and objects.
///
/// ```
/// # use json_syntax::{Value, json};
/// # use locspan::Meta;
/// let value: Meta<Value<()>, ()> = json!([
///     "notice",
///     "the",
///     "trailing",
///     "comma -->",
/// ]);
/// ```
///
/// Metadata information can be added using the `@` symbol.
///
/// ```
/// # use json_syntax::{Value, json};
/// # use locspan::Meta;
/// let value: Meta<Value<u8>, u8> = json!({
///     "code": 200 @ 0,
///     "success": true @ 1,
///     "payload": {
///         "features": [
///             "json" @ 2,
///             "syntax" @ 3
///         ]
///     }
/// });
/// ```
#[macro_export(local_inner_macros)]
macro_rules! json {
	//////////////////////////////////////////////////////////////////////////
	// TT muncher for parsing the inside of an array [...]. Produces a vec![...]
	// of the elements.
	//
	// Must be invoked as: json!(@array [] $($tt)*)
	//////////////////////////////////////////////////////////////////////////

	// Done with trailing comma.
	(@array [$($elems:expr,)*]) => {
		json_vec![$($elems,)*]
	};

	// Done without trailing comma.
	(@array [$($elems:expr),*]) => {
		json_vec![$($elems),*]
	};

	// Next element is `null` with metadata.
	(@array [$($elems:expr,)*] null @ $meta:expr $(,$($rest:tt)*)?) => {
		json!(@array [$($elems,)* json!(null @ $meta)] $(,$($rest)*)?)
	};

	// Next element is `null`.
	(@array [$($elems:expr,)*] null $($rest:tt)*) => {
		json!(@array [$($elems,)* json!(null)] $($rest)*)
	};

	// Next element is `true` with metadata.
	(@array [$($elems:expr,)*] true @ $meta:expr $(,$($rest:tt)*)?) => {
		json!(@array [$($elems,)* json!(true @ $meta)] $(,$($rest)*)?)
	};

	// Next element is `true`.
	(@array [$($elems:expr,)*] true $($rest:tt)*) => {
		json!(@array [$($elems,)* json!(true)] $($rest)*)
	};

	// Next element is `false` with metadata.
	(@array [$($elems:expr,)*] false @ $meta:expr $(,$($rest:tt)*)?) => {
		json!(@array [$($elems,)* json!(false @ $meta)] $(,$($rest)*)?)
	};

	// Next element is `false`.
	(@array [$($elems:expr,)*] false $($rest:tt)*) => {
		json!(@array [$($elems,)* json!(false)] $($rest)*)
	};

	// Next element is a literal with metadata.
	(@array [$($elems:expr,)*] $lit:literal @ $meta:expr $(,$($rest:tt)*)?) => {
		json!(@array [$($elems,)* json!($lit @ $meta)] $(,$($rest)*)?)
	};

	// Next element is a literal.
	(@array [$($elems:expr,)*] $lit:literal $($rest:tt)*) => {
		json!(@array [$($elems,)* json!($lit)] $($rest)*)
	};

	// Next element is an array with metadata.
	(@array [$($elems:expr,)*] [$($array:tt)*] @ $meta:expr $(,$($rest:tt)*)?) => {
		json!(@array [$($elems,)* json!([$($array)*] @ $meta)] $(,$($rest)*)?)
	};

	// Next element is an array.
	(@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
		json!(@array [$($elems,)* json!([$($array)*])] $($rest)*)
	};

	// Next element is a map with metadata.
	(@array [$($elems:expr,)*] {$($map:tt)*} @ $meta:expr $(,$($rest:tt)*)?) => {
		json!(@array [$($elems,)* json!({$($map)*} @ $meta)] $(,$($rest)*)?)
	};

	// Next element is a map.
	(@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
		json!(@array [$($elems,)* json!({$($map)*})] $($rest)*)
	};

	// Next element is an expression followed by comma.
	(@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
		json!(@array [$($elems,)* json!($next),] $($rest)*)
	};

	// Last element is an expression with no trailing comma.
	(@array [$($elems:expr,)*] $last:expr) => {
		json!(@array [$($elems,)* json!($last)])
	};

	// Comma after the most recent element.
	(@array [$($elems:expr),*] , $($rest:tt)*) => {
		json!(@array [$($elems,)*] $($rest)*)
	};

	// Unexpected token after most recent element.
	(@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
		json_unexpected!($unexpected)
	};

	//////////////////////////////////////////////////////////////////////////
	// TT muncher for parsing the inside of an object {...}.
	//
	// Must be invoked as: json!(@object [] [] ($($tt)*))
	//
	// We require two copies of the input tokens so that we can match on one
	// copy and trigger errors on the other copy.
	//////////////////////////////////////////////////////////////////////////

	// Done with trailing comma.
	(@object [$($elems:expr,)*] () () ()) => {
		json_vec![$($elems,)*]
	};

	// Done without trailing comma.
	(@object [$($elems:expr),*] () () ()) => {
		json_vec![$($elems),*]
	};

	// Create an entry literal key with metadata.
	(@key (($key:literal @ $meta:expr))) => {
		::locspan::Meta($key.into(), $meta)
	};

	// Create an entry literal key.
	(@key ($key:literal)) => {
		::locspan::Meta($key.into(), ::core::default::Default::default())
	};

	// Create an entry key.
	(@key ($key:expr)) => {
		$key.into()
	};

	// Next value is `null` with metadata.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: null @ $meta:expr $(,$($rest:tt)*)?) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!(null @ $meta))] () ($(,$($rest)*)?) ($(,$($rest)*)?))
	};

	// Next value is `null`.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!(null))] () ($($rest)*) ($($rest)*))
	};

	// Next value is `true` with metadata.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: true @ $meta:expr $(,$($rest:tt)*)?) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!(true @ $meta))] () ($(,$($rest)*)?) ($(,$($rest)*)?))
	};

	// Next value is `true`.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!(true))] () ($($rest)*) ($($rest)*))
	};

	// Next value is `false` with metadata.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: false @ $meta:expr $(,$($rest:tt)*)?) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!(false @ $meta))] () ($(,$($rest)*)?) ($(,$($rest)*)?))
	};

	// Next value is `false`.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!(false))] () ($($rest)*) ($($rest)*))
	};

	// Next value is a literal with metadata.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: $lit:literal @ $meta:expr $(,$($rest:tt)*)?) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!($lit @ $meta))] () ($(,$($rest)*)?) ($(,$($rest)*)?))
	};

	// Next value is a literal.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: $lit:literal $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!($lit))] () ($($rest)*) ($($rest)*))
	};

	// Next value is a array with metadata.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: [$($array:tt)*] @ $meta:expr $(,$($rest:tt)*)?) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!([$($array)*] @ $meta))] () ($(,$($rest)*)?) ($(,$($rest)*)?))
	};

	// Next value is a array.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!([$($array)*]))] () ($($rest)*) ($($rest)*))
	};

	// Next value is a map with metadata.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: {$($map:tt)*} @ $meta:expr $(,$($rest:tt)*)?) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!({$($map)*} @ $meta))] () ($(,$($rest)*)?) ($(,$($rest)*)?))
	};

	// Next value is a map.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!({$($map)*}))] () ($($rest)*) ($($rest)*))
	};

	// Next value is an expression followed by comma.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: $next:expr, $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!($next)),] () ($($rest)*) ($($rest)*))
	};

	// Last value is an expression with no trailing comma.
	(@object [$($elems:expr,)*] ($($key:tt)+) (: $last:expr) $copy:tt) => {
		json!(@object [$($elems,)* $crate::Entry::new(json!(@key ($($key)+)), json!($last))] () () ())
	};

	// Comma after the most recent element.
	(@object [$($elems:expr),*] () (, $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)*] () ($($rest)*) ($($rest)*))
	};

	// Missing value for last entry. Trigger a reasonable error message.
	(@object [$($elems:expr,)*] ($($key:tt)+) (:) $copy:tt) => {
		// "unexpected end of macro invocation"
		json!();
	};

	// Missing colon and value for last entry. Trigger a reasonable error
	// message.
	(@object [$($elems:expr,)*] ($($key:tt)+) () $copy:tt) => {
		// "unexpected end of macro invocation"
		json!();
	};

	// Misplaced colon. Trigger a reasonable error message.
	(@object [$($elems:expr,)*] () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
		// Takes no arguments so "no rules expected the token `:`".
		json_unexpected!($colon);
	};

	// Found a comma inside a key. Trigger a reasonable error message.
	(@object [$($elems:expr,)*] ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
		// Takes no arguments so "no rules expected the token `,`".
		json_unexpected!($comma);
	};

	// Key is fully parenthesized. This avoids clippy double_parens false
	// positives because the parenthesization may be necessary here.
	(@object [$($elems:expr,)*] () (($key:expr) : $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)*] ($key) (: $($rest)*) (: $($rest)*));
	};

	// Refuse to absorb colon token into key expression.
	(@object [$($elems:expr,)*] ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
		json_expect_expr_comma!($($unexpected)+);
	};

	// Munch a token into the current key.
	(@object [$($elems:expr,)*] ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
		json!(@object [$($elems,)*] ($($key)* $tt) ($($rest)*) ($($rest)*));
	};

	//////////////////////////////////////////////////////////////////////////
	// The main implementation.
	//
	// Must be invoked as: json!($($json)+)
	//////////////////////////////////////////////////////////////////////////

	(null @ $meta:expr) => {
		::locspan::Meta($crate::Value::Null, $meta)
	};

	(null) => {
		json!(null @ ::core::default::Default::default())
	};

	(true @ $meta:expr) => {
		::locspan::Meta($crate::Value::Boolean(true), $meta)
	};

	(true) => {
		json!(true @ ::core::default::Default::default())
	};

	(false @ $meta:expr) => {
		::locspan::Meta($crate::Value::Boolean(false), $meta)
	};

	(false) => {
		json!(false @ ::core::default::Default::default())
	};

	($lit:literal @ $meta:expr) => {
		::locspan::Meta($lit.try_into().unwrap(), $meta)
	};

	($lit:literal) => {
		json!($lit @ ::core::default::Default::default())
	};

	([] @ $meta:expr) => {
		::locspan::Meta($crate::Value::Array(json_vec![]), $meta)
	};

	([]) => {
		json!([] @ ::core::default::Default::default())
	};

	([ $($tt:tt)+ ] @ $meta:expr) => {
		::locspan::Meta($crate::Value::Array(json!(@array [] $($tt)+)), $meta)
	};

	([ $($tt:tt)+ ]) => {
		json!([ $($tt)+ ] @ ::core::default::Default::default())
	};

	({} @ $meta:expr) => {
		::locspan::Meta($crate::Value::Object(json_vec![]), $meta)
	};

	({}) => {
		json!({} @ ::core::default::Default::default())
	};

	({ $($tt:tt)+ } @ $meta:expr) => {
		::locspan::Meta($crate::Value::Object(json!(@object [] () ($($tt)+) ($($tt)+))), $meta)
	};

	({ $($tt:tt)+ }) => {
		json!({ $($tt)+ } @ ::core::default::Default::default())
	};

	($other:expr) => {
		$other.into()
	};
}

// The json_internal macro above cannot invoke vec directly because it uses
// local_inner_macros. A vec invocation there would resolve to $crate::vec.
// Instead invoke vec here outside of local_inner_macros.
#[macro_export]
#[doc(hidden)]
macro_rules! json_vec {
    ($($content:tt)*) => {
        vec![$($content)*]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! json_unexpected {
	() => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! json_expect_expr_comma {
	($e:expr , $($tt:tt)*) => {};
}
