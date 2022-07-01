use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::Loc;
use smallstr::SmallString;

fn is_control(c: char) -> bool {
	('\u{0000}'..='\u{001f}').contains(&c)
}

fn parse_hex4<F: Clone, E, C>(
	parser: &mut Parser<F, E, C>,
) -> Result<Loc<u32, F>, Loc<Error<E, F>, F>>
where
	C: Iterator<Item = Result<DecodedChar, E>>,
{
	match parser.next_char()? {
		Some(c) => match c.to_digit(16) {
			Some(h3) => match parser.next_char()? {
				Some(c) => match c.to_digit(16) {
					Some(h2) => match parser.next_char()? {
						Some(c) => match c.to_digit(16) {
							Some(h1) => match parser.next_char()? {
								Some(c) => match c.to_digit(16) {
									Some(h0) => Ok(Loc(
										h3 << 12 | h2 << 8 | h1 << 4 | h0,
										parser.position.current(),
									)),
									None => {
										Err(Loc(Error::unexpected(Some(c)), parser.position.last()))
									}
								},
								unexpected => {
									Err(Loc(Error::unexpected(unexpected), parser.position.last()))
								}
							},
							None => Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
						},
						unexpected => {
							Err(Loc(Error::unexpected(unexpected), parser.position.last()))
						}
					},
					None => Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
				},
				unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
			},
			None => Err(Loc(Error::unexpected(Some(c)), parser.position.last())),
		},
		unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
	}
}

impl<F: Clone, A: smallvec::Array<Item = u8>> Parse<F> for SmallString<A> {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		_context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match parser.next_char()? {
			Some('"') => {
				let mut result = Self::new();
				let span = parser.position.span;
				parser.position.span.clear();

				let mut high_surrogate: Option<Loc<u32, F>> = None;

				loop {
					let c = match parser.next_char()? {
						Some('"') => {
							if let Some(Loc(high, loc)) = high_surrogate {
								if parser.options.accept_truncated_surrogate_pair {
									result.push('\u{fffd}');
								} else {
									break Err(Loc(
										Error::MissingLowSurrogate(Loc(high as u16, loc)),
										parser.position.current(),
									));
								}
							}

							let mut pos = parser.position.current();
							pos.set_span(span.union(parser.position.span));
							break Ok(Loc(result, pos));
						}
						Some('\\') => match parser.next_char()? {
							Some(c @ ('"' | '\\' | '/')) => c,
							Some('b') => '\u{0008}',
							Some('t') => '\u{0009}',
							Some('n') => '\u{000a}',
							Some('f') => '\u{000c}',
							Some('r') => '\u{000d}',
							Some('u') => {
								let Loc(codepoint, codepoint_loc) = parse_hex4(parser)?;

								match high_surrogate.take() {
									Some(Loc(high, high_loc)) => {
										if (0xdc00..=0xdfff).contains(&codepoint) {
											let low = codepoint;
											let low_loc = codepoint_loc;
											let codepoint =
												((high - 0xd800) << 10 | (low - 0xdc00)) + 0x010000;
											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Loc(
															Error::InvalidUnicodeCodePoint(
																codepoint,
															),
															high_loc.with(low_loc.span()),
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
														break Err(Loc(
															Error::InvalidUnicodeCodePoint(
																codepoint,
															),
															codepoint_loc,
														));
													}
												}
											}
										} else {
											break Err(Loc(
												Error::InvalidLowSurrogate(
													Loc(high as u16, high_loc),
													codepoint,
												),
												codepoint_loc,
											));
										}
									}
									None => {
										if (0xd800..=0xdbff).contains(&codepoint) {
											high_surrogate = Some(Loc(codepoint, codepoint_loc));
											continue;
										} else {
											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Loc(
															Error::InvalidUnicodeCodePoint(
																codepoint,
															),
															codepoint_loc,
														));
													}
												}
											}
										}
									}
								}
							}
							unexpected => {
								break Err(Loc(
									Error::unexpected(unexpected),
									parser.position.last(),
								))
							}
						},
						Some(c) if !is_control(c) => c,
						unexpected => {
							break Err(Loc(Error::unexpected(unexpected), parser.position.last()))
						}
					};

					if let Some(Loc(high, loc)) = high_surrogate.take() {
						if parser.options.accept_truncated_surrogate_pair {
							result.push('\u{fffd}');
						} else {
							break Err(Loc(
								Error::MissingLowSurrogate(Loc(high as u16, loc)),
								parser.position.current(),
							));
						}
					}

					result.push(c);
					parser.position.span.clear();
				}
			}
			unexpected => Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
