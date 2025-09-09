use super::{array, object, Context, Error, Parse, Parser};
use crate::{object::Key, Array, JsonString, NumberBuf, Object, Value};
use decoded_char::DecodedChar;

/// Value fragment.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Fragment {
	Value(Value),
	BeginArray,
	BeginObject((Key, usize)),
}

impl Fragment {
	fn value_or_parse<C, E>(
		value: Option<(Value, usize)>,
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<(Self, usize), Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		match value {
			Some((value, i)) => Ok((value.into(), i)),
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
	) -> Result<(Self, usize), Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		parser.skip_whitespaces()?;

		let (value, i) = match parser.peek_char()? {
			Some('n') => {
				let ((), i) = <()>::parse_in(parser, context)?;
				(Value::Null, i)
			}
			Some('t' | 'f') => {
				let (value, i) = bool::parse_in(parser, context)?;
				(Value::Boolean(value), i)
			}
			Some('0'..='9' | '-') => {
				let (value, i) = NumberBuf::parse_in(parser, context)?;
				(Value::Number(value), i)
			}
			Some('"') => {
				let (value, i) = JsonString::parse_in(parser, context)?;
				(Value::String(value), i)
			}
			Some('[') => match array::StartFragment::parse_in(parser, context)? {
				(array::StartFragment::Empty, span) => (Value::Array(Array::new()), span),
				(array::StartFragment::NonEmpty, span) => return Ok((Self::BeginArray, span)),
			},
			Some('{') => match object::StartFragment::parse_in(parser, context)? {
				(object::StartFragment::Empty, span) => (Value::Object(Object::new()), span),
				(object::StartFragment::NonEmpty(key), span) => {
					return Ok((Self::BeginObject(key), span))
				}
			},
			unexpected => return Err(Error::unexpected(parser.position, unexpected)),
		};

		Ok((Self::Value(value), i))
	}
}

impl Parse for Value {
	fn parse_in<C, E>(
		parser: &mut Parser<C, E>,
		context: Context,
	) -> Result<(Self, usize), Error<E>>
	where
		C: Iterator<Item = Result<DecodedChar, E>>,
	{
		enum StackItem {
			Array((Array, usize)),
			ArrayItem((Array, usize)),
			Object((Object, usize)),
			ObjectEntry((Object, usize), (Key, usize)),
		}

		let mut stack: Vec<StackItem> = vec![];
		let mut value: Option<(Value, usize)> = None;

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
					(Fragment::Value(value), i) => {
						parser.skip_whitespaces()?;
						break match parser.next_char()? {
							(p, Some(c)) => Err(Error::unexpected(p, Some(c))),
							(_, None) => Ok((value, i)),
						};
					}
					(Fragment::BeginArray, i) => {
						stack.push(StackItem::ArrayItem((Array::new(), i)))
					}
					(Fragment::BeginObject(key), i) => {
						stack.push(StackItem::ObjectEntry((Object::new(), i), key))
					}
				},
				Some(StackItem::Array((array, i))) => {
					match array::ContinueFragment::parse_in(parser, i)? {
						array::ContinueFragment::Item => {
							stack.push(StackItem::ArrayItem((array, i)))
						}
						array::ContinueFragment::End => value = Some((Value::Array(array), i)),
					}
				}
				Some(StackItem::ArrayItem((mut array, i))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::Array)? {
						(Fragment::Value(value), _) => {
							array.push(value);
							stack.push(StackItem::Array((array, i)));
						}
						(Fragment::BeginArray, j) => {
							stack.push(StackItem::ArrayItem((array, i)));
							stack.push(StackItem::ArrayItem((Array::new(), j)))
						}
						(Fragment::BeginObject(value_key), j) => {
							stack.push(StackItem::ArrayItem((array, i)));
							stack.push(StackItem::ObjectEntry((Object::new(), j), value_key))
						}
					}
				}
				Some(StackItem::Object((object, i))) => {
					match object::ContinueFragment::parse_in(parser, i)? {
						object::ContinueFragment::Entry(key) => {
							stack.push(StackItem::ObjectEntry((object, i), key))
						}
						object::ContinueFragment::End => value = Some((Value::Object(object), i)),
					}
				}
				Some(StackItem::ObjectEntry((mut object, i), (key, e))) => {
					match Fragment::value_or_parse(value.take(), parser, Context::ObjectValue)? {
						(Fragment::Value(value), _) => {
							parser.end_fragment(e);
							object.push_back(key, value);
							stack.push(StackItem::Object((object, i)));
						}
						(Fragment::BeginArray, j) => {
							stack.push(StackItem::ObjectEntry((object, i), (key, e)));
							stack.push(StackItem::ArrayItem((Array::new(), j)))
						}
						(Fragment::BeginObject(value_key), j) => {
							stack.push(StackItem::ObjectEntry((object, i), (key, e)));
							stack.push(StackItem::ObjectEntry((Object::new(), j), value_key))
						}
					}
				}
			}
		}
	}
}
