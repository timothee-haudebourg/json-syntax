use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use std::{fmt, io};

mod array;
mod boolean;
mod null;
mod number;
mod object;
mod string;
mod value;

use crate::CodeMap;

/// Parser options.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Options {
	/// Whether or not to accept a high surrogate without its low counterpart
	/// in strings.
	///
	/// In such instance, the high surrogate will be replaced with the Unicode
	/// REPLACEMENT CHARACTER, U+FFFD.
	pub accept_truncated_surrogate_pair: bool,

	/// Whether or not to accept invalid Unicode codepoints in strings.
	///
	/// Invalid codepoints will be replaced with the Unicode
	/// REPLACEMENT CHARACTER, U+FFFD.
	pub accept_invalid_codepoints: bool,
}

impl Options {
	/// Strict mode.
	///
	/// All options are set to `false`.
	pub fn strict() -> Self {
		Self {
			accept_truncated_surrogate_pair: false,
			accept_invalid_codepoints: false,
		}
	}

	/// Flexible mode.
	///
	/// All options are set to `true`.
	pub fn flexible() -> Self {
		Self {
			accept_truncated_surrogate_pair: true,
			accept_invalid_codepoints: true,
		}
	}
}

impl Default for Options {
	fn default() -> Self {
		Self::strict()
	}
}

pub trait Parse: Sized {
	fn parse_slice(content: &[u8]) -> Result<(Self, CodeMap), Error> {
		Self::parse_utf8(utf8_decode::Decoder::new(content.iter().copied()))
			.map_err(Error::io_into_utf8)
	}

	fn parse_slice_with(content: &[u8], options: Options) -> Result<(Self, CodeMap), Error> {
		Self::parse_utf8_with(utf8_decode::Decoder::new(content.iter().copied()), options)
			.map_err(Error::io_into_utf8)
	}

	fn parse_str(content: &str) -> Result<(Self, CodeMap), Error> {
		Self::parse_utf8(content.chars().map(Ok))
	}

	fn parse_str_with(content: &str, options: Options) -> Result<(Self, CodeMap), Error> {
		Self::parse_utf8_with(content.chars().map(Ok), options)
	}

	fn parse_infallible_utf8<C>(chars: C) -> Result<(Self, CodeMap), Error>
	where
		C: Iterator<Item = char>,
	{
		Self::parse_infallible(chars.map(DecodedChar::from_utf8))
	}

	fn parse_utf8_infallible_with<C>(chars: C, options: Options) -> Result<(Self, CodeMap), Error>
	where
		C: Iterator<Item = char>,
	{
		Self::parse_infallible_with(chars.map(DecodedChar::from_utf8), options)
	}

	fn parse_utf8<C, E>(chars: C) -> Result<(Self, CodeMap), Error<E>>
	where
		C: Iterator<Item = Result<char, E>>,
	{
		Self::parse(chars.map(|c| c.map(DecodedChar::from_utf8)))
	}

	fn parse_utf8_with<C, E>(chars: C, options: Options) -> Result<(Self, CodeMap), Error<E>>
	where
		C: Iterator<Item = Result<char, E>>,
	{
		Self::parse_with(chars.map(|c| c.map(DecodedChar::from_utf8)), options)
	}

	fn parse_infallible<C>(chars: C) -> Result<(Self, CodeMap), Error>
	where
		C: Iterator<Item = DecodedChar>,
	{
		let mut parser = Parser::new(chars.map(Ok));
		let value = Self::parse_in(&mut parser, Context::None)?.into_value();
		Ok((value, parser.code_map))
	}

	fn parse_infallible_with<C>(chars: C, options: Options) -> Result<(Self, CodeMap), Error>
	where
		C: Iterator<Item = DecodedChar>,
	{
		let mut parser = Parser::new_with(chars.map(Ok), options);
		let value = Self::parse_in(&mut parser, Context::None)?.into_value();
		Ok((value, parser.code_map))
	}

	fn parse<C, E>(chars: C) -> Result<(Self, CodeMap), Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let mut parser = Parser::new(chars);
		let value = Self::parse_in(&mut parser, Context::None)?.into_value();
		Ok((value, parser.code_map))
	}

	fn parse_with<C, E>(chars: C, options: Options) -> Result<(Self, CodeMap), Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let mut parser = Parser::new_with(chars, options);
		let value = Self::parse_in(&mut parser, Context::None)?.into_value();
		Ok((value, parser.code_map))
	}

	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>;
}

/// JSON parser.
pub struct Parser<C: Iterator<Item = Result<DecodedChar, E>>, E> {
	/// Character stream.
	chars: C,

	/// Pending next char.
	pending: Option<DecodedChar>,

	/// Position in the stream.
	position: usize,

	/// Parser options.
	options: Options,

	/// Code-map.
	code_map: CodeMap,
}

/// Checks if the given char `c` is a JSON whitespace.
#[inline(always)]
pub fn is_whitespace(c: char) -> bool {
	matches!(c, ' ' | '\t' | '\r' | '\n')
}

impl<C: Iterator<Item = Result<DecodedChar, E>>, E> Parser<C, E> {
	pub fn new(chars: C) -> Self {
		Self {
			chars,
			pending: None,
			position: 0,
			options: Options::default(),
			code_map: CodeMap::default(),
		}
	}

	pub fn new_with(chars: C, options: Options) -> Self {
		Self {
			chars,
			pending: None,
			position: 0,
			options,
			code_map: CodeMap::default(),
		}
	}

	fn begin_fragment(&mut self) -> usize {
		self.code_map.reserve(self.position)
	}

	fn end_fragment(&mut self, i: usize) {
		let entry_count = self.code_map.len();
		let entry = self.code_map.get_mut(i).unwrap();
		entry.span.set_end(self.position);
		entry.volume = entry_count - i;
	}

	fn peek_char(&mut self) -> Result<Option<char>, Error<E>> {
		match self.pending {
			Some(c) => Ok(Some(c.chr())),
			None => match self.chars.next() {
				Some(Ok(c)) => {
					self.pending = Some(c);
					Ok(Some(c.chr()))
				}
				Some(Err(e)) => Err(Error::Stream(self.position, e)),
				None => Ok(None),
			},
		}
	}

	fn next_char(&mut self) -> Result<(usize, Option<char>), Error<E>> {
		let c = match self.pending.take() {
			Some(c) => Some(c),
			None => self
				.chars
				.next()
				.transpose()
				.map_err(|e| Error::Stream(self.position, e))?,
		};

		let p = self.position;
		let c = c.map(|c| {
			self.position += c.len();
			c.chr()
		});

		Ok((p, c))
	}

	fn skip_whitespaces(&mut self) -> Result<(), Error<E>> {
		while let Some(c) = self.peek_char()? {
			if is_whitespace(c) {
				self.next_char()?;
			} else {
				break;
			}
		}

		Ok(())
	}
}

/// Parse error.
#[derive(Debug)]
pub enum Error<E = core::convert::Infallible> {
	/// Stream error.
	///
	/// The first parameter is the byte index at which the error occurred.
	Stream(usize, E),

	/// Unexpected character or end of stream.
	///
	/// The first parameter is the byte index at which the error occurred.
	Unexpected(usize, Option<char>),

	/// Invalid unicode codepoint.
	///
	/// The first parameter is the span at which the error occurred.
	InvalidUnicodeCodePoint(Span, u32),

	/// Missing low surrogate in a string.
	///
	/// The first parameter is the byte index at which the error occurred.
	MissingLowSurrogate(Span, u16),

	/// Invalid low surrogate in a string.
	///
	/// The first parameter is the span at which the error occurred.
	InvalidLowSurrogate(Span, u16, u32),

	/// UTF-8 encoding error.
	InvalidUtf8(usize),
}

impl<E> Error<E> {
	/// Creates an `Unexpected` error.
	#[inline(always)]
	fn unexpected(position: usize, c: Option<char>) -> Self {
		// panic!("unexpected {:?}", c);
		Self::Unexpected(position, c)
	}

	pub fn position(&self) -> usize {
		match self {
			Self::Stream(p, _) => *p,
			Self::Unexpected(p, _) => *p,
			Self::InvalidUnicodeCodePoint(span, _) => span.start(),
			Self::MissingLowSurrogate(span, _) => span.start(),
			Self::InvalidLowSurrogate(span, _, _) => span.start(),
			Self::InvalidUtf8(p) => *p,
		}
	}

	pub fn span(&self) -> Span {
		match self {
			Self::Stream(p, _) => Span::new(*p, *p),
			Self::Unexpected(p, _) => Span::new(*p, *p),
			Self::InvalidUnicodeCodePoint(span, _) => *span,
			Self::MissingLowSurrogate(span, _) => *span,
			Self::InvalidLowSurrogate(span, _, _) => *span,
			Self::InvalidUtf8(p) => Span::new(*p, *p),
		}
	}
}

impl Error<io::Error> {
	fn io_into_utf8(self) -> Error {
		match self {
			Self::Stream(p, _) => Error::InvalidUtf8(p),
			Self::Unexpected(p, e) => Error::Unexpected(p, e),
			Self::InvalidUnicodeCodePoint(s, e) => Error::InvalidUnicodeCodePoint(s, e),
			Self::MissingLowSurrogate(s, e) => Error::MissingLowSurrogate(s, e),
			Self::InvalidLowSurrogate(s, a, b) => Error::InvalidLowSurrogate(s, a, b),
			Self::InvalidUtf8(p) => Error::InvalidUtf8(p),
		}
	}
}

impl<E: fmt::Display> fmt::Display for Error<E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Stream(_, e) => e.fmt(f),
			Self::Unexpected(_, Some(c)) => write!(f, "unexpected character `{}`", c),
			Self::Unexpected(_, None) => write!(f, "unexpected end of file"),
			Self::InvalidUnicodeCodePoint(_, c) => write!(f, "invalid Unicode code point {:x}", *c),
			Self::MissingLowSurrogate(_, _) => write!(f, "missing low surrogate"),
			Self::InvalidLowSurrogate(_, _, _) => write!(f, "invalid low surrogate"),
			Self::InvalidUtf8(_) => write!(f, "invalid UTF-8"),
		}
	}
}

impl<E: 'static + std::error::Error> std::error::Error for Error<E> {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Stream(_, e) => Some(e),
			_ => None,
		}
	}
}

/// Parsing context.
///
/// Defines what characters are allowed after a value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Context {
	None,
	Array,
	ObjectKey,
	ObjectValue,
}

impl Context {
	/// Checks if the given character `c` can follow a value in this context.
	pub fn follows(&self, c: char) -> bool {
		match self {
			Self::None => is_whitespace(c),
			Self::Array => is_whitespace(c) || matches!(c, ',' | ']'),
			Self::ObjectKey => is_whitespace(c) || matches!(c, ':'),
			Self::ObjectValue => is_whitespace(c) || matches!(c, ',' | '}'),
		}
	}
}
