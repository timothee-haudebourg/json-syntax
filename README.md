# Strict JSON parsing and mapping library

[![CI](https://github.com/timothee-haudebourg/json-syntax/workflows/CI/badge.svg)](https://github.com/timothee-haudebourg/json-syntax/actions)
[![Crate informations](https://img.shields.io/crates/v/json-syntax.svg?style=flat-square)](https://crates.io/crates/json-syntax)
[![License](https://img.shields.io/crates/l/json-syntax.svg?style=flat-square)](https://github.com/timothee-haudebourg/json-syntax#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/json-syntax)

This library provides a strict JSON parser as defined by
[RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) and
[ECMA-404](https://www.ecma-international.org/publications-and-standards/standards/ecma-404/).
It is built on the [`locspan`](https://crates.io/crates/locspan) library
so as to keep track of the position of each JSON value in the parsed
document.

## Features

- Strict implementation of [RFC 8259](https://datatracker.ietf.org/doc/html/rfc8259) and
  [ECMA-404](https://www.ecma-international.org/publications-and-standards/standards/ecma-404/).
- No stack overflow, your memory is the limit.
- Numbers are stored in lexical form thanks to the [`json-number`](https://crates.io/crates/json-number) crate,
  their precision is not limited.
- Duplicate values are preserved. A JSON object is just a list of entries,
  in the order of definition.
- Strings are stored on the stack whenever possible, thanks to the [`smallstr`](https://crates.io/crates/smallstr) crate.
- The parser is configurable to accept documents that do not strictly
  adhere to the standard.
- Thoroughly tested.

## Usage

```rust
use std::fs;
use json_syntax::{Value, Parse};
use decoded_char::DecodedChars;
use locspan::Loc;

fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
  Ok(t)
}

let filename = "tests/inputs/y_structure_500_nested_arrays.json";
let input = fs::read_to_string(filename).unwrap();
let Loc(value, value_location) = Value::parse(filename, input.decoded_chars().map(infallible)).expect("parse error");

// ...
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
