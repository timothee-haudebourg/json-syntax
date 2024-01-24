use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Meta;
use locspan_derive::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum StartFragment {
	Empty,
	NonEmpty,
}

impl Parse for StartFragment {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let i = parser.begin_fragment();
		match parser.next_char()? {
			(_, Some('[')) => {
				parser.skip_whitespaces()?;

				match parser.peek_char()? {
					Some(']') => {
						parser.next_char()?;
						parser.end_fragment(i);
						Ok(Meta(StartFragment::Empty, i))
					}
					_ => {
						// wait for value.
						Ok(Meta(StartFragment::NonEmpty, i))
					}
				}
			}
			(p, unexpected) => Err(Error::unexpected(p, unexpected)),
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

impl ContinueFragment {
	pub fn parse_in<C, E>(parser: &mut Parser<C, E>, array: usize) -> Result<Self, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		parser.skip_whitespaces()?;
		match parser.next_char()? {
			(_, Some(',')) => Ok(Self::Item),
			(_, Some(']')) => {
				parser.end_fragment(array);
				Ok(Self::End)
			}
			(p, unexpected) => Err(Error::unexpected(p, unexpected)),
		}
	}
}
