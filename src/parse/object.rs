use super::{Context, Error, Parse, Parser};
use crate::Key;
use decoded_char::DecodedChar;
use locspan::Loc;

pub enum StartFragment<F> {
	Empty,
	NonEmpty(Loc<Key, F>),
}

impl<F: Clone> Parse<F> for StartFragment<F> {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some('{') => {
				parser.skip_whitespaces()?;

				match parser.peek_char()? {
					Some('}') => {
						parser.next_char()?;
						Ok(Loc(StartFragment::Empty, parser.position.current()))
					}
					_ => {
						let span = parser.position.span;
						parser.position.span.clear();
						let key = Key::parse_in(parser, Context::ObjectKey)?;
						parser.skip_whitespaces()?;
						parser.position.span = span.union(key.span());
						match parser.next_char()? {
							Some(':') => Ok(Loc(Self::NonEmpty(key), parser.position.current())),
							unexpected => {
								Err(Loc(Error::unexpected(unexpected), parser.position.last()))
							}
						}
					}
				}
			}
			unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}

pub enum ContinueFragment<F> {
	End,
	Entry(Loc<Key, F>),
}

impl<F: Clone> Parse<F> for ContinueFragment<F> {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some(',') => {
				let span = parser.position.span;
				parser.skip_whitespaces()?;
				let key = Key::parse_in(parser, Context::ObjectKey)?;
				parser.skip_whitespaces()?;
				parser.position.span = span.union(key.span());
				match parser.next_char()? {
					Some(':') => Ok(Loc(Self::Entry(key), parser.position.current())),
					unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
				}
			}
			Some('}') => Ok(Loc(Self::End, parser.position.current())),
			unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
