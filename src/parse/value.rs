use super::{array, object, Context, Error, Parse, Parser, ValueOrParse};
use crate::{object::Key, Array, NumberBuf, Object, String, Value};
use decoded_char::DecodedChar;
use locspan::{Meta, Span};
use locspan_derive::*;

/// Value fragment.
#[derive(
	Clone,
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
#[locspan(ignore(F))]
pub enum Fragment<M> {
	Value(Value<M>),
	BeginArray,
	BeginObject(#[locspan(deref_stripped)] Meta<Key, M>),
}

impl<M> From<Value<M>> for Fragment<M> {
	fn from(v: Value<M>) -> Self {
		Self::Value(v)
	}
}

impl<M> Parse<M> for Fragment<M> {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		parser.skip_whitespaces()?;

		let value = match parser.peek_char()? {
			Some('n') => <()>::parse_spanned(parser, context)?.map(|()| Value::Null),
			Some('t' | 'f') => bool::parse_spanned(parser, context)?.map(Value::Boolean),
			Some('0'..='9' | '-') => NumberBuf::parse_spanned(parser, context)?.map(Value::Number),
			Some('"') => String::parse_spanned(parser, context)?.map(Value::String),
			Some('[') => match array::StartFragment::parse_spanned(parser, context)? {
				Meta(array::StartFragment::Empty, span) => Meta(Value::Array(Array::new()), span),
				Meta(array::StartFragment::NonEmpty, span) => {
					return Ok(Meta(Self::BeginArray, span))
				}
			},
			Some('{') => match object::StartFragment::parse_spanned(parser, context)? {
				Meta(object::StartFragment::Empty, span) => {
					Meta(Value::Object(Object::new()), span)
				}
				Meta(object::StartFragment::NonEmpty(key), span) => {
					return Ok(Meta(Self::BeginObject(key), span))
				}
			},
			unexpected => return Err(Meta(Error::unexpected(unexpected), parser.position.last())),
		};

		parser.skip_trailing_whitespaces(context)?;

		Ok(value.map(Self::Value))
	}
}

impl<M> Parse<M> for Value<M> {
	fn parse_spanned<C, F, E>(
		parser: &mut Parser<C, F, E>,
		context: Context,
	) -> Result<Meta<Self, Span>, Meta<Error<M, E>, M>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
		F: FnMut(Span) -> M,
	{
		enum Item<M> {
			Array(Meta<Array<M>, Span>),
			ArrayItem(Meta<Array<M>, Span>),
			Object(Meta<Object<M>, Span>),
			ObjectEntry(Meta<Object<M>, Span>, Meta<Key, M>),
		}

		let mut stack: Vec<Item<M>> = vec![];
		let mut value: Option<Meta<Value<M>, Span>> = None;

		fn stack_context<M>(stack: &[Item<M>], root: Context) -> Context {
			match stack.last() {
				Some(Item::Array(_) | Item::ArrayItem(_)) => Context::Array,
				Some(Item::Object(_)) => Context::ObjectKey,
				Some(Item::ObjectEntry(_, _)) => Context::ObjectValue,
				None => root,
			}
		}

		loop {
			match stack.pop() {
				None => match Fragment::value_or_parse(
					value.take(),
					parser,
					stack_context(&stack, context),
				)? {
					Meta(Fragment::Value(value), span) => break Ok(Meta(value, span)),
					Meta(Fragment::BeginArray, span) => {
						stack.push(Item::ArrayItem(Meta(Array::new(), span)))
					}
					Meta(Fragment::BeginObject(key), span) => {
						stack.push(Item::ObjectEntry(Meta(Object::new(), span), key))
					}
				},
				Some(Item::Array(Meta(array, span))) => {
					match array::ContinueFragment::parse_spanned(
						parser,
						stack_context(&stack, context),
					)? {
						Meta(array::ContinueFragment::Item, comma_span) => {
							stack.push(Item::ArrayItem(Meta(array, span.union(comma_span))))
						}
						Meta(array::ContinueFragment::End, closing_span) => {
							parser.skip_trailing_whitespaces(stack_context(&stack, context))?;
							value = Some(Meta(Value::Array(array), span.union(closing_span)))
						}
					}
				}
				Some(Item::ArrayItem(Meta(mut array, span))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::Array)? {
						Meta(Fragment::Value(value), value_span) => {
							array.push(Meta(value, parser.position.metadata_at(value_span)));
							stack.push(Item::Array(Meta(array, span.union(value_span))));
						}
						Meta(Fragment::BeginArray, value_span) => {
							stack.push(Item::ArrayItem(Meta(array, span.union(value_span))));
							stack.push(Item::ArrayItem(Meta(Array::new(), value_span)))
						}
						Meta(Fragment::BeginObject(value_key), value_span) => {
							stack.push(Item::ArrayItem(Meta(array, span.union(value_span))));
							stack.push(Item::ObjectEntry(
								Meta(Object::new(), value_span),
								value_key,
							))
						}
					}
				}
				Some(Item::Object(Meta(object, span))) => {
					match object::ContinueFragment::parse_spanned(
						parser,
						stack_context(&stack, context),
					)? {
						Meta(object::ContinueFragment::Entry(key), comma_key_span) => stack.push(
							Item::ObjectEntry(Meta(object, span.union(comma_key_span)), key),
						),
						Meta(object::ContinueFragment::End, closing_span) => {
							parser.skip_trailing_whitespaces(stack_context(&stack, context))?;
							value = Some(Meta(Value::Object(object), span.union(closing_span)))
						}
					}
				}
				Some(Item::ObjectEntry(Meta(mut object, span), key)) => {
					match Fragment::value_or_parse(value.take(), parser, Context::ObjectValue)? {
						Meta(Fragment::Value(value), value_span) => {
							object.push(key, Meta(value, parser.position.metadata_at(value_span)));
							stack.push(Item::Object(Meta(object, span.union(value_span))));
						}
						Meta(Fragment::BeginArray, value_span) => {
							stack
								.push(Item::ObjectEntry(Meta(object, span.union(value_span)), key));
							stack.push(Item::ArrayItem(Meta(Array::new(), value_span)))
						}
						Meta(Fragment::BeginObject(value_key), value_span) => {
							stack
								.push(Item::ObjectEntry(Meta(object, span.union(value_span)), key));
							stack.push(Item::ObjectEntry(
								Meta(Object::new(), value_span),
								value_key,
							))
						}
					}
				}
			}
		}
	}
}
