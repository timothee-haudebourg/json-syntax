use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::{Meta, Span};

impl<M> Parse<M> for () {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		_context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match parser.next_char()? {
			Some('n') => match parser.next_char()? {
				Some('u') => match parser.next_char()? {
					Some('l') => match parser.next_char()? {
						Some('l') => Ok(Meta((), parser.position.current_span())),
						unexpected => {
							Err(Meta(Error::unexpected(unexpected), parser.position.last()))
						}
					},
					unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
				},
				unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
			},
			unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
