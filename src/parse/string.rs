use super::{Context, Error, Parse, Parser};
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use smallstr::SmallString;

fn is_control(c: char) -> bool {
	('\u{0000}'..='\u{001f}').contains(&c)
}

fn parse_hex4<C, F, E, M>(
	parser: &mut Parser<C, F, E>,
) -> Result<Meta<u32, Span>, Meta<Error<E, M>, M>>
where
	C: Iterator<Item = Result<DecodedChar, E>>,
	F: FnMut(Span) -> M,
{
	match parser.next_char()? {
		Some(c) => match c.to_digit(16) {
			Some(h3) => match parser.next_char()? {
				Some(c) => match c.to_digit(16) {
					Some(h2) => match parser.next_char()? {
						Some(c) => match c.to_digit(16) {
							Some(h1) => match parser.next_char()? {
								Some(c) => match c.to_digit(16) {
									Some(h0) => Ok(Meta(
										h3 << 12 | h2 << 8 | h1 << 4 | h0,
										parser.position.current_span(),
									)),
									None => Err(Meta(
										Error::unexpected(Some(c)),
										parser.position.last(),
									)),
								},
								unexpected => {
									Err(Meta(Error::unexpected(unexpected), parser.position.last()))
								}
							},
							None => Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
						},
						unexpected => {
							Err(Meta(Error::unexpected(unexpected), parser.position.last()))
						}
					},
					None => Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
				},
				unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
			},
			None => Err(Meta(Error::unexpected(Some(c)), parser.position.last())),
		},
		unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
	}
}

impl<M, A: smallvec::Array<Item = u8>> Parse<M> for SmallString<A> {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		_context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<E, M>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match parser.next_char()? {
			Some('"') => {
				let mut result = Self::new();
				let span = parser.position.span;
				parser.position.span.clear();

				let mut high_surrogate: Option<Meta<u32, Span>> = None;

				loop {
					let c = match parser.next_char()? {
						Some('"') => {
							if let Some(Meta(high, span)) = high_surrogate {
								if parser.options.accept_truncated_surrogate_pair {
									result.push('\u{fffd}');
								} else {
									break Err(Meta(
										Error::MissingLowSurrogate(Meta(
											high as u16,
											parser.position.metadata_at(span),
										)),
										parser.position.current(),
									));
								}
							}

							parser.position.span = span.union(parser.position.span);
							break Ok(Meta(result, parser.position.current_span()));
						}
						Some('\\') => match parser.next_char()? {
							Some(c @ ('"' | '\\' | '/')) => c,
							Some('b') => '\u{0008}',
							Some('t') => '\u{0009}',
							Some('n') => '\u{000a}',
							Some('f') => '\u{000c}',
							Some('r') => '\u{000d}',
							Some('u') => {
								let Meta(codepoint, codepoint_span) = parse_hex4(parser)?;

								match high_surrogate.take() {
									Some(Meta(high, high_span)) => {
										if (0xdc00..=0xdfff).contains(&codepoint) {
											let low = codepoint;
											let low_span = codepoint_span;
											let codepoint =
												((high - 0xd800) << 10 | (low - 0xdc00)) + 0x010000;
											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Meta(
															Error::InvalidUnicodeCodePoint(
																codepoint,
															),
															parser.position.metadata_at(
																low_span.union(high_span),
															),
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
														break Err(Meta(
															Error::InvalidUnicodeCodePoint(
																codepoint,
															),
															parser
																.position
																.metadata_at(codepoint_span),
														));
													}
												}
											}
										} else {
											break Err(Meta(
												Error::InvalidLowSurrogate(
													Meta(
														high as u16,
														parser.position.metadata_at(high_span),
													),
													codepoint,
												),
												parser.position.metadata_at(codepoint_span),
											));
										}
									}
									None => {
										if (0xd800..=0xdbff).contains(&codepoint) {
											high_surrogate = Some(Meta(codepoint, codepoint_span));
											continue;
										} else {
											match char::from_u32(codepoint) {
												Some(c) => c,
												None => {
													if parser.options.accept_invalid_codepoints {
														'\u{fffd}'
													} else {
														break Err(Meta(
															Error::InvalidUnicodeCodePoint(
																codepoint,
															),
															parser
																.position
																.metadata_at(codepoint_span),
														));
													}
												}
											}
										}
									}
								}
							}
							unexpected => {
								break Err(Meta(
									Error::unexpected(unexpected),
									parser.position.last(),
								))
							}
						},
						Some(c) if !is_control(c) => c,
						unexpected => {
							break Err(Meta(Error::unexpected(unexpected), parser.position.last()))
						}
					};

					if let Some(Meta(high, span)) = high_surrogate.take() {
						if parser.options.accept_truncated_surrogate_pair {
							result.push('\u{fffd}');
						} else {
							break Err(Meta(
								Error::MissingLowSurrogate(Meta(
									high as u16,
									parser.position.metadata_at(span),
								)),
								parser.position.current(),
							));
						}
					}

					result.push(c);
					parser.position.span.clear();
				}
			}
			unexpected => Err(Meta(Error::unexpected(unexpected), parser.position.last())),
		}
	}
}
