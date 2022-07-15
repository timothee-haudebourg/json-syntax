use super::{array, object, Context, Error, Parse, Parser, ValueOrParse};
use crate::{object::Key, Array, NumberBuf, Object, String, Value};
use decoded_char::DecodedChar;
use locspan::{Loc, Location, Meta};
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
#[stripped_ignore(F)]
pub enum Fragment<F> {
	Value(Value<Location<F>>),
	BeginArray,
	BeginObject(#[stripped_deref] Loc<Key, F>),
}

impl<F> From<Value<Location<F>>> for Fragment<F> {
	fn from(v: Value<Location<F>>) -> Self {
		Self::Value(v)
	}
}

impl<F: Clone> Parse<F> for Fragment<F> {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		parser.skip_whitespaces()?;

		let value = match parser.peek_char()? {
			Some('n') => <()>::parse_in(parser, context)?.map(|()| Value::Null),
			Some('t' | 'f') => bool::parse_in(parser, context)?.map(Value::Boolean),
			Some('0'..='9' | '-') => NumberBuf::parse_in(parser, context)?.map(Value::Number),
			Some('"') => String::parse_in(parser, context)?.map(Value::String),
			Some('[') => match array::StartFragment::parse_in(parser, context)? {
				Meta(array::StartFragment::Empty, loc) => Loc(Value::Array(Array::new()), loc),
				Meta(array::StartFragment::NonEmpty, loc) => return Ok(Loc(Self::BeginArray, loc)),
			},
			Some('{') => match object::StartFragment::parse_in(parser, context)? {
				Meta(object::StartFragment::Empty, loc) => Loc(Value::Object(Object::new()), loc),
				Meta(object::StartFragment::NonEmpty(key), loc) => {
					return Ok(Loc(Self::BeginObject(key), loc))
				}
			},
			unexpected => return Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		};

		parser.skip_trailing_whitespaces(context)?;

		Ok(value.map(Self::Value))
	}
}

impl<F: Clone> Parse<F> for Value<Location<F>> {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		enum Item<F> {
			Array(Loc<Array<Location<F>>, F>),
			ArrayItem(Loc<Array<Location<F>>, F>),
			Object(Loc<Object<Location<F>>, F>),
			ObjectEntry(Loc<Object<Location<F>>, F>, Loc<Key, F>),
		}

		let mut stack: Vec<Item<F>> = vec![];
		let mut value: Option<Loc<Value<Location<F>>, F>> = None;

		fn stack_context<F>(stack: &[Item<F>], root: Context) -> Context {
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
					Meta(Fragment::Value(value), loc) => break Ok(Loc(value, loc)),
					Meta(Fragment::BeginArray, loc) => {
						stack.push(Item::ArrayItem(Loc(Array::new(), loc)))
					}
					Meta(Fragment::BeginObject(key), loc) => {
						stack.push(Item::ObjectEntry(Loc(Object::new(), loc), key))
					}
				},
				Some(Item::Array(Meta(array, loc))) => {
					match array::ContinueFragment::parse_in(parser, stack_context(&stack, context))?
					{
						Meta(array::ContinueFragment::Item, comma_loc) => {
							stack.push(Item::ArrayItem(Loc(array, loc.with(comma_loc.span()))))
						}
						Meta(array::ContinueFragment::End, closing_loc) => {
							parser.skip_trailing_whitespaces(stack_context(&stack, context))?;
							value = Some(Loc(Value::Array(array), loc.with(closing_loc.span())))
						}
					}
				}
				Some(Item::ArrayItem(Meta(mut array, loc))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::Array)? {
						Meta(Fragment::Value(value), value_loc) => {
							let value_span = value_loc.span();
							array.push(Loc(value, value_loc));
							stack.push(Item::Array(Loc(array, loc.with(value_span))));
						}
						Meta(Fragment::BeginArray, value_loc) => {
							stack.push(Item::ArrayItem(Loc(array, loc.with(value_loc.span()))));
							stack.push(Item::ArrayItem(Loc(Array::new(), value_loc)))
						}
						Meta(Fragment::BeginObject(value_key), value_loc) => {
							stack.push(Item::ArrayItem(Loc(array, loc.with(value_loc.span()))));
							stack.push(Item::ObjectEntry(Loc(Object::new(), value_loc), value_key))
						}
					}
				}
				Some(Item::Object(Meta(object, loc))) => match object::ContinueFragment::parse_in(
					parser,
					stack_context(&stack, context),
				)? {
					Meta(object::ContinueFragment::Entry(key), comma_key_loc) => stack.push(
						Item::ObjectEntry(Loc(object, loc.with(comma_key_loc.span())), key),
					),
					Meta(object::ContinueFragment::End, closing_loc) => {
						parser.skip_trailing_whitespaces(stack_context(&stack, context))?;
						value = Some(Loc(Value::Object(object), loc.with(closing_loc.span())))
					}
				},
				Some(Item::ObjectEntry(Meta(mut object, loc), key)) => {
					match Fragment::value_or_parse(value.take(), parser, Context::ObjectValue)? {
						Meta(Fragment::Value(value), value_loc) => {
							let value_span = value_loc.span();
							object.push(key, Loc(value, value_loc));
							stack.push(Item::Object(Loc(object, loc.with(value_span))));
						}
						Meta(Fragment::BeginArray, value_loc) => {
							stack.push(Item::ObjectEntry(
								Loc(object, loc.with(value_loc.span())),
								key,
							));
							stack.push(Item::ArrayItem(Loc(Array::new(), value_loc)))
						}
						Meta(Fragment::BeginObject(value_key), value_loc) => {
							stack.push(Item::ObjectEntry(
								Loc(object, loc.with(value_loc.span())),
								key,
							));
							stack.push(Item::ObjectEntry(Loc(Object::new(), value_loc), value_key))
						}
					}
				}
			}
		}
	}
}
