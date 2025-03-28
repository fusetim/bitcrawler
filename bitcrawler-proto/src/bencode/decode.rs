use super::{BencodeString, BencodeValue, Error};

/// Decodes a bencoded string from the given input.
///
/// # Arguments
///
/// * `input` - A reference to a type that implements `AsRef<[u8]>`, representing the bencoded string.
///
/// # Returns
///
/// * `Ok(usize, BencodeString)` - The decoded string if the input is valid and the number of characters read.
/// * `Err(Error::InvalidString)` - If the input is not a valid bencoded string.
///
/// # Errors
///
/// This function will return an `Error::InvalidString` in the following cases:
/// - The input contains non-digit characters before the `:` separator.
/// - The length specified before the `:` separator is negative.
/// - The length specified is greater than the remaining characters after the `:` separator.
/// - The `:` separator is missing.
///
/// # Examples
///
/// ```rust
/// use bitcrawler_proto::bencode::{decode_string, Error};
///
/// let input = b"4:spam";
/// let result = decode_string(&input);
/// assert_eq!(result, Ok((6, "spam".into())));
///
/// let invalid_input = b"4spam";
/// let result = decode_string(&invalid_input);
/// assert!(matches!(result, Err(Error::InvalidString)));
/// ```
pub fn decode_string<T>(input: &T) -> Result<(usize, BencodeString), Error>
where
    T: AsRef<[u8]>,
{
    let input = input.as_ref();

    // Find the separator index and parse the length.
    let separator_index = input
        .iter()
        .position(|&c| c == b':')
        .ok_or(Error::InvalidString)?;
    let length = {
        let length_str = &input[0..separator_index];
        let mut value = 0;
        for &c in length_str {
            if c < b'0' || c > b'9' {
                return Err(Error::InvalidString);
            }
            value = value * 10 + (c - b'0') as usize;
        }
        value
    };

    // Return the decoded string if the length is valid.
    if length == 0 {
        return Ok((separator_index + 1, BencodeString(vec![])));
    } else if length > input.len() - separator_index - 1 {
        return Err(Error::InvalidString);
    } else {
        // Note that all indices on string are in bytes, so we need to add 1 to the separator index to skip the separator.
        // The length is the number of bytes to read a fortiori.
        return Ok((
            separator_index + length + 1,
            input[separator_index + 1..separator_index + 1 + length]
                .to_vec()
                .into(),
        ));
    }
}

/// Decodes a bencoded integer from the given input.
///
/// # Arguments
///
/// * `input` - A reference to a type that implements `AsRef<[u8]>`, representing the bencoded integer.
///
/// # Returns
///
/// * `Ok(usize, i128)` - The decoded integer if the input is valid and the number of characters read.
/// * `Err(Error::InvalidInteger)` - If the input is not a valid bencoded integer.
///
/// # Errors
///
/// This function will return an `Error::InvalidInteger` in the following cases:
///
/// - The input does not start with the `i` character.
/// - The input does not contain the `e` character.
/// - The integer is not a valid signed integer.
/// - The integer is not within the range of `i128` (spec does not specify a maximum size).
///
/// This function will not return an error if the integer is prefixed with zeros (e.g., `i000e`).
pub fn decode_integer<T>(input: &T) -> Result<(usize, i128), Error>
where
    T: AsRef<[u8]>,
{
    let input = input.as_ref();

    // Find the separator indices.
    if input[0] != b'i' {
        return Err(Error::InvalidInteger);
    }
    let end_index = input
        .iter()
        .position(|&c| c == b'e')
        .ok_or(Error::InvalidInteger)?;
    if end_index == 0 {
        return Err(Error::InvalidInteger);
    }

    // Parse the integer.
    let integer_string = String::from_utf8_lossy(&input[1..end_index]);
    let integer = integer_string
        .parse::<i128>()
        .map_err(|_| Error::InvalidInteger)?;

    // Return the decoded integer.
    Ok((end_index + 1, integer))
}

#[derive(Debug, PartialEq, Eq)]
enum DecodeState {
    Start,
    Value(BencodeValue),
    ListStart,
    DictStart,
    DictKey(BencodeString),
    DictEntry(BencodeString, BencodeValue),
}

/// Decodes a bencoded value from the given input.
///
/// # Arguments
///
/// * `input` - A reference to a type that implements `AsRef<[u8]>`, representing the bencoded value.
///
/// # Returns
///
/// * `Ok(usize, BencodedValue)` - The decoded value if the input is valid and the number of characters read.
/// * `Err(_)` - If the input is not a valid bencoded value.
pub fn decode<T>(input: &T) -> Result<(usize, BencodeValue), Error>
where
    T: AsRef<[u8]>,
{
    let input = input.as_ref();
    let len = input.len();
    let mut stack = Vec::new();
    stack.push(DecodeState::Start);

    let mut cursor = 0;
    while cursor < len {
        let char = input[cursor] as char;
        let input_ = &input[cursor..];
        match char {
            'i' => {
                let value = decode_integer(&input_)?;
                cursor += value.0;
                let state = stack.pop().expect("Invalid stack state");
                match state {
                    DecodeState::DictKey(key) => {
                        stack.push(DecodeState::DictEntry(key, BencodeValue::Integer(value.1)));
                    }
                    _ => {
                        stack.push(state);
                        stack.push(DecodeState::Value(BencodeValue::Integer(value.1)));
                    }
                }
            }
            'l' => {
                stack.push(DecodeState::ListStart);
                cursor += 1;
            }
            'd' => {
                stack.push(DecodeState::DictStart);
                cursor += 1;
            }
            'e' => {
                // End of dict/list
                cursor += 1;
                let mut values = Vec::new();
                loop {
                    if let Some(state) = stack.pop() {
                        match state {
                            DecodeState::ListStart => {
                                let mut list = Vec::new();
                                loop {
                                    if let Some(DecodeState::Value(value)) = values.pop() {
                                        list.push(value);
                                    } else {
                                        break;
                                    }
                                }
                                if !values.is_empty() {
                                    return Err(Error::InvalidValue);
                                }
                                if let Some(prev_state) = stack.pop() {
                                    match prev_state {
                                        DecodeState::DictKey(key) => {
                                            stack.push(DecodeState::DictEntry(
                                                key,
                                                BencodeValue::List(list),
                                            ));
                                        }
                                        _ => {
                                            stack.push(prev_state);
                                            stack
                                                .push(DecodeState::Value(BencodeValue::List(list)));
                                        }
                                    }
                                } else {
                                    unreachable!("Invalid stack state");
                                }
                                break;
                            }
                            DecodeState::DictStart => {
                                let mut dict = Vec::new();
                                loop {
                                    if let Some(DecodeState::DictEntry(key, value)) = values.pop() {
                                        dict.push((key, value));
                                    } else {
                                        break;
                                    }
                                }
                                if !values.is_empty() {
                                    return Err(Error::InvalidValue);
                                }
                                if let Some(prev_state) = stack.pop() {
                                    match prev_state {
                                        DecodeState::DictKey(key) => {
                                            stack.push(DecodeState::DictEntry(
                                                key,
                                                BencodeValue::Dict(dict),
                                            ));
                                        }
                                        _ => {
                                            stack.push(prev_state);
                                            stack
                                                .push(DecodeState::Value(BencodeValue::Dict(dict)));
                                        }
                                    }
                                } else {
                                    unreachable!("Invalid stack state");
                                }
                                break;
                            }
                            DecodeState::Value(_) => {
                                values.push(state);
                            }
                            DecodeState::DictEntry(_, _) => {
                                values.push(state);
                            }
                            _ => {
                                return Err(Error::InvalidValue);
                            }
                        }
                    } else {
                        return Err(Error::InvalidValue);
                    }
                }
            }
            _ => {
                let value = decode_string(&input_)?;
                let state = stack.pop().expect("Invalid stack state");
                cursor += value.0;
                match state {
                    DecodeState::DictKey(key) => {
                        stack.push(DecodeState::DictEntry(
                            key,
                            BencodeValue::ByteString(value.1),
                        ));
                    }
                    DecodeState::DictEntry(_, _) => {
                        stack.push(state);
                        stack.push(DecodeState::DictKey(value.1));
                    }
                    DecodeState::DictStart => {
                        stack.push(state);
                        stack.push(DecodeState::DictKey(value.1));
                    }
                    _ => {
                        stack.push(state);
                        stack.push(DecodeState::Value(BencodeValue::ByteString(value.1)));
                    }
                }
            }
        }
    }
    if stack.len() != 2 {
        return Err(Error::InvalidValue);
    }
    if let Some(DecodeState::Value(value)) = stack.pop() {
        Ok((cursor, value))
    } else {
        Err(Error::InvalidValue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bencoded_string() {
        let input = b"4:spam";
        let result = decode_string(&input);
        assert_eq!(result, Ok((6, "spam".into())));
    }

    #[test]
    fn test_invalid_missing_separator() {
        let input = b"4spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_invalid_non_digit_length() {
        let input = b"a:spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_invalid_negative_length() {
        let input = b"-1:spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_invalid_length_exceeds_remaining() {
        let input = b"10:spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_empty_bencoded_string() {
        let input = b"0:";
        let result = decode_string(&input);
        assert_eq!(result, Ok((2, "".into())));
    }

    #[test]
    fn test_valid_bencoded_string_with_numbers() {
        let input = b"5:12345";
        let result = decode_string(&input);
        assert_eq!(result, Ok((7, "12345".into())));
    }

    #[test]
    fn test_valid_bencoded_string_with_special_characters() {
        let input = b"6:!@#$%^";
        let result = decode_string(&input);
        assert_eq!(result, Ok((8, "!@#$%^".into())));
    }

    #[test]
    fn test_valid_bencoded_string_with_whitespace() {
        let input = b"5:hello";
        let result = decode_string(&input);
        assert_eq!(result, Ok((7, "hello".into())));
    }

    #[test]
    fn test_invalid_empty_input() {
        let input = b"";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_valid_bencoded_integer() {
        let input = b"i42e";
        let result = decode_integer(&input);
        assert_eq!(result, Ok((4, 42)));
    }

    #[test]
    fn test_invalid_missing_start() {
        let input = b"42e";
        let result = decode_integer(&input);
        assert!(matches!(result, Err(Error::InvalidInteger)));
    }

    #[test]
    fn test_invalid_missing_end() {
        let input = b"i42";
        let result = decode_integer(&input);
        assert!(matches!(result, Err(Error::InvalidInteger)));
    }

    #[test]
    fn test_invalid_non_integer() {
        let input = b"i42a";
        let result = decode_integer(&input);
        assert!(matches!(result, Err(Error::InvalidInteger)));
    }

    #[test]
    fn test_valid_negative_integer() {
        let input = b"i-42e";
        let result = decode_integer(&input);
        assert!(matches!(result, Ok((5, -42))));
    }

    #[test]
    fn test_valid_bencoded_integer_with_zeros() {
        let input = b"i000e";
        let result = decode_integer(&input);
        assert_eq!(result, Ok((5, 0)));
    }

    #[test]
    fn test_valid_bencoded_interger_zero() {
        let input = b"i0e";
        let result = decode_integer(&input);
        assert_eq!(result, Ok((3, 0)));
    }

    #[test]
    fn test_valid_bencoded_list() {
        let input = b"l4:spam4:eggse";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 14);
        assert!(matches!(result.1, BencodeValue::List(_)));
        let list = match result.1 {
            BencodeValue::List(list) => list,
            _ => panic!("Invalid value"),
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], BencodeValue::ByteString("spam".into()));
        assert_eq!(list[1], BencodeValue::ByteString("eggs".into()));
    }

    #[test]
    fn test_valid_bencoded_dict() {
        let input = b"d3:cow3:moo4:spam4:eggse";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 24);
        assert!(matches!(result.1, BencodeValue::Dict(_)));
        let dict = match result.1 {
            BencodeValue::Dict(dict) => dict,
            _ => panic!("Invalid value"),
        };
        assert_eq!(dict.len(), 2);
        assert_eq!(
            dict[0],
            ("cow".into(), BencodeValue::ByteString("moo".into()))
        );
        assert_eq!(
            dict[1],
            ("spam".into(), BencodeValue::ByteString("eggs".into()))
        );
    }

    #[test]
    fn test_valid_bencoded_dict_with_list() {
        let input = b"d4:spamli4ei-4ei0eee";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 20);
        assert!(matches!(result.1, BencodeValue::Dict(_)));
        let dict = match result.1 {
            BencodeValue::Dict(dict) => dict,
            _ => panic!("Invalid value"),
        };
        assert_eq!(dict.len(), 1);
        assert_eq!(
            dict[0],
            (
                "spam".into(),
                BencodeValue::List(vec![
                    BencodeValue::Integer(4),
                    BencodeValue::Integer(-4),
                    BencodeValue::Integer(0),
                ])
            )
        );
    }

    #[test]
    fn test_valid_bencoded_list_in_list() {
        let input = b"lli4ei-4ei0eee";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 14);
        assert!(matches!(result.1, BencodeValue::List(_)));
        let list = match result.1 {
            BencodeValue::List(list) => list,
            _ => panic!("Invalid value"),
        };
        assert_eq!(list.len(), 1);
        assert!(matches!(list[0], BencodeValue::List(_)));
        let inner_list = match &list[0] {
            BencodeValue::List(list) => list,
            _ => panic!("Invalid value"),
        };
        assert_eq!(inner_list.len(), 3);
        assert_eq!(inner_list[0], BencodeValue::Integer(4));
        assert_eq!(inner_list[1], BencodeValue::Integer(-4));
        assert_eq!(inner_list[2], BencodeValue::Integer(0));
    }

    #[test]
    fn test_valid_bencoded_dict_in_dict() {
        let input = b"d3:cowd3:moo4:spamee";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 20);
        assert!(matches!(result.1, BencodeValue::Dict(_)));
        let dict = match result.1 {
            BencodeValue::Dict(dict) => dict,
            _ => panic!("Invalid value"),
        };
        assert_eq!(dict.len(), 1);
        assert_eq!(
            dict[0],
            (
                "cow".into(),
                BencodeValue::Dict(vec![(
                    "moo".into(),
                    BencodeValue::ByteString("spam".into())
                ),])
            )
        );
    }
}
