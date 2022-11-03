use super::{Context, Error, Parse, Parser};
use crate::object::Key;
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use locspan_derive::*;

#[derive(
	Clone,
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
#[locspan(ignore(M))]
pub enum StartFragment<M> {
	Empty,
	NonEmpty(#[locspan(deref_stripped)] Meta<Key, M>),
}

impl<M> Parse<M> for StartFragment<M> {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		_context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match parser.next_char()? {
			Some('{') => {
				parser.skip_whitespaces()?;

				match parser.peek_char()? {
					Some('}') => {
						parser.next_char()?;
						Ok(Meta(StartFragment::Empty, parser.position.current_span()))
					}
					_ => {
						let span = parser.position.span;
						parser.position.span.clear();
						let key = Key::parse_in(parser, Context::ObjectKey)?;
						let span = span.union(parser.position.span);
						parser.skip_whitespaces()?;
						match parser.next_char()? {
							Some(':') => Ok(Meta(Self::NonEmpty(key), span)),
							unexpected => {
								Err(Meta(Error::unexpected(unexpected), parser.position.last()))
							}
						}
					}
				}
			}
			unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}

#[derive(
	Clone,
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
#[locspan(ignore(M))]
pub enum ContinueFragment<M> {
	End,
	Entry(#[locspan(deref_stripped)] Meta<Key, M>),
}

impl<M> Parse<M> for ContinueFragment<M> {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		_context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match parser.next_char()? {
			Some(',') => {
				let span = parser.position.span;
				parser.skip_whitespaces()?;
				let key = Key::parse_in(parser, Context::ObjectKey)?;
				let span = span.union(parser.position.span);
				parser.skip_whitespaces()?;
				match parser.next_char()? {
					Some(':') => Ok(Meta(Self::Entry(key), span)),
					unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
				}
			}
			Some('}') => Ok(Meta(Self::End, parser.position.current_span())),
			unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
