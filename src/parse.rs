use crate::Value;
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use std::fmt;
use std::iter::Peekable;

mod array;
mod boolean;
mod null;
mod number;
mod object;
mod string;
mod value;

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

pub trait Parse<M>: Sized {
	fn parse_str<F>(content: &str, metadata_builder: F) -> Result<Meta<Self, M>, Meta<Error<M>, M>>
	where
		F: FnMut(Span) -> M,
	{
		Self::parse_utf8(content.chars().map(Ok), metadata_builder)
	}

	fn parse_str_with<F>(
		content: &str,
		options: Options,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M>, M>>
	where
		F: FnMut(Span) -> M,
	{
		Self::parse_utf8_with(content.chars().map(Ok), options, metadata_builder)
	}

	fn parse_infallible_utf8<C, F>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M>, M>>
	where
		C: Iterator<Item = char>,
		F: FnMut(Span) -> M,
	{
		Self::parse_infallible(chars.map(DecodedChar::from_utf8), metadata_builder)
	}

	fn parse_utf8_infallible_with<C, F>(
		chars: C,
		options: Options,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M>, M>>
	where
		C: Iterator<Item = char>,
		F: FnMut(Span) -> M,
	{
		Self::parse_infallible_with(chars.map(DecodedChar::from_utf8), options, metadata_builder)
	}

	fn parse_utf8<C, F, E>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<char, E>>,
		F: FnMut(Span) -> M,
	{
		Self::parse(
			chars.map(|c| c.map(DecodedChar::from_utf8)),
			metadata_builder,
		)
	}

	fn parse_utf8_with<C, F, E>(
		chars: C,
		options: Options,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<char, E>>,
		F: FnMut(Span) -> M,
	{
		Self::parse_with(
			chars.map(|c| c.map(DecodedChar::from_utf8)),
			options,
			metadata_builder,
		)
	}

	fn parse_infallible<C, F>(
		chars: C,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M>, M>>
	where
		C: Iterator<Item = DecodedChar>,
		F: FnMut(Span) -> M,
	{
		let mut parser = Parser::new(chars.map(Ok), metadata_builder);
		Self::parse_in(&mut parser, Context::None)
	}

	fn parse_infallible_with<C, F>(
		chars: C,
		options: Options,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M>, M>>
	where
		C: Iterator<Item = DecodedChar>,
		F: FnMut(Span) -> M,
	{
		let mut parser = Parser::new_with(chars.map(Ok), options, metadata_builder);
		Self::parse_in(&mut parser, Context::None)
	}

	fn parse<C, F, E>(chars: C, metadata_builder: F) -> Result<Meta<Self, M>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		let mut parser = Parser::new(chars, metadata_builder);
		Self::parse_in(&mut parser, Context::None)
	}

	fn parse_with<C, F, E>(
		chars: C,
		options: Options,
		metadata_builder: F,
	) -> Result<Meta<Self, M>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		let mut parser = Parser::new_with(chars, options, metadata_builder);
		Self::parse_in(&mut parser, Context::None)
	}

	fn parse_in<C, F, E>(
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, M>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		let Meta(value, span) = Self::parse_spanned(parser, context)?;
		Ok(Meta(value, parser.position.metadata_at(span)))
	}

	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M;
}

pub trait ValueOrParse<M>: Parse<M> {
	fn value_or_parse<C, F, E>(
		value: Option<Meta<Value<M>, Span>>,
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M;
}

impl<T, M> ValueOrParse<M> for T
where
	T: Parse<M> + From<Value<M>>,
{
	fn value_or_parse<C, F, E>(
		value: Option<Meta<Value<M>, Span>>,
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		match value {
			Some(value) => Ok(value.cast()),
			None => Self::parse_spanned(parser, context),
		}
	}
}

/// JSON parser.
pub struct Parser<C: Iterator<Item = Result<DecodedChar, E>>, F, E> {
	/// Character stream.
	chars: Peekable<C>,

	/// Position in the stream.
	position: Position<F>,

	/// Parser options.
	options: Options,
}

/// Checks if the given char `c` is a JSON whitespace.
#[inline(always)]
pub fn is_whitespace(c: char) -> bool {
	matches!(c, ' ' | '\t' | '\r' | '\n')
}

impl<C: Iterator<Item = Result<DecodedChar, E>>, F, M, E> Parser<C, F, E>
where
	F: FnMut(Span) -> M,
{
	pub fn new(chars: C, metadata_builder: F) -> Self {
		Self {
			chars: chars.peekable(),
			position: Position::new(metadata_builder),
			options: Options::default(),
		}
	}

	pub fn new_with(chars: C, options: Options, metadata_builder: F) -> Self {
		Self {
			chars: chars.peekable(),
			position: Position::new(metadata_builder),
			options,
		}
	}

	fn peek_char(&mut self) -> Result<Option<char>, Meta<Error<M, E>, M>> {
		match self.chars.peek() {
			None => Ok(None),
			Some(Ok(c)) => Ok(Some(c.chr())),
			Some(Err(_)) => self.next_char(),
		}
	}

	fn next_char(&mut self) -> Result<Option<char>, Meta<Error<M, E>, M>> {
		match self.chars.next() {
			None => Ok(None),
			Some(Ok(c)) => {
				self.position.span.push(c.len());
				self.position.last_span.clear();
				self.position.last_span.push(c.len());
				Ok(Some(c.into_char()))
			}
			Some(Err(e)) => Err(Meta(Error::Stream(e), self.position.end())),
		}
	}

	fn skip_whitespaces(&mut self) -> Result<(), Meta<Error<M, E>, M>> {
		while let Some(c) = self.peek_char()? {
			if is_whitespace(c) {
				self.next_char()?;
			} else {
				break;
			}
		}

		self.position.span.clear();
		Ok(())
	}

	fn skip_trailing_whitespaces(&mut self, context: Context) -> Result<(), Meta<Error<M, E>, M>> {
		self.skip_whitespaces()?;

		if let Some(c) = self.peek_char()? {
			if !context.follows(c) {
				// panic!("unexpected {:?} in {:?}", c, context);
				return Err(Meta(Error::unexpected(Some(c)), self.position.last()));
			}
		}

		Ok(())
	}
}

/// Parse error.
#[derive(Debug)]
pub enum Error<M, E = core::convert::Infallible> {
	/// Stream error.
	Stream(E),

	/// Unexpected character or end of stream.
	Unexpected(Option<char>),

	/// Invalid unicode codepoint.
	InvalidUnicodeCodePoint(u32),

	/// Missing low surrogate in a string.
	MissingLowSurrogate(Meta<u16, M>),

	/// Invalid low surrogate in a string.
	InvalidLowSurrogate(Meta<u16, M>, u32),
}

impl<M, E> Error<M, E> {
	/// Creates an `Unexpected` error.
	#[inline(always)]
	fn unexpected(c: Option<char>) -> Self {
		// panic!("unexpected {:?}", c);
		Self::Unexpected(c)
	}
}

impl<E: fmt::Display, M> fmt::Display for Error<M, E> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Stream(e) => e.fmt(f),
			Self::Unexpected(Some(c)) => write!(f, "unexpected character `{}`", c),
			Self::Unexpected(None) => write!(f, "unexpected end of file"),
			Self::InvalidUnicodeCodePoint(c) => write!(f, "invalid Unicode code point {:x}", *c),
			Self::MissingLowSurrogate(_) => write!(f, "missing low surrogate"),
			Self::InvalidLowSurrogate(_, _) => write!(f, "invalid low surrogate"),
		}
	}
}

impl<E: 'static + std::error::Error, M: std::fmt::Debug> std::error::Error for Error<M, E> {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Self::Stream(e) => Some(e),
			_ => None,
		}
	}
}

pub type MetaError<M, E = core::convert::Infallible> = Meta<Error<M, E>, M>;

/// Lexer position.
struct Position<F> {
	span: Span,
	last_span: Span,
	metadata_builder: F,
}

impl<F> Position<F> {
	fn new(metadata_builder: F) -> Self {
		Self {
			span: Span::default(),
			last_span: Span::default(),
			metadata_builder,
		}
	}

	fn current_span(&self) -> Span {
		self.span
	}
}

impl<F: FnMut(Span) -> M, M> Position<F> {
	fn metadata_at(&mut self, span: Span) -> M {
		(self.metadata_builder)(span)
	}

	fn current(&mut self) -> M {
		(self.metadata_builder)(self.span)
	}

	fn end(&mut self) -> M {
		(self.metadata_builder)(self.span.end().into())
	}

	fn last(&mut self) -> M {
		(self.metadata_builder)(self.last_span)
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
