use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Loc;
use locspan_derive::*;

#[derive(
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
pub enum StartFragment {
	Empty,
	NonEmpty,
}

impl<F: Clone> Parse<F> for StartFragment {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some('[') => {
				parser.skip_whitespaces()?;

				match parser.peek_char()? {
					Some(']') => {
						parser.next_char()?;
						Ok(Loc(StartFragment::Empty, parser.position.current()))
					}
					_ => {
						// wait for value.
						Ok(Loc(StartFragment::NonEmpty, parser.position.current()))
					}
				}
			}
			unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}

#[derive(
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Debug,
	StrippedPartialEq,
	StrippedEq,
	StrippedPartialOrd,
	StrippedOrd,
	StrippedHash,
)]
pub enum ContinueFragment {
	Item,
	End,
}

impl<F: Clone> Parse<F> for ContinueFragment {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some(',') => Ok(Loc(Self::Item, parser.position.current())),
			Some(']') => Ok(Loc(Self::End, parser.position.current())),
			unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
