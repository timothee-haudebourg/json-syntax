use crate::Value;
use decoded_char::DecodedChar;
use locspan::{Loc, Location, Span};
use std::iter::Peekable;

mod array;
mod boolean;
mod null;
mod number;
mod object;
mod string;
mod value;

/// Parser options.
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

pub trait Parse<F>: Sized {
	fn parse<E, C>(file: F, chars: C) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let mut parser = Parser::new(file, chars);
		Self::parse_in(&mut parser, Context::None)
	}

	fn parse_with<E, C>(
		file: F,
		chars: C,
		options: Options,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		let mut parser = Parser::new_with(file, chars, options);
		Self::parse_in(&mut parser, Context::None)
	}

	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>;
}

pub trait ValueOrParse<F>: Parse<F> {
	fn value_or_parse<E, C>(
		value: Option<Loc<Value<F>, F>>,
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>;
}

impl<F, T> ValueOrParse<F> for T
where
	T: Parse<F> + From<Value<F>>,
{
	fn value_or_parse<E, C>(
		value: Option<Loc<Value<F>, F>>,
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match value {
			Some(value) => Ok(value.cast()),
			None => Self::parse_in(parser, context),
		}
	}
}

/// JSON parser.
pub struct Parser<F, E, C: Iterator<Item = Result<DecodedChar, E>>> {
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

impl<F, E, C: Iterator<Item = Result<DecodedChar, E>>> Parser<F, E, C> {
	pub fn new(file: F, chars: C) -> Self {
		Self {
			chars: chars.peekable(),
			position: Position::new(file),
			options: Options::default(),
		}
	}

	pub fn new_with(file: F, chars: C, options: Options) -> Self {
		Self {
			chars: chars.peekable(),
			position: Position::new(file),
			options,
		}
	}

	fn peek_char(&mut self) -> Result<Option<char>, Loc<Error<E, F>, F>>
	where
		F: Clone,
	{
		match self.chars.peek() {
			None => Ok(None),
			Some(Ok(c)) => Ok(Some(c.chr())),
			Some(Err(_)) => self.next_char(),
		}
	}

	fn next_char(&mut self) -> Result<Option<char>, Loc<Error<E, F>, F>>
	where
		F: Clone,
	{
		match self.chars.next() {
			None => Ok(None),
			Some(Ok(c)) => {
				self.position.span.push(c.len());
				self.position.last_span.clear();
				self.position.last_span.push(c.len());
				Ok(Some(c.into_char()))
			}
			Some(Err(e)) => Err(Loc(Error::Stream(e), self.position.end())),
		}
	}

	fn skip_whitespaces(&mut self) -> Result<(), Loc<Error<E, F>, F>>
	where
		F: Clone,
	{
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

	fn skip_trailing_whitespaces(&mut self, context: Context) -> Result<(), Loc<Error<E, F>, F>>
	where
		F: Clone,
	{
		self.skip_whitespaces()?;

		if let Some(c) = self.peek_char()? {
			if !context.follows(c) {
				// panic!("unexpected {:?} in {:?}", c, context);
				return Err(Loc(Error::unexpected(Some(c)), self.position.last()));
			}
		}

		Ok(())
	}
}

/// Parse error.
#[derive(Debug)]
pub enum Error<E, F> {
	/// Stream error.
	Stream(E),

	/// Unexpected character or end of stream.
	Unexpected(Option<char>),

	/// Invalid unicode codepoint.
	InvalidUnicodeCodePoint(u32),

	/// Missing low surrogate in a string.
	MissingLowSurrogate(Loc<u16, F>),

	/// Invalid low surrogate in a string.
	InvalidLowSurrogate(Loc<u16, F>, u32),
}

impl<E, F> Error<E, F> {
	/// Creates an `Unexpected` error.
	#[inline(always)]
	fn unexpected(c: Option<char>) -> Self {
		// panic!("unexpected {:?}", c);
		Self::Unexpected(c)
	}
}

/// Lexer position.
struct Position<F> {
	file: F,
	span: Span,
	last_span: Span,
}

impl<F> Position<F> {
	fn new(file: F) -> Self {
		Self {
			file,
			span: Span::default(),
			last_span: Span::default(),
		}
	}

	fn current(&self) -> Location<F>
	where
		F: Clone,
	{
		Location::new(self.file.clone(), self.span)
	}

	fn end(&self) -> Location<F>
	where
		F: Clone,
	{
		Location::new(self.file.clone(), self.span.end().into())
	}

	fn last(&self) -> Location<F>
	where
		F: Clone,
	{
		Location::new(self.file.clone(), self.last_span)
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
