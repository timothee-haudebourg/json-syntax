use super::{JsonParse, JsonParseError, JsonParsingContext, Parser};
use decoded_char::DecodedChar;

impl JsonParse for bool {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: JsonParsingContext,
	) -> Result<(Self, usize), JsonParseError<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let i = parser.begin_fragment();
		match parser.next_char()? {
			(_, Some('t')) => match parser.next_char()? {
				(_, Some('r')) => match parser.next_char()? {
					(_, Some('u')) => match parser.next_char()? {
						(_, Some('e')) => {
							parser.end_fragment(i);
							Ok((true, i))
						}
						(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
					},
					(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
				},
				(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
			},
			(_, Some('f')) => match parser.next_char()? {
				(_, Some('a')) => match parser.next_char()? {
					(_, Some('l')) => match parser.next_char()? {
						(_, Some('s')) => match parser.next_char()? {
							(_, Some('e')) => {
								parser.end_fragment(i);
								Ok((false, i))
							}
							(p, unexpected) => Err(JsonParseError::unexpected(p, unexpected)),
						},
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
