use super::BencodedValue;

/// Encodes a string into a bencoded string.
///
/// # Arguments
///
/// * `input` - The string to encode.
///
/// # Returns
///
/// A bencoded string.
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
pub fn encode_string(input: &str) -> String {
    format!("{}:{}", input.len(), input)
}

/// Encodes an integer into a bencoded string.
///
/// # Arguments
///
/// * `input` - The integer to encode.
///
/// # Returns
///
/// A bencoded string.
///
/// # What is the integer encoded as?
///
/// The integer is encoded as follows:
/// ```i<integer>e```
///
/// Where:
/// * `<integer>` is the integer itself.
///
/// **Note:** The integer is encoded as a signed integer, and leading zeros are not allowed.
///
/// # Deviations from the specification
///
/// The specification allows for arbitrary precision integers, but this implementation only supports 64-bit integers.
///
/// *Reference:* [BEP 3](https://www.bittorrent.org/beps/bep_0003.html)
pub fn encore_integer(input: i64) -> String {
    format!("i{}e", input)
}

/// Encodes a list of Bencoded value into a bencoded string.
///
/// # Arguments
///
/// * `input` - The list of Bencoded value to encode.
///
/// # Returns
///
/// A bencoded string.
///
/// # What is the list encoded as?
///
/// The list is encoded as follows:
/// ```l<element1><element2>...e```
pub fn encode_list<T>(input: &[T]) -> String
where
    T: AsRef<str>,
{
    let mut result = String::from("l");
    for item in input {
        result.push_str(item.as_ref());
    }
    result.push('e');
    result
}

/// Encodes a dictionary of Bencoded value into a bencoded string.
///
/// # Arguments
///
/// * `input` - The dictionary of Bencoded value to encode (key sorted).
///
/// # Returns
///
/// A bencoded string.
///
/// # What is the dictionary encoded as?
///
/// The dictionary is encoded as follows:
///
/// ```d<key1><value1><key2><value2>...e```
///
/// **Note:** The keys must be sorted.
pub fn encode_dict<T, U>(input: &[(T, U)]) -> String
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    let mut result = String::from("d");
    for (key, value) in input {
        result.push_str(key.as_ref());
        result.push_str(value.as_ref());
    }
    result.push('e');
    result
}

enum EncodingToken {
    Value(BencodedValue),
    ListStart,
    ListEnd,
    DictStart,
    DictEntry(String),
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
pub fn encode(input: &BencodedValue) -> String {
    let mut token_stack = Vec::new();
    let mut value_stack = Vec::new();
    match input {
        BencodedValue::String(s) => {
            token_stack.push(encode_string(s));
        }
        BencodedValue::Integer(i) => {
            token_stack.push(encore_integer(*i));
        }
        BencodedValue::List(_) => {
            value_stack.push(EncodingToken::Value(input.clone()));
        }
        BencodedValue::Dict(_) => {
            value_stack.push(EncodingToken::Value(input.clone()));
        }
    }
    while let Some(value) = value_stack.pop() {
        match value {
            EncodingToken::Value(BencodedValue::String(s)) => {
                token_stack.push(encode_string(&s));
            }
            EncodingToken::Value(BencodedValue::Integer(i)) => {
                token_stack.push(encore_integer(i));
            }
            EncodingToken::Value(BencodedValue::List(l)) => {
                value_stack.push(EncodingToken::ListEnd);
                for item in l.into_iter().rev() {
                    value_stack.push(EncodingToken::Value(item));
                }
                value_stack.push(EncodingToken::ListStart);
            }
            EncodingToken::Value(BencodedValue::Dict(mut d)) => {
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
            EncodingToken::ListEnd => {
                token_stack.push("e".to_string());
            }
            EncodingToken::DictEnd => {
                token_stack.push("e".to_string());
            }
            EncodingToken::DictEntry(key) => {
                token_stack.push(encode_string(&key));
            }
            EncodingToken::ListStart => {
                token_stack.push("l".to_string());
            }
            EncodingToken::DictStart => {
                token_stack.push("d".to_string());
            }
        }
    }
    return token_stack.join("");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_empty() {
        let result = encode_string("");
        assert_eq!(result, "0:");
    }

    #[test]
    fn string_hello() {
        let result = encode_string("hello");
        assert_eq!(result, "5:hello");
    }

    #[test]
    fn string_unicode() {
        let result = encode_string("🦀");
        assert_eq!(result, "4:🦀");
    }

    #[test]
    fn string_newline() {
        let result = encode_string("\n");
        assert_eq!(result, "1:\n");
    }

    #[test]
    fn string_space() {
        let result = encode_string(" ");
        assert_eq!(result, "1: ");
    }

    #[test]
    fn integer_zero() {
        let result = encore_integer(0);
        assert_eq!(result, "i0e");
    }

    #[test]
    fn integer_positive() {
        let result = encore_integer(42);
        assert_eq!(result, "i42e");
    }

    #[test]
    fn integer_negative() {
        let result = encore_integer(-42);
        assert_eq!(result, "i-42e");
    }

    #[test]
    fn list_empty() {
        let result = encode_list::<&str>(&[]);
        assert_eq!(result, "le");
    }

    #[test]
    fn list_single() {
        let result = encode_list(&["5:hello"]);
        assert_eq!(result, "l5:helloe");
    }

    #[test]
    fn list_multiple() {
        let result = encode_list(&["5:hello", "5:world"]);
        assert_eq!(result, "l5:hello5:worlde");
    }

    #[test]
    fn dict_empty() {
        let result = encode_dict::<&str, &str>(&[]);
        assert_eq!(result, "de");
    }

    #[test]
    fn dict_single() {
        let result = encode_dict(&[("5:hello", "5:world")]);
        assert_eq!(result, "d5:hello5:worlde");
    }

    #[test]
    fn dict_multiple() {
        let result = encode_dict(&[("5:hello", "5:world"), ("5:world", "5:hello")]);
        assert_eq!(result, "d5:hello5:world5:world5:helloe");
    }

    #[test]
    fn encode_test_string() {
        let result = encode(&BencodedValue::String("hello".to_string()));
        assert_eq!(result, "5:hello");
    }

    #[test]
    fn encode_test_integer() {
        let result = encode(&BencodedValue::Integer(42));
        assert_eq!(result, "i42e");
    }

    #[test]
    fn encode_test_list() {
        let result = encode(&BencodedValue::List(vec![
            BencodedValue::String("hello".to_string()),
            BencodedValue::Integer(42),
        ]));
        assert_eq!(result, "l5:helloi42ee");
    }

    #[test]
    fn encode_test_dict() {
        let result = encode(&BencodedValue::Dict(vec![
            (
                "hello".to_string(),
                BencodedValue::String("world".to_string()),
            ),
            ("world".to_string(), BencodedValue::Integer(42)),
        ]));
        assert_eq!(result, "d5:hello5:world5:worldi42ee");
    }

    #[test]
    fn encode_test_nested_list() {
        let result = encode(&BencodedValue::List(vec![
            BencodedValue::String("hello".to_string()),
            BencodedValue::List(vec![
                BencodedValue::String("world".to_string()),
                BencodedValue::Integer(42),
            ]),
        ]));
        assert_eq!(result, "l5:hellol5:worldi42eee");
    }

    #[test]
    fn encode_test_nested_dict() {
        let result = encode(&BencodedValue::Dict(vec![
            (
                "hello".to_string(),
                BencodedValue::String("world".to_string()),
            ),
            (
                "world".to_string(),
                BencodedValue::Dict(vec![
                    (
                        "hello".to_string(),
                        BencodedValue::String("world".to_string()),
                    ),
                    ("world".to_string(), BencodedValue::Integer(42)),
                ]),
            ),
        ]));
        assert_eq!(result, "d5:hello5:world5:worldd5:hello5:world5:worldi42eee");
    }

    #[test]
    fn encode_test_realworld_usecase_dht_announce_peer() {
        let result = encode(&BencodedValue::Dict(vec![
            ("t".to_string(), BencodedValue::String("aa".to_string())),
            ("y".to_string(), BencodedValue::String("q".to_string())),
            (
                "q".to_string(),
                BencodedValue::String("announce_peer".to_string()),
            ),
            (
                "a".to_string(),
                BencodedValue::Dict(vec![
                    (
                        "id".to_string(),
                        BencodedValue::String("abcdefghij0123456789".to_string()),
                    ),
                    (
                        "info_hash".to_string(),
                        BencodedValue::String("mnopqrstuvwxyz123456".to_string()),
                    ),
                    ("port".to_string(), BencodedValue::Integer(6881)),
                    (
                        "token".to_string(),
                        BencodedValue::String("aoeusnth".to_string()),
                    ),
                    ("implied_port".to_string(), BencodedValue::Integer(1)),
                ]),
            ),
        ]));
        assert_eq!(
            result,
            "d1:ad2:id20:abcdefghij012345678912:implied_porti1e9:info_hash20:mnopqrstuvwxyz1234564:porti6881e5:token8:aoeusnthe1:q13:announce_peer1:t2:aa1:y1:qe"
        );
    }
}
