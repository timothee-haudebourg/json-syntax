use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Loc;

impl<F: Clone> Parse<F> for bool {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some('t') => match parser.next_char()? {
				Some('r') => match parser.next_char()? {
					Some('u') => match parser.next_char()? {
						Some('e') => Ok(Loc(true, parser.position.current())),
						unexpected => {
							Err(Loc(Error::unexpected(unexpected), parser.position.last()))
						}
					},
					unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
				},
				unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
			},
			Some('f') => match parser.next_char()? {
				Some('a') => match parser.next_char()? {
					Some('l') => match parser.next_char()? {
						Some('s') => match parser.next_char()? {
							Some('e') => Ok(Loc(false, parser.position.current())),
							unexpected => {
								Err(Loc(Error::unexpected(unexpected), parser.position.last()))
							}
						},
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
