use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use smallstr::SmallString;

fn is_control(c: char) -> bool {
	('\u{0000}'..='\u{001f}').contains(&c)
}

fn parse_hex4<C, E>(parser: &mut Parser<C, E>) -> Result<u32, Error<E>>
where
	C: Iterator<Item = Result<DecodedChar, E>>,
{
	match parser.next_char()? {
		(p, Some(c)) => match c.to_digit(16) {
			Some(h3) => match parser.next_char()? {
				(p, Some(c)) => match c.to_digit(16) {
					Some(h2) => match parser.next_char()? {
						(p, Some(c)) => match c.to_digit(16) {
							Some(h1) => match parser.next_char()? {
								(p, Some(c)) => match c.to_digit(16) {
									Some(h0) => Ok(h3 << 12 | h2 << 8 | h1 << 4 | h0),
									None => Err(Error::unexpected(p, Some(c))),
								},
								(p, unexpected) => Err(Error::unexpected(p, unexpected)),
							},
							None => Err(Error::unexpected(p, Some(c))),
						},
						(p, unexpected) => Err(Error::unexpected(p, unexpected)),
					},
					None => Err(Error::unexpected(p, Some(c))),
				},
				(p, unexpected) => Err(Error::unexpected(p, unexpected)),
			},
			None => Err(Error::unexpected(p, Some(c))),
		},
		(p, unexpected) => Err(Error::unexpected(p, unexpected)),
	}
}

impl<A: smallvec::Array<Item = u8>> Parse for SmallString<A> {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		_context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let i = parser.begin_fragment();
		match parser.next_char()? {
			(_, Some('"')) => {
				let mut result = Self::new();
				let mut high_surrogate: Option<(usize, u32)> = None;
				loop {
					let c = match parser.next_char()? {
						(p, Some('"')) => {
							if let Some((p_high, high)) = high_surrogate {
								if parser.options.accept_truncated_surrogate_pair {
									result.push('\u{fffd}');
								} else {
									break Err(Error::MissingLowSurrogate(
										Span::new(p_high, p),
										high as u16,
									));
								}
							}

							parser.end_fragment(i);
							break Ok(Meta(result, i));
						}
						(_, Some('\\')) => match parser.next_char()? {
							(_, Some(c @ ('"' | '\\' | '/'))) => c,
							(_, Some('b')) => '\u{0008}',
							(_, Some('t')) => '\u{0009}',
							(_, Some('n')) => '\u{000a}',
							(_, Some('f')) => '\u{000c}',
							(_, Some('r')) => '\u{000d}',
							(p, Some('u')) => {
								let codepoint = parse_hex4(parser)?;

								match high_surrogate.take() {
									Some((p_high, high)) => {
										if (0xdc00..=0xdfff).contains(&codepoint) {
											let low = codepoint;
											let codepoint =
												((high - 0xd800) << 10 | (low - 0xdc00)) + 0x010000;
											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Error::InvalidUnicodeCodePoint(
															Span::new(p_high, parser.position),
															codepoint,
														));
													}
												}
											}
										} else if parser.options.accept_truncated_surrogate_pair {
											result.push('\u{fffd}');

											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Error::InvalidUnicodeCodePoint(
															Span::new(p, parser.position),
															codepoint,
														));
													}
												}
											}
										} else {
											break Err(Error::InvalidLowSurrogate(
												Span::new(p, parser.position),
												high as u16,
												codepoint,
											));
										}
									}
									None => {
										if (0xd800..=0xdbff).contains(&codepoint) {
											high_surrogate = Some((p, codepoint));
											continue;
										} else {
											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Error::InvalidUnicodeCodePoint(
															Span::new(p, parser.position),
															codepoint,
														));
													}
												}
											}
										}
									}
								}
							}
							(p, unexpected) => break Err(Error::unexpected(p, unexpected)),
						},
						(_, Some(c)) if !is_control(c) => c,
						(p, unexpected) => break Err(Error::unexpected(p, unexpected)),
					};

					if let Some((p_high, high)) = high_surrogate.take() {
						if parser.options.accept_truncated_surrogate_pair {
							result.push('\u{fffd}');
						} else {
							break Err(Error::MissingLowSurrogate(
								Span::new(p_high, parser.position),
								high as u16,
							));
						}
					}

					result.push(c);
				}
			}
			(p, unexpected) => Err(Error::unexpected(p, unexpected)),
		}
	}
}
