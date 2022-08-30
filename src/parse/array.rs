use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
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

impl<M> Parse<M> for StartFragment {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		_context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match parser.next_char()? {
			Some('[') => {
				parser.skip_whitespaces()?;

				match parser.peek_char()? {
					Some(']') => {
						parser.next_char()?;
						Ok(Meta(StartFragment::Empty, parser.position.current_span()))
					}
					_ => {
						// wait for value.
						Ok(Meta(
							StartFragment::NonEmpty,
							parser.position.current_span(),
						))
					}
				}
			}
			unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
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

impl<M> Parse<M> for ContinueFragment {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		_context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match parser.next_char()? {
			Some(',') => Ok(Meta(Self::Item, parser.position.current_span())),
			Some(']') => Ok(Meta(Self::End, parser.position.current_span())),
			unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
