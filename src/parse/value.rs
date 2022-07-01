use super::{array, object, Context, Error, Parse, Parser, ValueOrParse};
use crate::{Array, Entry, Key, NumberBuf, Object, String, Value};
use decoded_char::DecodedChar;
use locspan::Loc;

/// Value fragment.
pub enum Fragment<F> {
	Value(Value<F>),
	BeginArray,
	BeginObject(Loc<Key, F>),
}

impl<F> From<Value<F>> for Fragment<F> {
	fn from(v: Value<F>) -> Self {
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
				Loc(array::StartFragment::Empty, loc) => Loc(Value::Array(Array::new()), loc),
				Loc(array::StartFragment::NonEmpty, loc) => return Ok(Loc(Self::BeginArray, loc)),
			},
			Some('{') => match object::StartFragment::parse_in(parser, context)? {
				Loc(object::StartFragment::Empty, loc) => Loc(Value::Object(Object::new()), loc),
				Loc(object::StartFragment::NonEmpty(key), loc) => {
					return Ok(Loc(Self::BeginObject(key), loc))
				}
			},
			unexpected => return Err(Loc(Error::unexpected(unexpected), parser.position.last())),
		};

		parser.skip_trailing_whitespaces(context)?;

		Ok(value.map(Self::Value))
	}
}

impl<F: Clone> Parse<F> for Value<F> {
	fn parse_in<E, C>(
		parser: &mut Parser<F, E, C>,
		context: Context,
	) -> Result<Loc<Self, F>, Loc<Error<E, F>, F>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		enum Item<F> {
			Array(Loc<Array<F>, F>),
			ArrayItem(Loc<Array<F>, F>),
			Object(Loc<Object<F>, F>),
			ObjectEntry(Loc<Object<F>, F>, Loc<Key, F>),
		}

		let mut stack: Vec<Item<F>> = vec![];
		let mut value: Option<Loc<Value<F>, F>> = None;

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
					Loc(Fragment::Value(value), loc) => break Ok(Loc(value, loc)),
					Loc(Fragment::BeginArray, loc) => {
						stack.push(Item::ArrayItem(Loc(Array::new(), loc)))
					}
					Loc(Fragment::BeginObject(key), loc) => {
						stack.push(Item::ObjectEntry(Loc(Object::new(), loc), key))
					}
				},
				Some(Item::Array(Loc(array, loc))) => {
					match array::ContinueFragment::parse_in(parser, stack_context(&stack, context))?
					{
						Loc(array::ContinueFragment::Item, comma_loc) => {
							stack.push(Item::ArrayItem(Loc(array, loc.with(comma_loc.span()))))
						}
						Loc(array::ContinueFragment::End, closing_loc) => {
							parser.skip_trailing_whitespaces(stack_context(&stack, context))?;
							value = Some(Loc(Value::Array(array), loc.with(closing_loc.span())))
						}
					}
				}
				Some(Item::ArrayItem(Loc(mut array, loc))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::Array)? {
						Loc(Fragment::Value(value), value_loc) => {
							let value_span = value_loc.span();
							array.push(Loc(value, value_loc));
							stack.push(Item::Array(Loc(array, loc.with(value_span))));
						}
						Loc(Fragment::BeginArray, value_loc) => {
							stack.push(Item::Array(Loc(array, loc.with(value_loc.span()))));
							stack.push(Item::ArrayItem(Loc(Array::new(), value_loc)))
						}
						Loc(Fragment::BeginObject(value_key), value_loc) => {
							stack.push(Item::Array(Loc(array, loc.with(value_loc.span()))));
							stack.push(Item::ObjectEntry(Loc(Object::new(), value_loc), value_key))
						}
					}
				}
				Some(Item::Object(Loc(object, loc))) => match object::ContinueFragment::parse_in(
					parser,
					stack_context(&stack, context),
				)? {
					Loc(object::ContinueFragment::Entry(key), comma_key_loc) => stack.push(
						Item::ObjectEntry(Loc(object, loc.with(comma_key_loc.span())), key),
					),
					Loc(object::ContinueFragment::End, closing_loc) => {
						parser.skip_trailing_whitespaces(stack_context(&stack, context))?;
						value = Some(Loc(Value::Object(object), loc.with(closing_loc.span())))
					}
				},
				Some(Item::ObjectEntry(Loc(mut object, loc), key)) => {
					match Fragment::value_or_parse(value.take(), parser, Context::ObjectValue)? {
						Loc(Fragment::Value(value), value_loc) => {
							let value_span = value_loc.span();
							object.push(Entry::new(key, Loc(value, value_loc)));
							stack.push(Item::Object(Loc(object, loc.with(value_span))));
						}
						Loc(Fragment::BeginArray, value_loc) => {
							stack.push(Item::ObjectEntry(
								Loc(object, loc.with(value_loc.span())),
								key,
							));
							stack.push(Item::ArrayItem(Loc(Array::new(), value_loc)))
						}
						Loc(Fragment::BeginObject(value_key), value_loc) => {
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
