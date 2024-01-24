use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Meta;

impl Parse for () {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
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
							Ok(Meta((), i))
						}
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
