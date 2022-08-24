use super::{Context, Error, Parse, Parser};
use crate::{NumberBuf, SMALL_STRING_CAPACITY};
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use smallvec::SmallVec;

impl<M> Parse<M> for NumberBuf {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<E, M>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		let mut buffer: SmallVec<[u8; SMALL_STRING_CAPACITY]> = SmallVec::new();

		enum State {
			Init,
			FirstDigit,
			Zero,
			NonZero,
			FractionalFirst,
			FractionalRest,
			ExponentSign,
			ExponentFirst,
			ExponentRest,
		}

		let mut state = State::Init;

		while let Some(c) = parser.peek_char()? {
			match state {
				State::Init => match c {
					'-' => state = State::FirstDigit,
					'0' => state = State::Zero,
					'1'..='9' => state = State::NonZero,
					_ => return Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::FirstDigit => match c {
					'0' => state = State::Zero,
					'1'..='9' => state = State::NonZero,
					_ => return Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::Zero => match c {
					'.' => state = State::FractionalFirst,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Meta(Error::unexpected(Some(c)), parser.position.last()));
						}
					}
				},
				State::NonZero => match c {
					'0'..='9' => state = State::NonZero,
					'.' => state = State::FractionalFirst,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Meta(Error::unexpected(Some(c)), parser.position.last()));
						}
					}
				},
				State::FractionalFirst => match c {
					'0'..='9' => state = State::FractionalRest,
					_ => return Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::FractionalRest => match c {
					'0'..='9' => state = State::FractionalRest,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Meta(Error::unexpected(Some(c)), parser.position.last()));
						}
					}
				},
				State::ExponentSign => match c {
					'+' | '-' => state = State::ExponentFirst,
					'0'..='9' => state = State::ExponentRest,
					_ => return Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::ExponentFirst => match c {
					'0'..='9' => state = State::ExponentRest,
					_ => return Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::ExponentRest => match c {
					'0'..='9' => state = State::ExponentRest,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Meta(Error::unexpected(Some(c)), parser.position.last()));
						}
					}
				},
			}

			// u8 conversion is safe since a number is composed of ASCII chars.
			buffer.push(c as u8);
			parser.next_char()?;
		}

		if matches!(
			state,
			State::Zero | State::NonZero | State::FractionalRest | State::ExponentRest
		) {
			Ok(Meta(
				unsafe { NumberBuf::new_unchecked(buffer) },
				parser.position.current_span(),
			))
		} else {
			Err(Meta(Error::unexpected(None), parser.position.last()))
		}
	}
}
