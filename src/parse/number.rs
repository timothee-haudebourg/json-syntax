use super::{Context, Error, Parse, Parser};
use crate::{NumberBuf, SMALL_STRING_CAPACITY};
use decoded_char::DecodedChar;
use locspan::Meta;
use smallvec::SmallVec;

impl Parse for NumberBuf {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let i = parser.begin_fragment();
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
					_ => return Err(Error::unexpected(parser.position, Some(c))),
				},
				State::FirstDigit => match c {
					'0' => state = State::Zero,
					'1'..='9' => state = State::NonZero,
					_ => return Err(Error::unexpected(parser.position, Some(c))),
				},
				State::Zero => match c {
					'.' => state = State::FractionalFirst,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Error::unexpected(parser.position, Some(c)));
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
							return Err(Error::unexpected(parser.position, Some(c)));
						}
					}
				},
				State::FractionalFirst => match c {
					'0'..='9' => state = State::FractionalRest,
					_ => return Err(Error::unexpected(parser.position, Some(c))),
				},
				State::FractionalRest => match c {
					'0'..='9' => state = State::FractionalRest,
					'e' | 'E' => state = State::ExponentSign,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Error::unexpected(parser.position, Some(c)));
						}
					}
				},
				State::ExponentSign => match c {
					'+' | '-' => state = State::ExponentFirst,
					'0'..='9' => state = State::ExponentRest,
					_ => return Err(Error::unexpected(parser.position, Some(c))),
				},
				State::ExponentFirst => match c {
					'0'..='9' => state = State::ExponentRest,
					_ => return Err(Error::unexpected(parser.position, Some(c))),
				},
				State::ExponentRest => match c {
					'0'..='9' => state = State::ExponentRest,
					_ => {
						if context.follows(c) {
							break;
						} else {
							return Err(Error::unexpected(parser.position, Some(c)));
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
			parser.end_fragment(i);
			Ok(Meta(unsafe { NumberBuf::new_unchecked(buffer) }, i))
		} else {
			Err(Error::unexpected(parser.position, None))
		}
	}
}
