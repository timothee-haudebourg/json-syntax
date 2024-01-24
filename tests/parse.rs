use json_syntax::{parse::Options, Parse, Value};
use std::fmt::Debug;
use std::fs;
use std::path::Path;

fn test<P: Clone + AsRef<Path> + Debug>(filename: P, options: Options) {
	let buffer = fs::read(filename.clone()).unwrap();
	let input = if options.accept_invalid_codepoints {
		String::from_utf8_lossy(&buffer)
	} else {
		std::borrow::Cow::Borrowed(std::str::from_utf8(&buffer).unwrap())
	};

	Value::parse_str_with(&input, options).expect("parse error");
}

#[test]
fn flexible_i_object_key_lone_2nd_surrogate() {
	test(
		"tests/inputs/i_object_key_lone_2nd_surrogate.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_object_key_lone_2nd_surrogate() {
	test(
		"tests/inputs/i_object_key_lone_2nd_surrogate.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_1st_valid_surrogate_2nd_invalid() {
	test(
		"tests/inputs/i_string_1st_valid_surrogate_2nd_invalid.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_1st_valid_surrogate_2nd_invalid() {
	test(
		"tests/inputs/i_string_1st_valid_surrogate_2nd_invalid.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_utf8_invalid_sequence() {
	test(
		"tests/inputs/i_string_UTF-8_invalid_sequence.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_utf8_invalid_sequence() {
	test(
		"tests/inputs/i_string_UTF-8_invalid_sequence.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_utf8_surrogate_ud800() {
	test(
		"tests/inputs/i_string_UTF8_surrogate_U+D800.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_utf8_surrogate_ud800() {
	test(
		"tests/inputs/i_string_UTF8_surrogate_U+D800.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_incomplete_surrogate_and_escape_valid() {
	test(
		"tests/inputs/i_string_incomplete_surrogate_and_escape_valid.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_incomplete_surrogate_and_escape_valid() {
	test(
		"tests/inputs/i_string_incomplete_surrogate_and_escape_valid.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_incomplete_surrogate_pair() {
	test(
		"tests/inputs/i_string_incomplete_surrogate_pair.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_incomplete_surrogate_pair() {
	test(
		"tests/inputs/i_string_incomplete_surrogate_pair.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_incomplete_surrogates_escape_valid() {
	test(
		"tests/inputs/i_string_incomplete_surrogates_escape_valid.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_incomplete_surrogates_escape_valid() {
	test(
		"tests/inputs/i_string_incomplete_surrogates_escape_valid.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_invalid_lonely_surrogate() {
	test(
		"tests/inputs/i_string_invalid_lonely_surrogate.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_invalid_lonely_surrogate() {
	test(
		"tests/inputs/i_string_invalid_lonely_surrogate.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_invalid_surrogate() {
	test(
		"tests/inputs/i_string_invalid_surrogate.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_invalid_surrogate() {
	test(
		"tests/inputs/i_string_invalid_surrogate.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_invalid_utf_minus_8() {
	test(
		"tests/inputs/i_string_invalid_utf-8.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_invalid_utf_minus_8() {
	test(
		"tests/inputs/i_string_invalid_utf-8.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_inverted_surrogates_u1d11e() {
	test(
		"tests/inputs/i_string_inverted_surrogates_U+1D11E.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_inverted_surrogates_u1d11e() {
	test(
		"tests/inputs/i_string_inverted_surrogates_U+1D11E.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_iso_latin_1() {
	test(
		"tests/inputs/i_string_iso_latin_1.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_iso_latin_1() {
	test("tests/inputs/i_string_iso_latin_1.json", Options::strict())
}

#[test]
fn flexible_i_string_lone_second_surrogate() {
	test(
		"tests/inputs/i_string_lone_second_surrogate.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_lone_second_surrogate() {
	test(
		"tests/inputs/i_string_lone_second_surrogate.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_lone_utf8_continuation_byte() {
	test(
		"tests/inputs/i_string_lone_utf8_continuation_byte.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_lone_utf8_continuation_byte() {
	test(
		"tests/inputs/i_string_lone_utf8_continuation_byte.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_not_in_unicode_range() {
	test(
		"tests/inputs/i_string_not_in_unicode_range.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_not_in_unicode_range() {
	test(
		"tests/inputs/i_string_not_in_unicode_range.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_overlong_sequence_2_bytes() {
	test(
		"tests/inputs/i_string_overlong_sequence_2_bytes.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_overlong_sequence_2_bytes() {
	test(
		"tests/inputs/i_string_overlong_sequence_2_bytes.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_overlong_sequence_6_bytes() {
	test(
		"tests/inputs/i_string_overlong_sequence_6_bytes.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_overlong_sequence_6_bytes() {
	test(
		"tests/inputs/i_string_overlong_sequence_6_bytes.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_overlong_sequence_6_bytes_null() {
	test(
		"tests/inputs/i_string_overlong_sequence_6_bytes_null.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_overlong_sequence_6_bytes_null() {
	test(
		"tests/inputs/i_string_overlong_sequence_6_bytes_null.json",
		Options::strict(),
	)
}

#[test]
fn flexible_i_string_truncated_minus_utf_minus_8() {
	test(
		"tests/inputs/i_string_truncated-utf-8.json",
		Options::flexible(),
	)
}

#[test]
#[should_panic]
fn strict_i_string_truncated_minus_utf_minus_8() {
	test(
		"tests/inputs/i_string_truncated-utf-8.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_1_true_without_comma() {
	test(
		"tests/inputs/n_array_1_true_without_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_a_invalid_utf8() {
	test(
		"tests/inputs/n_array_a_invalid_utf8.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_colon_instead_of_comma() {
	test(
		"tests/inputs/n_array_colon_instead_of_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_comma_after_close() {
	test(
		"tests/inputs/n_array_comma_after_close.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_comma_and_number() {
	test(
		"tests/inputs/n_array_comma_and_number.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_double_comma() {
	test("tests/inputs/n_array_double_comma.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_double_extra_comma() {
	test(
		"tests/inputs/n_array_double_extra_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_extra_close() {
	test("tests/inputs/n_array_extra_close.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_extra_comma() {
	test("tests/inputs/n_array_extra_comma.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_incomplete() {
	test("tests/inputs/n_array_incomplete.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_incomplete_invalid_value() {
	test(
		"tests/inputs/n_array_incomplete_invalid_value.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_inner_array_no_comma() {
	test(
		"tests/inputs/n_array_inner_array_no_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_invalid_utf8() {
	test("tests/inputs/n_array_invalid_utf8.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_items_separated_by_semicolon() {
	test(
		"tests/inputs/n_array_items_separated_by_semicolon.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_just_comma() {
	test("tests/inputs/n_array_just_comma.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_just_minus() {
	test("tests/inputs/n_array_just_minus.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_missing_value() {
	test("tests/inputs/n_array_missing_value.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_newlines_unclosed() {
	test(
		"tests/inputs/n_array_newlines_unclosed.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_number_and_comma() {
	test(
		"tests/inputs/n_array_number_and_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_number_and_several_commas() {
	test(
		"tests/inputs/n_array_number_and_several_commas.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_spaces_vertical_tab_formfeed() {
	test(
		"tests/inputs/n_array_spaces_vertical_tab_formfeed.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_star_inside() {
	test("tests/inputs/n_array_star_inside.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_unclosed() {
	test("tests/inputs/n_array_unclosed.json", Options::strict())
}

#[test]
#[should_panic]
fn n_array_unclosed_trailing_comma() {
	test(
		"tests/inputs/n_array_unclosed_trailing_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_unclosed_with_new_lines() {
	test(
		"tests/inputs/n_array_unclosed_with_new_lines.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_array_unclosed_with_object_inside() {
	test(
		"tests/inputs/n_array_unclosed_with_object_inside.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_incomplete_false() {
	test("tests/inputs/n_incomplete_false.json", Options::strict())
}

#[test]
#[should_panic]
fn n_incomplete_null() {
	test("tests/inputs/n_incomplete_null.json", Options::strict())
}

#[test]
#[should_panic]
fn n_incomplete_true() {
	test("tests/inputs/n_incomplete_true.json", Options::strict())
}

#[test]
#[should_panic]
fn n_multidigit_number_then_00() {
	test(
		"tests/inputs/n_multidigit_number_then_00.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_plus_plus_() {
	test("tests/inputs/n_number_++.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_plus_1() {
	test("tests/inputs/n_number_+1.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_plus_inf() {
	test("tests/inputs/n_number_+Inf.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_minus_01() {
	test("tests/inputs/n_number_-01.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_minus_1_0_() {
	test("tests/inputs/n_number_-1.0..json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_minus_2_() {
	test("tests/inputs/n_number_-2..json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_minus_nan() {
	test("tests/inputs/n_number_-NaN.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_minus_1() {
	test("tests/inputs/n_number_.-1.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_2e_minus_3() {
	test("tests/inputs/n_number_.2e-3.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0_1_2() {
	test("tests/inputs/n_number_0.1.2.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0_3e_plus_() {
	test("tests/inputs/n_number_0.3e+.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0_3e() {
	test("tests/inputs/n_number_0.3e.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0_e1() {
	test("tests/inputs/n_number_0.e1.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0_capital_e_plus_() {
	test("tests/inputs/n_number_0_capital_E+.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0_capital_e() {
	test("tests/inputs/n_number_0_capital_E.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0e_plus_() {
	test("tests/inputs/n_number_0e+.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_0e() {
	test("tests/inputs/n_number_0e.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_1_0e_plus_() {
	test("tests/inputs/n_number_1.0e+.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_1_0e_minus_() {
	test("tests/inputs/n_number_1.0e-.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_1_0e() {
	test("tests/inputs/n_number_1.0e.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_1_000() {
	test("tests/inputs/n_number_1_000.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_1ee2() {
	test("tests/inputs/n_number_1eE2.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_2_e_plus_3() {
	test("tests/inputs/n_number_2.e+3.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_2_e_minus_3() {
	test("tests/inputs/n_number_2.e-3.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_2_e3() {
	test("tests/inputs/n_number_2.e3.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_9_e_plus_() {
	test("tests/inputs/n_number_9.e+.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_inf() {
	test("tests/inputs/n_number_Inf.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_nan() {
	test("tests/inputs/n_number_NaN.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_uff11_fullwidth_digit_one() {
	test(
		"tests/inputs/n_number_U+FF11_fullwidth_digit_one.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_expression() {
	test("tests/inputs/n_number_expression.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_hex_1_digit() {
	test("tests/inputs/n_number_hex_1_digit.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_hex_2_digits() {
	test("tests/inputs/n_number_hex_2_digits.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_infinity() {
	test("tests/inputs/n_number_infinity.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_invalid_plus_minus_() {
	test("tests/inputs/n_number_invalid+-.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_invalid_minus_negative_minus_real() {
	test(
		"tests/inputs/n_number_invalid-negative-real.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_invalid_minus_utf_minus_8_minus_in_minus_bigger_minus_int() {
	test(
		"tests/inputs/n_number_invalid-utf-8-in-bigger-int.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_invalid_minus_utf_minus_8_minus_in_minus_exponent() {
	test(
		"tests/inputs/n_number_invalid-utf-8-in-exponent.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_invalid_minus_utf_minus_8_minus_in_minus_int() {
	test(
		"tests/inputs/n_number_invalid-utf-8-in-int.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_minus_infinity() {
	test(
		"tests/inputs/n_number_minus_infinity.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_minus_sign_with_trailing_garbage() {
	test(
		"tests/inputs/n_number_minus_sign_with_trailing_garbage.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_minus_space_1() {
	test(
		"tests/inputs/n_number_minus_space_1.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_neg_int_starting_with_zero() {
	test(
		"tests/inputs/n_number_neg_int_starting_with_zero.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_neg_real_without_int_part() {
	test(
		"tests/inputs/n_number_neg_real_without_int_part.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_neg_with_garbage_at_end() {
	test(
		"tests/inputs/n_number_neg_with_garbage_at_end.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_real_garbage_after_e() {
	test(
		"tests/inputs/n_number_real_garbage_after_e.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_real_with_invalid_utf8_after_e() {
	test(
		"tests/inputs/n_number_real_with_invalid_utf8_after_e.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_real_without_fractional_part() {
	test(
		"tests/inputs/n_number_real_without_fractional_part.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_starting_with_dot() {
	test(
		"tests/inputs/n_number_starting_with_dot.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_with_alpha() {
	test("tests/inputs/n_number_with_alpha.json", Options::strict())
}

#[test]
#[should_panic]
fn n_number_with_alpha_char() {
	test(
		"tests/inputs/n_number_with_alpha_char.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_number_with_leading_zero() {
	test(
		"tests/inputs/n_number_with_leading_zero.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_bad_value() {
	test("tests/inputs/n_object_bad_value.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_bracket_key() {
	test("tests/inputs/n_object_bracket_key.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_comma_instead_of_colon() {
	test(
		"tests/inputs/n_object_comma_instead_of_colon.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_double_colon() {
	test("tests/inputs/n_object_double_colon.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_emoji() {
	test("tests/inputs/n_object_emoji.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_garbage_at_end() {
	test(
		"tests/inputs/n_object_garbage_at_end.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_key_with_single_quotes() {
	test(
		"tests/inputs/n_object_key_with_single_quotes.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_lone_continuation_byte_in_key_and_trailing_comma() {
	test(
		"tests/inputs/n_object_lone_continuation_byte_in_key_and_trailing_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_missing_colon() {
	test(
		"tests/inputs/n_object_missing_colon.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_missing_key() {
	test("tests/inputs/n_object_missing_key.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_missing_semicolon() {
	test(
		"tests/inputs/n_object_missing_semicolon.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_missing_value() {
	test(
		"tests/inputs/n_object_missing_value.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_no_minus_colon() {
	test("tests/inputs/n_object_no-colon.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_non_string_key() {
	test(
		"tests/inputs/n_object_non_string_key.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_non_string_key_but_huge_number_instead() {
	test(
		"tests/inputs/n_object_non_string_key_but_huge_number_instead.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_repeated_null_null() {
	test(
		"tests/inputs/n_object_repeated_null_null.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_several_trailing_commas() {
	test(
		"tests/inputs/n_object_several_trailing_commas.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_single_quote() {
	test("tests/inputs/n_object_single_quote.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_trailing_comma() {
	test(
		"tests/inputs/n_object_trailing_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_trailing_comment() {
	test(
		"tests/inputs/n_object_trailing_comment.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_trailing_comment_open() {
	test(
		"tests/inputs/n_object_trailing_comment_open.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_trailing_comment_slash_open() {
	test(
		"tests/inputs/n_object_trailing_comment_slash_open.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_trailing_comment_slash_open_incomplete() {
	test(
		"tests/inputs/n_object_trailing_comment_slash_open_incomplete.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_two_commas_in_a_row() {
	test(
		"tests/inputs/n_object_two_commas_in_a_row.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_unquoted_key() {
	test("tests/inputs/n_object_unquoted_key.json", Options::strict())
}

#[test]
#[should_panic]
fn n_object_unterminated_minus_value() {
	test(
		"tests/inputs/n_object_unterminated-value.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_with_single_string() {
	test(
		"tests/inputs/n_object_with_single_string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_object_with_trailing_garbage() {
	test(
		"tests/inputs/n_object_with_trailing_garbage.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_single_space() {
	test("tests/inputs/n_single_space.json", Options::strict())
}

#[test]
#[should_panic]
fn n_string_1_surrogate_then_escape() {
	test(
		"tests/inputs/n_string_1_surrogate_then_escape.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_1_surrogate_then_escape_u() {
	test(
		"tests/inputs/n_string_1_surrogate_then_escape_u.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_1_surrogate_then_escape_u1() {
	test(
		"tests/inputs/n_string_1_surrogate_then_escape_u1.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_1_surrogate_then_escape_u1x() {
	test(
		"tests/inputs/n_string_1_surrogate_then_escape_u1x.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_accentuated_char_no_quotes() {
	test(
		"tests/inputs/n_string_accentuated_char_no_quotes.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_backslash_00() {
	test("tests/inputs/n_string_backslash_00.json", Options::strict())
}

#[test]
#[should_panic]
fn n_string_escape_x() {
	test("tests/inputs/n_string_escape_x.json", Options::strict())
}

#[test]
#[should_panic]
fn n_string_escaped_backslash_bad() {
	test(
		"tests/inputs/n_string_escaped_backslash_bad.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_escaped_ctrl_char_tab() {
	test(
		"tests/inputs/n_string_escaped_ctrl_char_tab.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_escaped_emoji() {
	test(
		"tests/inputs/n_string_escaped_emoji.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_incomplete_escape() {
	test(
		"tests/inputs/n_string_incomplete_escape.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_incomplete_escaped_character() {
	test(
		"tests/inputs/n_string_incomplete_escaped_character.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_incomplete_surrogate() {
	test(
		"tests/inputs/n_string_incomplete_surrogate.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_incomplete_surrogate_escape_invalid() {
	test(
		"tests/inputs/n_string_incomplete_surrogate_escape_invalid.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_invalid_minus_utf_minus_8_minus_in_minus_escape() {
	test(
		"tests/inputs/n_string_invalid-utf-8-in-escape.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_invalid_backslash_esc() {
	test(
		"tests/inputs/n_string_invalid_backslash_esc.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_invalid_unicode_escape() {
	test(
		"tests/inputs/n_string_invalid_unicode_escape.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_invalid_utf8_after_escape() {
	test(
		"tests/inputs/n_string_invalid_utf8_after_escape.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_leading_uescaped_thinspace() {
	test(
		"tests/inputs/n_string_leading_uescaped_thinspace.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_no_quotes_with_bad_escape() {
	test(
		"tests/inputs/n_string_no_quotes_with_bad_escape.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_single_doublequote() {
	test(
		"tests/inputs/n_string_single_doublequote.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_single_quote() {
	test("tests/inputs/n_string_single_quote.json", Options::strict())
}

#[test]
#[should_panic]
fn n_string_single_string_no_double_quotes() {
	test(
		"tests/inputs/n_string_single_string_no_double_quotes.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_start_escape_unclosed() {
	test(
		"tests/inputs/n_string_start_escape_unclosed.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_unescaped_ctrl_char() {
	test(
		"tests/inputs/n_string_unescaped_ctrl_char.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_unescaped_newline() {
	test(
		"tests/inputs/n_string_unescaped_newline.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_unescaped_tab() {
	test(
		"tests/inputs/n_string_unescaped_tab.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_unicode_capitalu() {
	test(
		"tests/inputs/n_string_unicode_CapitalU.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_string_with_trailing_garbage() {
	test(
		"tests/inputs/n_string_with_trailing_garbage.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_100000_opening_arrays() {
	test(
		"tests/inputs/n_structure_100000_opening_arrays.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_u2060_word_joined() {
	test(
		"tests/inputs/n_structure_U+2060_word_joined.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_utf8_bom_no_data() {
	test(
		"tests/inputs/n_structure_UTF8_BOM_no_data.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_angle_bracket_() {
	test(
		"tests/inputs/n_structure_angle_bracket_..json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_angle_bracket_null() {
	test(
		"tests/inputs/n_structure_angle_bracket_null.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_array_trailing_garbage() {
	test(
		"tests/inputs/n_structure_array_trailing_garbage.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_array_with_extra_array_close() {
	test(
		"tests/inputs/n_structure_array_with_extra_array_close.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_array_with_unclosed_string() {
	test(
		"tests/inputs/n_structure_array_with_unclosed_string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_ascii_minus_unicode_minus_identifier() {
	test(
		"tests/inputs/n_structure_ascii-unicode-identifier.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_capitalized_true() {
	test(
		"tests/inputs/n_structure_capitalized_True.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_close_unopened_array() {
	test(
		"tests/inputs/n_structure_close_unopened_array.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_comma_instead_of_closing_brace() {
	test(
		"tests/inputs/n_structure_comma_instead_of_closing_brace.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_double_array() {
	test(
		"tests/inputs/n_structure_double_array.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_end_array() {
	test("tests/inputs/n_structure_end_array.json", Options::strict())
}

#[test]
#[should_panic]
fn n_structure_incomplete_utf8_bom() {
	test(
		"tests/inputs/n_structure_incomplete_UTF8_BOM.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_lone_minus_invalid_minus_utf_minus_8() {
	test(
		"tests/inputs/n_structure_lone-invalid-utf-8.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_lone_minus_open_minus_bracket() {
	test(
		"tests/inputs/n_structure_lone-open-bracket.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_no_data() {
	test("tests/inputs/n_structure_no_data.json", Options::strict())
}

#[test]
#[should_panic]
fn n_structure_null_minus_byte_minus_outside_minus_string() {
	test(
		"tests/inputs/n_structure_null-byte-outside-string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_number_with_trailing_garbage() {
	test(
		"tests/inputs/n_structure_number_with_trailing_garbage.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_object_followed_by_closing_object() {
	test(
		"tests/inputs/n_structure_object_followed_by_closing_object.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_object_unclosed_no_value() {
	test(
		"tests/inputs/n_structure_object_unclosed_no_value.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_object_with_comment() {
	test(
		"tests/inputs/n_structure_object_with_comment.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_object_with_trailing_garbage() {
	test(
		"tests/inputs/n_structure_object_with_trailing_garbage.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_array_apostrophe() {
	test(
		"tests/inputs/n_structure_open_array_apostrophe.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_array_comma() {
	test(
		"tests/inputs/n_structure_open_array_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_array_object() {
	test(
		"tests/inputs/n_structure_open_array_object.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_array_open_object() {
	test(
		"tests/inputs/n_structure_open_array_open_object.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_array_open_string() {
	test(
		"tests/inputs/n_structure_open_array_open_string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_array_string() {
	test(
		"tests/inputs/n_structure_open_array_string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_object() {
	test(
		"tests/inputs/n_structure_open_object.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_object_close_array() {
	test(
		"tests/inputs/n_structure_open_object_close_array.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_object_comma() {
	test(
		"tests/inputs/n_structure_open_object_comma.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_object_open_array() {
	test(
		"tests/inputs/n_structure_open_object_open_array.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_object_open_string() {
	test(
		"tests/inputs/n_structure_open_object_open_string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_object_string_with_apostrophes() {
	test(
		"tests/inputs/n_structure_open_object_string_with_apostrophes.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_open_open() {
	test("tests/inputs/n_structure_open_open.json", Options::strict())
}

#[test]
#[should_panic]
fn n_structure_single_eacute() {
	test(
		"tests/inputs/n_structure_single_eacute.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_single_star() {
	test(
		"tests/inputs/n_structure_single_star.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_trailing_() {
	test(
		"tests/inputs/n_structure_trailing_#.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_uescaped_lf_before_string() {
	test(
		"tests/inputs/n_structure_uescaped_LF_before_string.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_unclosed_array() {
	test(
		"tests/inputs/n_structure_unclosed_array.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_unclosed_array_partial_null() {
	test(
		"tests/inputs/n_structure_unclosed_array_partial_null.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_unclosed_array_unfinished_false() {
	test(
		"tests/inputs/n_structure_unclosed_array_unfinished_false.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_unclosed_array_unfinished_true() {
	test(
		"tests/inputs/n_structure_unclosed_array_unfinished_true.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_unclosed_object() {
	test(
		"tests/inputs/n_structure_unclosed_object.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_unicode_minus_identifier() {
	test(
		"tests/inputs/n_structure_unicode-identifier.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_whitespace_u2060_word_joiner() {
	test(
		"tests/inputs/n_structure_whitespace_U+2060_word_joiner.json",
		Options::strict(),
	)
}

#[test]
#[should_panic]
fn n_structure_whitespace_formfeed() {
	test(
		"tests/inputs/n_structure_whitespace_formfeed.json",
		Options::strict(),
	)
}

#[test]
fn y_array_arrayswithspaces() {
	test(
		"tests/inputs/y_array_arraysWithSpaces.json",
		Options::strict(),
	)
}

#[test]
fn y_array_empty_minus_string() {
	test("tests/inputs/y_array_empty-string.json", Options::strict())
}

#[test]
fn y_array_empty() {
	test("tests/inputs/y_array_empty.json", Options::strict())
}

#[test]
fn y_array_ending_with_newline() {
	test(
		"tests/inputs/y_array_ending_with_newline.json",
		Options::strict(),
	)
}

#[test]
fn y_array_false() {
	test("tests/inputs/y_array_false.json", Options::strict())
}

#[test]
fn y_array_heterogeneous() {
	test("tests/inputs/y_array_heterogeneous.json", Options::strict())
}

#[test]
fn y_array_null() {
	test("tests/inputs/y_array_null.json", Options::strict())
}

#[test]
fn y_array_with_1_and_newline() {
	test(
		"tests/inputs/y_array_with_1_and_newline.json",
		Options::strict(),
	)
}

#[test]
fn y_array_with_leading_space() {
	test(
		"tests/inputs/y_array_with_leading_space.json",
		Options::strict(),
	)
}

#[test]
fn y_array_with_several_null() {
	test(
		"tests/inputs/y_array_with_several_null.json",
		Options::strict(),
	)
}

#[test]
fn y_array_with_trailing_space() {
	test(
		"tests/inputs/y_array_with_trailing_space.json",
		Options::strict(),
	)
}

#[test]
fn y_number() {
	test("tests/inputs/y_number.json", Options::strict())
}

#[test]
fn y_number_0e_plus_1() {
	test("tests/inputs/y_number_0e+1.json", Options::strict())
}

#[test]
fn y_number_0e1() {
	test("tests/inputs/y_number_0e1.json", Options::strict())
}

#[test]
fn y_number_after_space() {
	test("tests/inputs/y_number_after_space.json", Options::strict())
}

#[test]
fn y_number_double_close_to_zero() {
	test(
		"tests/inputs/y_number_double_close_to_zero.json",
		Options::strict(),
	)
}

#[test]
fn y_number_double_huge_neg_exp() {
	test(
		"tests/inputs/y_number_double_huge_neg_exp.json",
		Options::strict(),
	)
}

#[test]
fn y_number_huge_exp() {
	test("tests/inputs/y_number_huge_exp.json", Options::strict())
}

#[test]
fn y_number_int_with_exp() {
	test("tests/inputs/y_number_int_with_exp.json", Options::strict())
}

#[test]
fn y_number_minus_zero() {
	test("tests/inputs/y_number_minus_zero.json", Options::strict())
}

#[test]
fn y_number_neg_int_huge_exp() {
	test(
		"tests/inputs/y_number_neg_int_huge_exp.json",
		Options::strict(),
	)
}

#[test]
fn y_number_negative_int() {
	test("tests/inputs/y_number_negative_int.json", Options::strict())
}

#[test]
fn y_number_negative_one() {
	test("tests/inputs/y_number_negative_one.json", Options::strict())
}

#[test]
fn y_number_negative_zero() {
	test(
		"tests/inputs/y_number_negative_zero.json",
		Options::strict(),
	)
}

#[test]
fn y_number_pos_double_huge_exp() {
	test(
		"tests/inputs/y_number_pos_double_huge_exp.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_capital_e() {
	test(
		"tests/inputs/y_number_real_capital_e.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_capital_e_neg_exp() {
	test(
		"tests/inputs/y_number_real_capital_e_neg_exp.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_capital_e_pos_exp() {
	test(
		"tests/inputs/y_number_real_capital_e_pos_exp.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_exponent() {
	test(
		"tests/inputs/y_number_real_exponent.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_fraction_exponent() {
	test(
		"tests/inputs/y_number_real_fraction_exponent.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_neg_exp() {
	test("tests/inputs/y_number_real_neg_exp.json", Options::strict())
}

#[test]
fn y_number_real_neg_overflow() {
	test(
		"tests/inputs/y_number_real_neg_overflow.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_pos_exponent() {
	test(
		"tests/inputs/y_number_real_pos_exponent.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_pos_overflow() {
	test(
		"tests/inputs/y_number_real_pos_overflow.json",
		Options::strict(),
	)
}

#[test]
fn y_number_real_underflow() {
	test(
		"tests/inputs/y_number_real_underflow.json",
		Options::strict(),
	)
}

#[test]
fn y_number_simple_int() {
	test("tests/inputs/y_number_simple_int.json", Options::strict())
}

#[test]
fn y_number_simple_real() {
	test("tests/inputs/y_number_simple_real.json", Options::strict())
}

#[test]
fn y_number_too_big_neg_int() {
	test(
		"tests/inputs/y_number_too_big_neg_int.json",
		Options::strict(),
	)
}

#[test]
fn y_number_too_big_pos_int() {
	test(
		"tests/inputs/y_number_too_big_pos_int.json",
		Options::strict(),
	)
}

#[test]
fn y_number_very_big_negative_int() {
	test(
		"tests/inputs/y_number_very_big_negative_int.json",
		Options::strict(),
	)
}

#[test]
fn y_object() {
	test("tests/inputs/y_object.json", Options::strict())
}

#[test]
fn y_object_basic() {
	test("tests/inputs/y_object_basic.json", Options::strict())
}

#[test]
fn y_object_duplicated_key() {
	test(
		"tests/inputs/y_object_duplicated_key.json",
		Options::strict(),
	)
}

#[test]
fn y_object_duplicated_key_and_value() {
	test(
		"tests/inputs/y_object_duplicated_key_and_value.json",
		Options::strict(),
	)
}

#[test]
fn y_object_empty() {
	test("tests/inputs/y_object_empty.json", Options::strict())
}

#[test]
fn y_object_empty_key() {
	test("tests/inputs/y_object_empty_key.json", Options::strict())
}

#[test]
fn y_object_escaped_null_in_key() {
	test(
		"tests/inputs/y_object_escaped_null_in_key.json",
		Options::strict(),
	)
}

#[test]
fn y_object_extreme_numbers() {
	test(
		"tests/inputs/y_object_extreme_numbers.json",
		Options::strict(),
	)
}

#[test]
fn y_object_long_strings() {
	test("tests/inputs/y_object_long_strings.json", Options::strict())
}

#[test]
fn y_object_simple() {
	test("tests/inputs/y_object_simple.json", Options::strict())
}

#[test]
fn y_object_string_unicode() {
	test(
		"tests/inputs/y_object_string_unicode.json",
		Options::strict(),
	)
}

#[test]
fn y_object_with_newlines() {
	test(
		"tests/inputs/y_object_with_newlines.json",
		Options::strict(),
	)
}

#[test]
fn y_string_1_2_3_bytes_utf8_sequences() {
	test(
		"tests/inputs/y_string_1_2_3_bytes_UTF-8_sequences.json",
		Options::strict(),
	)
}

#[test]
fn y_string_accepted_surrogate_pair() {
	test(
		"tests/inputs/y_string_accepted_surrogate_pair.json",
		Options::strict(),
	)
}

#[test]
fn y_string_accepted_surrogate_pairs() {
	test(
		"tests/inputs/y_string_accepted_surrogate_pairs.json",
		Options::strict(),
	)
}

#[test]
fn y_string_allowed_escapes() {
	test(
		"tests/inputs/y_string_allowed_escapes.json",
		Options::strict(),
	)
}

#[test]
fn y_string_backslash_and_u_escaped_zero() {
	test(
		"tests/inputs/y_string_backslash_and_u_escaped_zero.json",
		Options::strict(),
	)
}

#[test]
fn y_string_backslash_doublequotes() {
	test(
		"tests/inputs/y_string_backslash_doublequotes.json",
		Options::strict(),
	)
}

#[test]
fn y_string_comments() {
	test("tests/inputs/y_string_comments.json", Options::strict())
}

#[test]
fn y_string_double_escape_a() {
	test(
		"tests/inputs/y_string_double_escape_a.json",
		Options::strict(),
	)
}

#[test]
fn y_string_double_escape_n() {
	test(
		"tests/inputs/y_string_double_escape_n.json",
		Options::strict(),
	)
}

#[test]
fn y_string_escaped_control_character() {
	test(
		"tests/inputs/y_string_escaped_control_character.json",
		Options::strict(),
	)
}

#[test]
fn y_string_escaped_noncharacter() {
	test(
		"tests/inputs/y_string_escaped_noncharacter.json",
		Options::strict(),
	)
}

#[test]
fn y_string_in_array() {
	test("tests/inputs/y_string_in_array.json", Options::strict())
}

#[test]
fn y_string_in_array_with_leading_space() {
	test(
		"tests/inputs/y_string_in_array_with_leading_space.json",
		Options::strict(),
	)
}

#[test]
fn y_string_last_surrogates_1_and_2() {
	test(
		"tests/inputs/y_string_last_surrogates_1_and_2.json",
		Options::strict(),
	)
}

#[test]
fn y_string_nbsp_uescaped() {
	test(
		"tests/inputs/y_string_nbsp_uescaped.json",
		Options::strict(),
	)
}

#[test]
fn y_string_noncharacterinutf8_u10ffff() {
	test(
		"tests/inputs/y_string_nonCharacterInUTF-8_U+10FFFF.json",
		Options::strict(),
	)
}

#[test]
fn y_string_noncharacterinutf8_uffff() {
	test(
		"tests/inputs/y_string_nonCharacterInUTF-8_U+FFFF.json",
		Options::strict(),
	)
}

#[test]
fn y_string_null_escape() {
	test("tests/inputs/y_string_null_escape.json", Options::strict())
}

#[test]
fn y_string_one_minus_byte_minus_utf_minus_8() {
	test(
		"tests/inputs/y_string_one-byte-utf-8.json",
		Options::strict(),
	)
}

#[test]
fn y_string_pi() {
	test("tests/inputs/y_string_pi.json", Options::strict())
}

#[test]
fn y_string_reservedcharacterinutf8_u1bfff() {
	test(
		"tests/inputs/y_string_reservedCharacterInUTF-8_U+1BFFF.json",
		Options::strict(),
	)
}

#[test]
fn y_string_simple_ascii() {
	test("tests/inputs/y_string_simple_ascii.json", Options::strict())
}

#[test]
fn y_string_space() {
	test("tests/inputs/y_string_space.json", Options::strict())
}

#[test]
fn y_string_surrogates_u1d11e_musical_symbol_g_clef() {
	test(
		"tests/inputs/y_string_surrogates_U+1D11E_MUSICAL_SYMBOL_G_CLEF.json",
		Options::strict(),
	)
}

#[test]
fn y_string_three_minus_byte_minus_utf_minus_8() {
	test(
		"tests/inputs/y_string_three-byte-utf-8.json",
		Options::strict(),
	)
}

#[test]
fn y_string_two_minus_byte_minus_utf_minus_8() {
	test(
		"tests/inputs/y_string_two-byte-utf-8.json",
		Options::strict(),
	)
}

#[test]
fn y_string_u_plus_2028_line_sep() {
	test(
		"tests/inputs/y_string_u+2028_line_sep.json",
		Options::strict(),
	)
}

#[test]
fn y_string_u_plus_2029_par_sep() {
	test(
		"tests/inputs/y_string_u+2029_par_sep.json",
		Options::strict(),
	)
}

#[test]
fn y_string_uescape() {
	test("tests/inputs/y_string_uEscape.json", Options::strict())
}

#[test]
fn y_string_uescaped_newline() {
	test(
		"tests/inputs/y_string_uescaped_newline.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unescaped_char_delete() {
	test(
		"tests/inputs/y_string_unescaped_char_delete.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode() {
	test("tests/inputs/y_string_unicode.json", Options::strict())
}

#[test]
fn y_string_unicodeescapedbackslash() {
	test(
		"tests/inputs/y_string_unicodeEscapedBackslash.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_2() {
	test("tests/inputs/y_string_unicode_2.json", Options::strict())
}

#[test]
fn y_string_unicode_u10fffe_nonchar() {
	test(
		"tests/inputs/y_string_unicode_U+10FFFE_nonchar.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_u1fffe_nonchar() {
	test(
		"tests/inputs/y_string_unicode_U+1FFFE_nonchar.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_u200b_zero_width_space() {
	test(
		"tests/inputs/y_string_unicode_U+200B_ZERO_WIDTH_SPACE.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_u2064_invisible_plus() {
	test(
		"tests/inputs/y_string_unicode_U+2064_invisible_plus.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_ufdd0_nonchar() {
	test(
		"tests/inputs/y_string_unicode_U+FDD0_nonchar.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_ufffe_nonchar() {
	test(
		"tests/inputs/y_string_unicode_U+FFFE_nonchar.json",
		Options::strict(),
	)
}

#[test]
fn y_string_unicode_escaped_double_quote() {
	test(
		"tests/inputs/y_string_unicode_escaped_double_quote.json",
		Options::strict(),
	)
}

#[test]
fn y_string_utf8() {
	test("tests/inputs/y_string_utf8.json", Options::strict())
}

#[test]
fn y_string_with_del_character() {
	test(
		"tests/inputs/y_string_with_del_character.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_500_nested_arrays() {
	test(
		"tests/inputs/y_structure_500_nested_arrays.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_lonely_false() {
	test(
		"tests/inputs/y_structure_lonely_false.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_lonely_int() {
	test(
		"tests/inputs/y_structure_lonely_int.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_lonely_negative_real() {
	test(
		"tests/inputs/y_structure_lonely_negative_real.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_lonely_null() {
	test(
		"tests/inputs/y_structure_lonely_null.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_lonely_string() {
	test(
		"tests/inputs/y_structure_lonely_string.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_lonely_true() {
	test(
		"tests/inputs/y_structure_lonely_true.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_string_empty() {
	test(
		"tests/inputs/y_structure_string_empty.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_trailing_newline() {
	test(
		"tests/inputs/y_structure_trailing_newline.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_true_in_array() {
	test(
		"tests/inputs/y_structure_true_in_array.json",
		Options::strict(),
	)
}

#[test]
fn y_structure_whitespace_array() {
	test(
		"tests/inputs/y_structure_whitespace_array.json",
		Options::strict(),
	)
}

#[test]
fn y_issue_1() {
	test("tests/inputs/y_issue_1.json", Options::strict())
}
