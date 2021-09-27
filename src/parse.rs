use std::io;
use source_span::{Loc, Position, Span, Metrics};
use crate::{Value, SmallString};

pub enum Warning {
	DuplicateEntry(SmallString, Span)
}

pub enum Error {
	IO(io::Error),
	UnexpectedChar(char)
}

impl From<io::Error> for Error {
	fn from(e: io::Error) -> Self {
		Self::IO(e)
	}
}

/// Parse a JSON value.
pub fn parse<C, M: Metrics>(
	warnings: &mut Vec<Loc<Warning>>,
	chars: &mut C,
	mut pos: Position,
	metrics: &M
) -> Result<Option<Value>, Loc<Error>> where C: Iterator<Item=io::Result<char>> {
	loop {
		match chars.next() {
			None => break Ok(None),
			Some(Ok(c)) => match c {
				c if c.is_whitespace() => (),
				'[' => {
					panic!("TODO array")
				}
				'{' => {
					panic!("TODO object")
				}
				c if c.is_digit(10) => {
					panic!("TODO number")
				}
				'+' => {
					panic!("TODO positive number")
				}
				'-' => {
					panic!("TODO negative number")
				}
				'\"' => {
					panic!("TODO string")
				}
				'n' => {
					panic!("TODO null")
				}
				't' => {
					panic!("TODO true")
				}
				'f' => {
					panic!("TODO false")
				}
				c => {
					let mut span: Span = pos.into();
					span.push(c, metrics);
					break Err(Loc::new(Error::UnexpectedChar(c), span))
				}
			},
			Some(Err(e)) => break Err(Loc::new(Error::IO(e), pos.into()))
		}
	}
}