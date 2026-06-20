use std::borrow::{Borrow, ToOwned};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use smallvec::SmallVec;

use crate::{lexical::BorrowJsonLexical, JsonBytes};

pub const DEFAULT_STACK_CAPACITY: usize = 8;

pub type DefaultBuffer = SmallVec<[u8; DEFAULT_STACK_CAPACITY]>;

/// Invalid number error.
///
/// The inner value is the data failed to be parsed.
#[derive(Clone, Copy, Debug)]
pub struct InvalidJsonNumber<T = DefaultBuffer>(pub T);

impl<T: fmt::Display> fmt::Display for InvalidJsonNumber<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "invalid JSON number: {}", self.0)
	}
}

impl<T: fmt::Display + fmt::Debug> std::error::Error for InvalidJsonNumber<T> {}

/// Number sign.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sign {
	Negative,
	Zero,
	Positive,
}

impl Sign {
	/// Checks if the number is zero.
	#[inline(always)]
	pub fn is_zero(&self) -> bool {
		matches!(self, Self::Zero)
	}

	/// Checks if the number is non positive (negative or zero).
	#[inline(always)]
	pub fn is_non_positive(&self) -> bool {
		matches!(self, Self::Negative | Self::Zero)
	}

	/// Checks if the number is non negative (positive or zero).
	#[inline(always)]
	pub fn is_non_negative(&self) -> bool {
		matches!(self, Self::Positive | Self::Zero)
	}

	/// Checks if the number is strictly positive (non zero nor negative).
	#[inline(always)]
	pub fn is_positive(&self) -> bool {
		matches!(self, Self::Positive)
	}

	/// Checks if the number is strictly negative (non zero nor positive).
	#[inline(always)]
	pub fn is_negative(&self) -> bool {
		matches!(self, Self::Negative)
	}
}

/// Lexical JSON number.
///
/// This hold the lexical representation of a JSON number.
/// All the comparison operations are done on this *lexical* representation,
/// meaning that `1` is actually greater than `0.1e+80` for instance.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsonNumber {
	data: [u8],
}

impl JsonNumber {
	/// Creates a new number by parsing the given input `data`.
	pub fn new<B: AsRef<[u8]> + ?Sized>(data: &B) -> Result<&JsonNumber, InvalidJsonNumber<&B>> {
		let s = data.as_ref();

		enum State {
			Init,
			FirstDigit,
			Zero,
			NonZero,
			FractionalFirst,
			FractionalRest,
			ExponentSign,
			ExponentFirst,
			ExponentRest,
		}

		let mut state = State::Init;

		for b in s {
			match state {
				State::Init => match *b {
					b'-' => state = State::FirstDigit,
					b'0' => state = State::Zero,
					b'1'..=b'9' => state = State::NonZero,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::FirstDigit => match *b {
					b'0' => state = State::Zero,
					b'1'..=b'9' => state = State::NonZero,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::Zero => match *b {
					b'.' => state = State::FractionalFirst,
					b'e' | b'E' => state = State::ExponentSign,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::NonZero => match *b {
					b'0'..=b'9' => state = State::NonZero,
					b'.' => state = State::FractionalFirst,
					b'e' | b'E' => state = State::ExponentSign,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::FractionalFirst => match *b {
					b'0'..=b'9' => state = State::FractionalRest,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::FractionalRest => match *b {
					b'0'..=b'9' => state = State::FractionalRest,
					b'e' | b'E' => state = State::ExponentSign,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::ExponentSign => match *b {
					b'+' | b'-' => state = State::ExponentFirst,
					b'0'..=b'9' => state = State::ExponentRest,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::ExponentFirst => match *b {
					b'0'..=b'9' => state = State::ExponentRest,
					_ => return Err(InvalidJsonNumber(data)),
				},
				State::ExponentRest => match *b {
					b'0'..=b'9' => state = State::ExponentRest,
					_ => return Err(InvalidJsonNumber(data)),
				},
			}
		}

		if matches!(
			state,
			State::Zero | State::NonZero | State::FractionalRest | State::ExponentRest
		) {
			Ok(unsafe { Self::new_unchecked(s) })
		} else {
			Err(InvalidJsonNumber(data))
		}
	}

	/// Creates a new number without parsing the given input `data`.
	///
	/// ## Safety
	///
	/// The `data` input **must** be a valid JSON number.
	#[inline(always)]
	pub unsafe fn new_unchecked<B: AsRef<[u8]> + ?Sized>(data: &B) -> &JsonNumber {
		std::mem::transmute(data.as_ref())
	}

	#[inline(always)]
	pub fn as_str(&self) -> &str {
		unsafe {
			// safe because `self.data` is always a valid UTF-8 sequence.
			std::str::from_utf8_unchecked(&self.data)
		}
	}

	/// Trims unnecessary trailing decimal point and zeros.
	///
	/// Also removes a zero exponent (e.g. `e0`, `E+0`) and the negative sign
	/// when the number is zero (e.g. `-0`, `-0.0`).
	pub fn trimmed(&self) -> &Self {
		let data = &self.data;

		// Find the start of the exponent part, if any.
		let exp_pos = data.iter().position(|&b| b == b'e' || b == b'E');
		let mantissa_end = exp_pos.unwrap_or(data.len());

		// Trim trailing fractional zeros from the mantissa only.
		let mut trim_end = 1;
		let mut in_frac = false;
		for (i, b) in data.iter().copied().enumerate().take(mantissa_end).skip(1) {
			match b {
				b'0' if in_frac => (), // trailing fractional zero: drop
				b'.' => in_frac = true,
				_ => trim_end = i + 1,
			}
		}

		// Determine whether the exponent is zero (so it can be dropped).
		let zero_exp = exp_pos.map_or(true, |e| {
			let after_e = &data[e + 1..];
			let digits = if after_e.first().is_some_and(|&b| b == b'+' || b == b'-') {
				&after_e[1..]
			} else {
				after_e
			};
			digits.iter().all(|&b| b == b'0')
		});

		// If the exponent is non-zero we must include it, but we can only return
		// a contiguous subslice, so fractional trimming cannot be combined with a
		// kept exponent. In that case fall back to the original data.
		let end = if zero_exp {
			trim_end
		} else {
			// mantissa was not trimmed; or
			// cannot trim fractional zeros and keep exponent in one slice.
			data.len()
		};

		// Strip the leading `-` when the number is zero.
		let start = if data[0] == b'-' && &data[1..trim_end] == b"0" {
			1
		} else {
			0
		};

		unsafe { Self::new_unchecked(&data[start..end]) }
	}

	/// Checks if the number is equal to zero (`0`).
	///
	/// This include every lexical representation where
	/// the decimal and fraction part are composed of only
	/// `0`, maybe preceded with `-`, and an arbitrary exponent part.
	#[inline(always)]
	pub fn is_zero(&self) -> bool {
		for b in &self.data {
			match b {
				b'-' | b'0' | b'.' => (),
				b'e' | b'E' => break,
				_ => return false,
			}
		}

		true
	}

	/// Returns the sign of the number.
	pub fn sign(&self) -> Sign {
		let mut non_negative = true;

		for b in &self.data {
			match b {
				b'-' => non_negative = false,
				b'0' | b'.' => (),
				b'e' | b'E' => break,
				_ => {
					return if non_negative {
						Sign::Positive
					} else {
						Sign::Negative
					}
				}
			}
		}

		Sign::Zero
	}

	/// Checks if the number is non positive (negative or zero).
	#[inline(always)]
	pub fn is_non_positive(&self) -> bool {
		self.sign().is_non_positive()
	}

	/// Checks if the number is non negative (positive or zero).
	#[inline(always)]
	pub fn is_non_negative(&self) -> bool {
		self.sign().is_non_negative()
	}

	/// Checks if the number is strictly positive (non zero nor negative).
	#[inline(always)]
	pub fn is_positive(&self) -> bool {
		self.sign().is_positive()
	}

	/// Checks if the number is strictly negative (non zero nor positive).
	#[inline(always)]
	pub fn is_negative(&self) -> bool {
		self.sign().is_negative()
	}

	/// Checks if the number has a decimal point.
	#[inline(always)]
	pub fn has_decimal_point(&self) -> bool {
		self.data.contains(&b'.')
	}

	/// Checks if the number has a fraction part.
	///
	/// This is an alias for [`has_decimal_point`](Self::has_decimal_point).
	#[inline(always)]
	pub fn has_fraction(&self) -> bool {
		self.has_decimal_point()
	}

	/// Checks if the number has an exponent part.
	#[inline(always)]
	pub fn has_exponent(&self) -> bool {
		for b in &self.data {
			if matches!(b, b'e' | b'E') {
				return true;
			}
		}

		false
	}

	#[inline(always)]
	pub fn is_i32(&self) -> bool {
		self.as_i32().is_some()
	}

	#[inline(always)]
	pub fn is_i64(&self) -> bool {
		self.as_i64().is_some()
	}

	#[inline(always)]
	pub fn is_u32(&self) -> bool {
		self.as_u32().is_some()
	}

	#[inline(always)]
	pub fn is_u64(&self) -> bool {
		self.as_u64().is_some()
	}

	#[inline(always)]
	pub fn as_i32(&self) -> Option<i32> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_i64(&self) -> Option<i64> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_u32(&self) -> Option<u32> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_u64(&self) -> Option<u64> {
		self.as_str().parse().ok()
	}

	#[inline(always)]
	pub fn as_f32_lossy(&self) -> f32 {
		lexical::parse_with_options::<_, _, { lexical::format::JSON }>(
			self.as_bytes(),
			&LOSSY_PARSE_FLOAT,
		)
		.unwrap()
	}

	/// Returns the number as a `f32` only if the operation does not induce
	/// imprecisions/approximations.
	///
	/// This operation is expensive as it requires allocating a new number
	/// buffer to check the decimal representation of the generated `f32`.
	#[inline(always)]
	pub fn as_f32_lossless(&self) -> Option<f32> {
		let f = self.as_f32_lossy();
		let n: JsonNumberBuf = f.try_into().unwrap();
		if n.as_number() == self.trimmed() {
			Some(f)
		} else {
			None
		}
	}

	#[inline(always)]
	pub fn as_f64_lossy(&self) -> f64 {
		lexical::parse_with_options::<_, _, { lexical::format::JSON }>(
			self.as_bytes(),
			&LOSSY_PARSE_FLOAT,
		)
		.unwrap()
	}

	/// Returns the number as a `f64` only if the operation does not induce
	/// imprecisions/approximations.
	///
	/// This operation is expensive as it requires allocating a new number
	/// buffer to check the decimal representation of the generated `f64`.
	#[inline(always)]
	pub fn as_f64_lossless(&self) -> Option<f64> {
		let f = self.as_f64_lossy();
		let n: JsonNumberBuf = f.try_into().unwrap();
		if n.as_number() == self {
			Some(f)
		} else {
			None
		}
	}

	pub fn to_custom_owned<B: JsonBytes>(&self) -> JsonNumberBuf<B> {
		unsafe { JsonNumberBuf::new_unchecked(B::from_bytes(self.as_bytes())) }
	}

	/// Returns the canonical representation of this number according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalized_with<'b>(&'b self, buffer: &'b mut ryu_js::Buffer) -> &'b JsonNumber {
		let f = self.as_f64_lossy();

		if f.is_finite() {
			unsafe { JsonNumber::new_unchecked(buffer.format_finite(self.as_f64_lossy())) }
		} else {
			self
		}
	}

	/// Returns the canonical representation of this number according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalized(&self) -> JsonNumberBuf {
		let mut buffer = ryu_js::Buffer::new();
		self.canonicalized_with(&mut buffer).to_owned()
	}

	/// Returns the canonical representation of this number according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonicalize")]
	pub fn custom_canonicalized_with<B: JsonBytes>(
		&self,
		buffer: &mut ryu_js::Buffer,
	) -> JsonNumberBuf<B> {
		self.canonicalized_with(buffer).to_custom_owned()
	}

	/// Returns the canonical representation of this number according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonicalize")]
	pub fn custom_canonicalized<B: JsonBytes>(&self) -> JsonNumberBuf<B> {
		let mut buffer = ryu_js::Buffer::new();
		self.custom_canonicalized_with(&mut buffer)
	}
}

impl BorrowJsonLexical for JsonNumber {}

impl<T> BorrowJsonLexical for JsonNumberBuf<T> {}

const LOSSY_PARSE_FLOAT: lexical::ParseFloatOptions = lexical::ParseFloatOptions::builder()
	.lossy(true)
	.build_unchecked();

impl Deref for JsonNumber {
	type Target = str;

	#[inline(always)]
	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for JsonNumber {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Borrow<str> for JsonNumber {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<[u8]> for JsonNumber {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<'a> TryFrom<&'a str> for &'a JsonNumber {
	type Error = InvalidJsonNumber<&'a str>;

	#[inline(always)]
	fn try_from(s: &'a str) -> Result<&'a JsonNumber, InvalidJsonNumber<&'a str>> {
		JsonNumber::new(s)
	}
}

impl ToOwned for JsonNumber {
	type Owned = JsonNumberBuf;

	fn to_owned(&self) -> Self::Owned {
		self.to_custom_owned()
	}
}

impl fmt::Display for JsonNumber {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl fmt::Debug for JsonNumber {
	#[inline(always)]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

/// JSON number buffer.
#[derive(Clone, Hash)]
pub struct JsonNumberBuf<B = DefaultBuffer> {
	data: B,
}

impl<B> JsonNumberBuf<B> {
	/// Creates a new number buffer by parsing the given input `data` buffer.
	#[inline(always)]
	pub fn new(data: B) -> Result<Self, InvalidJsonNumber<B>>
	where
		B: AsRef<[u8]>,
	{
		match JsonNumber::new(&data) {
			Ok(_) => Ok(JsonNumberBuf { data }),
			Err(_) => Err(InvalidJsonNumber(data)),
		}
	}

	/// Creates a new number buffer from the given input `data` buffer.
	///
	/// ## Safety
	///
	/// The input `data` **must** hold a valid JSON number string.
	#[inline(always)]
	pub unsafe fn new_unchecked(data: B) -> Self {
		JsonNumberBuf { data }
	}

	/// Creates a number buffer from the given `number`.
	#[inline(always)]
	pub fn from_number(n: &JsonNumber) -> Self
	where
		B: FromIterator<u8>,
	{
		unsafe { JsonNumberBuf::new_unchecked(n.bytes().collect()) }
	}

	#[inline(always)]
	pub fn as_buffer(&self) -> &B {
		&self.data
	}

	#[inline(always)]
	pub fn into_buffer(self) -> B {
		self.data
	}
}

impl<B: JsonBytes> JsonNumberBuf<B> {
	/// Puts this number in canonical form according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalize(&mut self) {
		*self = self.custom_canonicalized();
	}

	/// Puts this number in canonical form according to
	/// [RFC8785](https://www.rfc-editor.org/rfc/rfc8785#name-serialization-of-numbers).
	#[cfg(feature = "canonicalize")]
	pub fn canonicalize_with(&mut self, buffer: &mut ryu_js::Buffer) {
		*self = self.custom_canonicalized_with(buffer)
	}
}

impl JsonNumberBuf<String> {
	#[inline(always)]
	pub fn into_string(self) -> String {
		self.data
	}

	#[inline(always)]
	pub fn into_bytes(self) -> Vec<u8> {
		self.data.into_bytes()
	}
}

impl<B: JsonBytes> JsonNumberBuf<B> {
	#[inline(always)]
	pub fn as_number(&self) -> &JsonNumber {
		unsafe { JsonNumber::new_unchecked(&self.data) }
	}
}

impl<T, U> PartialEq<JsonNumberBuf<U>> for JsonNumberBuf<T>
where
	T: JsonBytes,
	U: JsonBytes,
{
	fn eq(&self, other: &JsonNumberBuf<U>) -> bool {
		self.as_number() == other.as_number()
	}
}

impl<T: JsonBytes> Eq for JsonNumberBuf<T> {}

impl<T, U> PartialOrd<JsonNumberBuf<U>> for JsonNumberBuf<T>
where
	T: JsonBytes,
	U: JsonBytes,
{
	fn partial_cmp(&self, other: &JsonNumberBuf<U>) -> Option<std::cmp::Ordering> {
		self.as_number().partial_cmp(other.as_number())
	}
}

impl<T: JsonBytes> Ord for JsonNumberBuf<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.as_number().cmp(other.as_number())
	}
}

impl<B: JsonBytes> FromStr for JsonNumberBuf<B> {
	type Err = InvalidJsonNumber<B>;

	#[inline(always)]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(B::from_bytes(s.as_bytes()))
	}
}

impl<B: JsonBytes> Deref for JsonNumberBuf<B> {
	type Target = JsonNumber;

	#[inline(always)]
	fn deref(&self) -> &JsonNumber {
		self.as_number()
	}
}

impl<B: JsonBytes> AsRef<JsonNumber> for JsonNumberBuf<B> {
	#[inline(always)]
	fn as_ref(&self) -> &JsonNumber {
		self.as_number()
	}
}

impl<B: JsonBytes> Borrow<JsonNumber> for JsonNumberBuf<B> {
	#[inline(always)]
	fn borrow(&self) -> &JsonNumber {
		self.as_number()
	}
}

impl<B: JsonBytes> AsRef<str> for JsonNumberBuf<B> {
	#[inline(always)]
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<B: JsonBytes> Borrow<str> for JsonNumberBuf<B> {
	#[inline(always)]
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<B: JsonBytes> AsRef<[u8]> for JsonNumberBuf<B> {
	#[inline(always)]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<B: JsonBytes> Borrow<[u8]> for JsonNumberBuf<B> {
	#[inline(always)]
	fn borrow(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl<B: JsonBytes> fmt::Display for JsonNumberBuf<B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

impl<B: JsonBytes> fmt::Debug for JsonNumberBuf<B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.as_str().fmt(f)
	}
}

macro_rules! impl_from_int {
	($($ty:ty),*) => {
		$(
			impl<B: JsonBytes> From<$ty> for JsonNumberBuf<B> {
				#[inline(always)]
				fn from(i: $ty) -> Self {
					unsafe {
						Self::new_unchecked(B::from_vec(lexical::to_string(i).into_bytes()))
					}
				}
			}
		)*
	};
}

/// Float conversion error.
#[derive(Clone, Copy, Debug)]
pub enum TryFromFloatError {
	/// The float was Nan, which is not a JSON number.
	Nan,

	/// The float was not finite, and hence not a JSON number.
	Infinite,
}

const WRITE_FLOAT: lexical::WriteFloatOptions = lexical::WriteFloatOptions::builder()
	.trim_floats(true)
	.exponent(b'e')
	.build_unchecked();

macro_rules! impl_try_from_float {
	($($ty:ty),*) => {
		$(
			impl<B: JsonBytes> TryFrom<$ty> for JsonNumberBuf<B> {
				type Error = TryFromFloatError;

				#[inline(always)]
				fn try_from(f: $ty) -> Result<Self, Self::Error> {
					if f.is_finite() {
						Ok(unsafe {
							Self::new_unchecked(B::from_vec(lexical::to_string_with_options::<_, {lexical::format::JSON}>(f, &WRITE_FLOAT).into_bytes()))
						})
					} else if f.is_nan() {
						Err(TryFromFloatError::Nan)
					} else {
						Err(TryFromFloatError::Infinite)
					}
				}
			}
		)*
	};
}

impl_from_int!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);
impl_try_from_float!(f32, f64);

#[cfg(test)]
mod tests {
	use super::*;

	fn trimming_test(a: &str, b: &str) {
		let a = JsonNumber::new(a).unwrap();
		let b = JsonNumber::new(b).unwrap();
		assert_eq!(a.trimmed(), b)
	}

	#[test]
	fn trimming() {
		// Basic fractional trimming (existing cases).
		trimming_test("0", "0");
		trimming_test("0.0", "0");
		trimming_test("1.0", "1");
		trimming_test("1.1", "1.1");
		trimming_test("1.10000", "1.1");
		trimming_test("100.0", "100");
		trimming_test("100.1000", "100.1");

		// Negative sign removed for zero values.
		trimming_test("-0", "0");
		trimming_test("-0.0", "0");
		trimming_test("-0.000", "0");

		// Zero exponent removed.
		trimming_test("0e0", "0");
		trimming_test("1e0", "1");
		trimming_test("1.0e0", "1");
		trimming_test("1.5e0", "1.5");
		trimming_test("100.0e0", "100");

		// Zero exponent and negative sign both removed.
		trimming_test("-0e0", "0");
		trimming_test("-0.0e0", "0");
		trimming_test("-0.000e0", "0");

		// Non-zero exponent kept unchanged.
		trimming_test("1e21", "1e21");
		trimming_test("1.5e2", "1.5e2");
		trimming_test("-1e5", "-1e5");

		// Trailing zeros with non-zero exponent: fractional zeros cannot be
		// removed because the result would not be a contiguous subslice.
		trimming_test("1.0e2", "1.0e2");
		trimming_test("1.50e2", "1.50e2");
		trimming_test("10.0e2", "10.0e2");
	}

	macro_rules! positive_tests {
		{ $($id:ident: $input:literal),* } => {
			$(
				#[test]
				fn $id () {
					assert!(JsonNumber::new($input).is_ok())
				}
			)*
		};
	}

	macro_rules! negative_tests {
		{ $($id:ident: $input:literal),* } => {
			$(
				#[test]
				fn $id () {
					assert!(JsonNumber::new($input).is_err())
				}
			)*
		};
	}

	macro_rules! sign_tests {
		{ $($id:ident: $input:literal => $sign:ident),* } => {
			$(
				#[test]
				fn $id () {
					assert_eq!(JsonNumber::new($input).unwrap().sign(), Sign::$sign)
				}
			)*
		};
	}

	macro_rules! canonical_tests {
		{ $($id:ident: $input:literal => $output:literal),* } => {
			$(
				#[cfg(feature="canonicalize")]
				#[test]
				fn $id () {
					assert_eq!(JsonNumber::new($input).unwrap().canonicalized().as_number(), JsonNumber::new($output).unwrap())
				}
			)*
		};
	}

	positive_tests! {
		pos_01: "0",
		pos_02: "-0",
		pos_03: "123",
		pos_04: "1.23",
		pos_05: "-12.34",
		pos_06: "12.34e+56",
		pos_07: "12.34E-56",
		pos_08: "0.0000"
	}

	negative_tests! {
		neg_01: "",
		neg_02: "00",
		neg_03: "01",
		neg_04: "-00",
		neg_05: "-01",
		neg_06: "0.000e+-1",
		neg_07: "12.34E-56abc",
		neg_08: "1.",
		neg_09: "12.34e",
		neg_10: "12.34e+",
		neg_11: "12.34E-"
	}

	sign_tests! {
		sign_zero_01: "0" => Zero,
		sign_zero_02: "-0" => Zero,
		sign_zero_03: "0.0" => Zero,
		sign_zero_04: "0.0e12" => Zero,
		sign_zero_05: "-0.0E-12" => Zero,
		sign_zero_06: "-0.00000" => Zero
	}

	sign_tests! {
		sign_pos_01: "1" => Positive,
		sign_pos_02: "0.1" => Positive,
		sign_pos_03: "0.01e23" => Positive,
		sign_pos_04: "1.0E-23" => Positive,
		sign_pos_05: "0.00001" => Positive
	}

	sign_tests! {
		sign_neg_01: "-1" => Negative,
		sign_neg_02: "-0.1" => Negative,
		sign_neg_03: "-0.01e23" => Negative,
		sign_neg_04: "-1.0E-23" => Negative,
		sign_neg_05: "-0.00001" => Negative
	}

	canonical_tests! {
		canonical_01: "-0.0000" => "0",
		canonical_02: "0.00000000028" => "2.8e-10"
	}
}
