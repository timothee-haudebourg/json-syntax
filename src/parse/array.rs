use super::{JsonParse, JsonParseError, JsonParsingContext, Parser};
use decoded_char::DecodedChar;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum StartFragment {
	Empty,
	NonEmpty,
}

impl JsonParse for StartFragment {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: JsonParsingContext,
	) -> Result<(Self, usize), JsonParseError<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let i = parser.begin_fragment();
		match parser.next_char()? {
			(_, Some('[')) => {
				parser.skip_whitespaces()?;

				match parser.peek_char()? {
					Some(']') => {
						parser.next_char()?;
						parser.end_fragment(i);
						Ok((StartFragment::Empty, i))
					}
					_ => {
						// wait for value.
						Ok((StartFragment::NonEmpty, i))
					}
				}
			}
			(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ContinueFragment {
	Item,
	End,
}

impl ContinueFragment {
	pub fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		array: usize,
	) -> Result<Self, JsonParseError<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		parser.skip_whitespaces()?;
		match parser.next_char()? {
			(_, Some(',')) => Ok(Self::Item),
			(_, Some(']')) => {
				parser.end_fragment(array);
				Ok(Self::End)
			}
			(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
		}
	}
}
