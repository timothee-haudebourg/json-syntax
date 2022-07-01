use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Loc;

impl<F: Clone> Parse<F> for () {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some('n') => match parser.next_char()? {
				Some('u') => match parser.next_char()? {
					Some('l') => match parser.next_char()? {
						Some('l') => Ok(Loc((), parser.position.current())),
						unexpected => {
							Err(Loc(Error::unexpected(unexpected), parser.position.last()))
						}
					},
					unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
				},
				unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
			},
			unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
