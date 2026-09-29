use super::{JsonParse, JsonParseError, JsonParsingContext, Parser};
use decoded_char::DecodedChar;

impl JsonParse for () {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: JsonParsingContext,
	) -> Result<(Self, usize), JsonParseError<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let i = parser.begin_fragment();
		match parser.next_char()? {
			(_, Some('n')) => match parser.next_char()? {
				(_, Some('u')) => match parser.next_char()? {
					(_, Some('l')) => match parser.next_char()? {
						(_, Some('l')) => {
							parser.end_fragment(i);
							Ok(((), i))
						}
						(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
					},
					(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
				},
				(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
			},
			(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
		}
	}
}
