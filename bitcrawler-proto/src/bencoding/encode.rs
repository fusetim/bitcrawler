use std::{borrow::Cow, io::Write};

use super::{BencodeString, BencodeValue};

/// Write a byte string (as bencode) to the output.
/// 
/// # Arguments
/// 
/// * `input` - The string to write.
/// * `output` - The output stream to write to.
pub fn write_string<'a, T, W>(input: T, mut output: W) 
where
    T: Into<Cow<'a, BencodeString>>,
    W: std::io::Write,
{
    let input = input.into();
    let length = input.0.len();
    let length_str = length.to_string();
    output.write_all(length_str.as_bytes()).unwrap();
    output.write_all(b":").unwrap();
    output.write_all(&input.0).unwrap();
}

/// Write an integer (as bencode) to the output.
/// 
/// # Arguments
/// 
/// * `input` - The integer to write.
/// * `output` - The output stream to write to.
/// 
/// # Notes
/// 
/// * Supported integer types are `i8`, `i16`, `i32`, `i64`, `u64` and `i128`.
/// 
pub fn write_integer<T, W>(input: T, mut output: W) 
where
    T: Into<i128>,
    W: std::io::Write,
{
    let input: i128 = input.into();
    output.write_all(b"i").unwrap();
    output.write_all(input.to_string().as_bytes()).unwrap();
    output.write_all(b"e").unwrap();
}

/// Encodes a string into a bencoded string.
///
/// # Arguments
///
/// * `input` - The string to encode.
///
/// # Returns
///
/// A bencoded byte string.
///
/// # What is the string encoded as?
///
/// The string is encoded as follows:
/// ```<length>:<string>```
///
/// Where:
/// * `<length>` is the length of the string.
/// * `<string>` is the string itself.
///
/// *Reference:* [BEP 3](https://www.bittorrent.org/beps/bep_0003.html)
pub fn encode_string<T: Into<BencodeString>>(input: T) -> Vec<u8> {
    let input: BencodeString = input.into();
    let length = input.0.len();
    let length_str = length.to_string();
    let mut result = Vec::new();
    result.extend_from_slice(length_str.as_bytes());
    result.push(b':');
    result.extend_from_slice(&input.0);
    result
}

enum EncodingToken {
    Value(BencodeValue),
    ListStart,
    ListEnd,
    DictStart,
    DictEntry(BencodeString),
    DictEnd,
}

/// Encodes a Bencoded value into a bencoded string.
///
/// # Arguments
///
/// * `input` - The Bencoded value to encode.
///
/// # Returns
///
/// A bencoded string.
pub fn encode(input: &BencodeValue) -> Vec<u8> {
    let mut token_stack = Vec::new();
    let mut value_stack = Vec::new();
    match input {
        BencodeValue::ByteString(s) => {
            write_string(s, &mut token_stack);
        }
        BencodeValue::Integer(i) => {
            write_integer(*i, &mut token_stack);
        }
        BencodeValue::List(_) => {
            value_stack.push(EncodingToken::Value(input.clone()));
        }
        BencodeValue::Dict(_) => {
            value_stack.push(EncodingToken::Value(input.clone()));
        }
    }
    while let Some(value) = value_stack.pop() {
        match value {
            EncodingToken::Value(BencodeValue::ByteString(s)) => {
                write_string(s, &mut token_stack);
            }
            EncodingToken::Value(BencodeValue::Integer(i)) => {
                write_integer(i, &mut token_stack);
            }
            EncodingToken::Value(BencodeValue::List(l)) => {
                value_stack.push(EncodingToken::ListEnd);
                for item in l.into_iter().rev() {
                    value_stack.push(EncodingToken::Value(item));
                }
                value_stack.push(EncodingToken::ListStart);
            }
            EncodingToken::Value(BencodeValue::Dict(mut d)) => {
                value_stack.push(EncodingToken::DictEnd);
                let mut dict_entries = Vec::new();
                d.sort_by(|(a, _), (b, _)| a.cmp(b));
                for (key, value) in d {
                    dict_entries.push(EncodingToken::DictEntry(key.clone()));
                    dict_entries.push(EncodingToken::Value(value.clone()));
                }
                for entry in dict_entries.into_iter().rev() {
                    value_stack.push(entry);
                }
                value_stack.push(EncodingToken::DictStart);
            }
            EncodingToken::ListEnd | EncodingToken::DictEnd => {
                // write_all on Vec never fails
                token_stack.write_all(b"e").unwrap();
            }
            EncodingToken::DictEntry(key) => {
                write_string(key, &mut token_stack);
            }
            EncodingToken::ListStart => {
                // write_all on Vec never fails
                token_stack.write_all(b"l").unwrap();
            }
            EncodingToken::DictStart => {
                // write_all on Vec never fails
                token_stack.write_all(b"d").unwrap();
            }
        }
    }
    return token_stack;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_empty() {
        let result = encode_string("");
        assert_eq!(result, b"0:");
    }

    #[test]
    fn string_hello() {
        let result = encode_string("hello");
        assert_eq!(result, b"5:hello");
    }

    #[test]
    fn string_unicode() {
        let result = encode_string("🦀");
        assert_eq!(result, b"4:\xF0\x9F\xA6\x80");
    }

    #[test]
    fn string_newline() {
        let result = encode_string("\n");
        assert_eq!(result, b"1:\n");
    }

    #[test]
    fn string_space() {
        let result = encode_string(" ");
        assert_eq!(result, b"1: ");
    }

    #[test]
    fn integer_zero() {
        let mut buffer = Vec::new();
        write_integer(0, &mut buffer);
        assert_eq!(buffer, b"i0e");
    }

    #[test]
    fn integer_positive() {
        let mut buffer = Vec::new();
        write_integer(42, &mut buffer);
        assert_eq!(buffer, b"i42e");
    }

    #[test]
    fn integer_negative() {
        let mut buffer = Vec::new();
        write_integer(-42, &mut buffer);
        assert_eq!(buffer, b"i-42e");
    }

    #[test]
    fn encode_test_string() {
        let result = encode(&&BencodeValue::ByteString("hello".into()));
        assert_eq!(result, b"5:hello");
    }

    #[test]
    fn encode_test_integer() {
        let result = encode(&BencodeValue::Integer(42));
        assert_eq!(result, b"i42e");
    }

    #[test]
    fn encode_test_list() {
        let result = encode(&BencodeValue::List(vec![
            BencodeValue::ByteString("hello".into()),
            BencodeValue::Integer(42),
        ]));
        assert_eq!(result, b"l5:helloi42ee");
    }

    #[test]
    fn encode_test_dict() {
        let result = encode(&BencodeValue::Dict(vec![
            (
                "hello".into(),
                BencodeValue::ByteString("world".into()),
            ),
            ("world".into(), BencodeValue::Integer(42)),
        ]));
        assert_eq!(result, b"d5:hello5:world5:worldi42ee");
    }

    #[test]
    fn encode_test_nested_list() {
        let result = encode(&BencodeValue::List(vec![
            BencodeValue::ByteString("hello".into()),
            BencodeValue::List(vec![
                BencodeValue::ByteString("world".into()),
                BencodeValue::Integer(42),
            ]),
        ]));
        assert_eq!(result, b"l5:hellol5:worldi42eee");
    }

    #[test]
    fn encode_test_nested_dict() {
        let result = encode(&BencodeValue::Dict(vec![
            (
                "hello".into(),
                BencodeValue::ByteString("world".into()),
            ),
            (
                "world".into(),
                BencodeValue::Dict(vec![
                    (
                        "hello".into(),
                        BencodeValue::ByteString("world".into()),
                    ),
                    ("world".into(), BencodeValue::Integer(42)),
                ]),
            ),
        ]));
        assert_eq!(result, b"d5:hello5:world5:worldd5:hello5:world5:worldi42eee");
    }

    #[test]
    fn encode_test_realworld_usecase_dht_announce_peer() {
        let result = encode(&BencodeValue::Dict(vec![
            ("t".into(), BencodeValue::ByteString("aa".into())),
            ("y".into(), BencodeValue::ByteString("q".into())),
            (
                "q".into(),
                BencodeValue::ByteString("announce_peer".into()),
            ),
            (
                "a".into(),
                BencodeValue::Dict(vec![
                    (
                        "id".into(),
                        BencodeValue::ByteString("abcdefghij0123456789".into()),
                    ),
                    (
                        "info_hash".into(),
                        BencodeValue::ByteString("mnopqrstuvwxyz123456".into()),
                    ),
                    ("port".into(), BencodeValue::Integer(6881)),
                    (
                        "token".into(),
                        BencodeValue::ByteString("aoeusnth".into()),
                    ),
                    ("implied_port".into(), BencodeValue::Integer(1)),
                ]),
            ),
        ]));
        assert_eq!(
            result,
            b"d1:ad2:id20:abcdefghij012345678912:implied_porti1e9:info_hash20:mnopqrstuvwxyz1234564:porti6881e5:token8:aoeusnthe1:q13:announce_peer1:t2:aa1:y1:qe"
        );
    }
}
