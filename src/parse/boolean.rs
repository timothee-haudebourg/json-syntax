use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Meta;

impl Parse for bool {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
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
							Ok(Meta(true, i))
						}
						(p, unexpected) => Err(Error::unexpected(p, unexpected)),
					},
					(p, unexpected) => Err(Error::unexpected(p, unexpected)),
				},
				(p, unexpected) => Err(Error::unexpected(p, unexpected)),
			},
			(_, Some('f')) => match parser.next_char()? {
				(_, Some('a')) => match parser.next_char()? {
					(_, Some('l')) => match parser.next_char()? {
						(_, Some('s')) => match parser.next_char()? {
							(_, Some('e')) => {
								parser.end_fragment(i);
								Ok(Meta(false, i))
							}
							(p, unexpected) => Err(Error::unexpected(p, unexpected)),
						},
						(p, unexpected) => Err(Error::unexpected(p, unexpected)),
					},
					(p, unexpected) => Err(Error::unexpected(p, unexpected)),
				},
				(p, unexpected) => Err(Error::unexpected(p, unexpected)),
			},
			(p, unexpected) => Err(Error::unexpected(p, unexpected)),
		}
	}
}
