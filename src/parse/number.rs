use super::{Context, Error, Parse, Parser};
use crate::{NumberBuf, SMALL_STRING_CAPACITY};
use decoded_char::DecodedChar;
use locspan::Loc;
use smallvec::SmallVec;

impl<F: Clone> Parse<F> for NumberBuf {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
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
					_ => return Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::FirstDigit => match c {
					'0' => state = State::Zero,
					'1'..='9' => state = State::NonZero,
					_ => return Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::Zero => match c {
					'.' => state = State::FractionalFirst,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Loc(Error::unexpected(Some(c)), parser.position.last()));
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
							return Err(Loc(Error::unexpected(Some(c)), parser.position.last()));
						}
					}
				},
				State::FractionalFirst => match c {
					'0'..='9' => state = State::FractionalRest,
					_ => return Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::FractionalRest => match c {
					'0'..='9' => state = State::FractionalRest,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Loc(Error::unexpected(Some(c)), parser.position.last()));
						}
					}
				},
				State::ExponentSign => match c {
					'+' | '-' => state = State::ExponentFirst,
					'0'..='9' => state = State::ExponentRest,
					_ => return Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::ExponentFirst => match c {
					'0'..='9' => state = State::ExponentRest,
					_ => return Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
				},
				State::ExponentRest => match c {
					'0'..='9' => state = State::ExponentRest,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Loc(Error::unexpected(Some(c)), parser.position.last()));
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
			Ok(Loc(
				unsafe { NumberBuf::new_unchecked(buffer) },
				parser.position.current(),
			))
		} else {
			Err(Loc(Error::unexpected(None), parser.position.last()))
		}
	}
}
