use super::{array, object, Context, Error, Parse, Parser};
use crate::{object::Key, Array, NumberBuf, Object, String, Value};
use decoded_char::DecodedChar;
use locspan::Meta;

/// Value fragment.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Fragment {
	Value(Value),
	BeginArray,
	BeginObject(Meta<Key, usize>),
}

impl Fragment {
	fn value_or_parse<C, E>(
		value: Option<Meta<Value, usize>>,
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match value {
			Some(value) => Ok(value.cast()),
			None => Self::parse_in(parser, context),
		}
	}
}

impl From<Value> for Fragment {
	fn from(v: Value) -> Self {
		Self::Value(v)
	}
}

impl Parse for Fragment {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
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
				Meta(array::StartFragment::Empty, span) => Meta(Value::Array(Array::new()), span),
				Meta(array::StartFragment::NonEmpty, span) => {
					return Ok(Meta(Self::BeginArray, span))
				}
			},
			Some('{') => match object::StartFragment::parse_in(parser, context)? {
				Meta(object::StartFragment::Empty, span) => {
					Meta(Value::Object(Object::new()), span)
				}
				Meta(object::StartFragment::NonEmpty(key), span) => {
					return Ok(Meta(Self::BeginObject(key), span))
				}
			},
			unexpected => return Err(Error::unexpected(parser.position, unexpected)),
		};

		Ok(value.map(Self::Value))
	}
}

impl Parse for Value {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<Meta<Self, usize>, Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		enum StackItem {
			Array(Meta<Array, usize>),
			ArrayItem(Meta<Array, usize>),
			Object(Meta<Object, usize>),
			ObjectEntry(Meta<Object, usize>, Meta<Key, usize>),
		}

		let mut stack: Vec<StackItem> = vec![];
		let mut value: Option<Meta<Value, usize>> = None;

		fn stack_context(stack: &[StackItem], root: Context) -> Context {
			match stack.last() {
				Some(StackItem::Array(_) | StackItem::ArrayItem(_)) => Context::Array,
				Some(StackItem::Object(_)) => Context::ObjectKey,
				Some(StackItem::ObjectEntry(_, _)) => Context::ObjectValue,
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
					Meta(Fragment::Value(value), i) => {
						parser.skip_whitespaces()?;
						break match parser.next_char()? {
							(p, Some(c)) => Err(Error::unexpected(p, Some(c))),
							(_, None) => Ok(Meta(value, i)),
						};
					}
					Meta(Fragment::BeginArray, i) => {
						stack.push(StackItem::ArrayItem(Meta(Array::new(), i)))
					}
					Meta(Fragment::BeginObject(key), i) => {
						stack.push(StackItem::ObjectEntry(Meta(Object::new(), i), key))
					}
				},
				Some(StackItem::Array(Meta(array, i))) => {
					match array::ContinueFragment::parse_in(parser, i)? {
						array::ContinueFragment::Item => {
							stack.push(StackItem::ArrayItem(Meta(array, i)))
						}
						array::ContinueFragment::End => value = Some(Meta(Value::Array(array), i)),
					}
				}
				Some(StackItem::ArrayItem(Meta(mut array, i))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::Array)? {
						Meta(Fragment::Value(value), _) => {
							array.push(value);
							stack.push(StackItem::Array(Meta(array, i)));
						}
						Meta(Fragment::BeginArray, j) => {
							stack.push(StackItem::ArrayItem(Meta(array, i)));
							stack.push(StackItem::ArrayItem(Meta(Array::new(), j)))
						}
						Meta(Fragment::BeginObject(value_key), j) => {
							stack.push(StackItem::ArrayItem(Meta(array, i)));
							stack.push(StackItem::ObjectEntry(Meta(Object::new(), j), value_key))
						}
					}
				}
				Some(StackItem::Object(Meta(object, i))) => {
					match object::ContinueFragment::parse_in(parser, i)? {
						object::ContinueFragment::Entry(key) => {
							stack.push(StackItem::ObjectEntry(Meta(object, i), key))
						}
						object::ContinueFragment::End => {
							value = Some(Meta(Value::Object(object), i))
						}
					}
				}
				Some(StackItem::ObjectEntry(Meta(mut object, i), Meta(key, e))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::ObjectValue)? {
						Meta(Fragment::Value(value), _) => {
							parser.end_fragment(e);
							object.push(key, value);
							stack.push(StackItem::Object(Meta(object, i)));
						}
						Meta(Fragment::BeginArray, j) => {
							stack.push(StackItem::ObjectEntry(Meta(object, i), Meta(key, e)));
							stack.push(StackItem::ArrayItem(Meta(Array::new(), j)))
						}
						Meta(Fragment::BeginObject(value_key), j) => {
							stack.push(StackItem::ObjectEntry(Meta(object, i), Meta(key, e)));
							stack.push(StackItem::ObjectEntry(Meta(Object::new(), j), value_key))
						}
					}
				}
			}
		}
	}
}
