#!/usr/bin/env ruby

puts <<~EOS
	use std::path::Path;
	use std::fs;
	use std::fmt::Debug;
	use json_syntax::{Value, Parse, parse::Options};
	use decoded_char::DecodedChars;

	fn infallible<T>(t: T) -> Result<T, std::convert::Infallible> {
		Ok(t)
	}

	fn test<P: Clone + AsRef<Path> + Debug>(filename: P, options: Options) {
		let buffer = fs::read(filename.clone()).unwrap();
		let input = if options.accept_invalid_codepoints {
			String::from_utf8_lossy(&buffer)
		} else {
			std::borrow::Cow::Borrowed(std::str::from_utf8(&buffer).unwrap())
		};

		Value::parse_with(filename, input.decoded_chars().map(infallible), options).expect("parse error");
	}
EOS

def sanitize(name)
	name.gsub('UTF-8', 'utf8').gsub('U+', 'u').gsub('+', '_plus_').gsub('-', '_minus_').gsub(/[.#]/, '_').gsub(/_+/, '_').downcase
end

Dir['tests/inputs/*'].each do |path|
	if m = /tests\/inputs\/(y_.*)\.json/.match(path) then
		name = sanitize(m[1])
		puts <<~EOS

			#[test]
			fn #{name}() {
				test("#{path}", Options::strict())
			}
		EOS
	end

	if m = /tests\/inputs\/(n_.*)\.json/.match(path) then
		name = sanitize(m[1])
		puts <<~EOS

			#[test]
			#[should_panic]
			fn #{name}() {
				test("#{path}", Options::strict())
			}
		EOS
	end

	if m = /tests\/inputs\/(i_.*)\.json/.match(path) then
		name = sanitize(m[1])
		puts <<~EOS

			#[test]
			fn flexible_#{name}() {
				test("#{path}", Options::flexible())
			}

			#[test]
			#[should_panic]
			fn strict_#{name}() {
				test("#{path}", Options::strict())
			}
		EOS
	end
end