use super::{BencodedValue, Error};

/// Decodes a bencoded string from the given input.
///
/// # Arguments
///
/// * `input` - A reference to a type that implements `AsRef<str>`, representing the bencoded string.
///
/// # Returns
///
/// * `Ok(usize, String)` - The decoded string if the input is valid and the number of characters read.
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
/// use bitcrawler_proto::bencoding::{decode_string, Error};
///
/// let input = "4:spam";
/// let result = decode_string(&input);
/// assert_eq!(result, Ok((6, "spam".to_string())));
///
/// let invalid_input = "4spam";
/// let result = decode_string(&invalid_input);
/// assert!(matches!(result, Err(Error::InvalidString)));
/// ```
pub fn decode_string<T>(input: &T) -> Result<(usize, String), Error>
where
    T: AsRef<str>,
{
    let input = input.as_ref();

    // Find the separator index and parse the length.
    let separator_index = input.find(':').ok_or(Error::InvalidString)?;
    let length = input[..separator_index]
        .parse::<usize>()
        .map_err(|_| Error::InvalidString)?;

    // Return the decoded string if the length is valid.
    if length == 0 {
        return Ok((separator_index + 1, "".to_string()));
    } else if length > input.len() - separator_index - 1 {
        return Err(Error::InvalidString);
    } else {
        // Note that all indices on string are in bytes, so we need to add 1 to the separator index to skip the separator.
        // The length is the number of bytes to read a fortiori.
        return Ok((
            separator_index + length + 1,
            input[separator_index + 1..separator_index + 1 + length].to_string(),
        ));
    }
}

/// Decodes a bencoded integer from the given input.
///
/// # Arguments
///
/// * `input` - A reference to a type that implements `AsRef<str>`, representing the bencoded integer.
///
/// # Returns
///
/// * `Ok(usize, i64)` - The decoded integer if the input is valid and the number of characters read.
/// * `Err(Error::InvalidInteger)` - If the input is not a valid bencoded integer.
///
/// # Errors
///
/// This function will return an `Error::InvalidInteger` in the following cases:
///
/// - The input does not start with the `i` character.
/// - The input does not contain the `e` character.
/// - The integer is not a valid signed integer.
/// - The integer is not within the range of `i64` (spec does not specify a maximum size).
///
/// This function will not return an error if the integer is prefixed with zeros (e.g., `i000e`).
pub fn decode_integer<T>(input: &T) -> Result<(usize, i64), Error>
where
    T: AsRef<str>,
{
    let input = input.as_ref();

    // Find the separator indices.
    if &input[0..1] != "i" {
        return Err(Error::InvalidInteger);
    }
    let end_index = input.find('e').ok_or(Error::InvalidInteger)?;

    // Parse the integer.
    let integer = input[1..end_index]
        .parse::<i64>()
        .map_err(|_| Error::InvalidInteger)?;

    // Return the decoded integer.
    Ok((end_index + 1, integer))
}

#[derive(Debug, PartialEq, Eq)]
enum DecodeState {
    Start,
    Value(BencodedValue),
    ListStart,
    DictStart,
    DictKey(String),
    DictEntry(String, BencodedValue),
}

/// Decodes a bencoded value from the given input.
///
/// # Arguments
///
/// * `input` - A reference to a type that implements `AsRef<str>`, representing the bencoded value.
///
/// # Returns
///
/// * `Ok(usize, BencodedValue)` - The decoded value if the input is valid and the number of characters read.
/// * `Err(_)` - If the input is not a valid bencoded value.
pub fn decode<T>(input: &T) -> Result<(usize, BencodedValue), Error>
where
    T: AsRef<str>,
{
    let input = input.as_ref();
    let len = input.len();
    let mut stack = Vec::new();
    stack.push(DecodeState::Start);

    let mut cursor = 0;
    while cursor < len {
        let char = &input[cursor..cursor + 1];
        let input_ = &input[cursor..];
        match char {
            "i" => {
                let value = decode_integer(&input_)?;
                cursor += value.0;
                let state = stack.pop().expect("Invalid stack state");
                match state {
                    DecodeState::DictKey(key) => {
                        stack.push(DecodeState::DictEntry(key, BencodedValue::Integer(value.1)));
                    }
                    _ => {
                        stack.push(state);
                        stack.push(DecodeState::Value(BencodedValue::Integer(value.1)));
                    }
                }
            }
            "l" => {
                stack.push(DecodeState::ListStart);
                cursor += 1;
            }
            "d" => {
                stack.push(DecodeState::DictStart);
                cursor += 1;
            }
            "e" => {
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
                                                BencodedValue::List(list),
                                            ));
                                        }
                                        _ => {
                                            stack.push(prev_state);
                                            stack.push(DecodeState::Value(BencodedValue::List(
                                                list,
                                            )));
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
                                                BencodedValue::Dict(dict),
                                            ));
                                        }
                                        _ => {
                                            stack.push(prev_state);
                                            stack.push(DecodeState::Value(BencodedValue::Dict(
                                                dict,
                                            )));
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
                        stack.push(DecodeState::DictEntry(key, BencodedValue::String(value.1)));
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
                        stack.push(DecodeState::Value(BencodedValue::String(value.1)));
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
        let input = "4:spam";
        let result = decode_string(&input);
        assert_eq!(result, Ok((6, "spam".to_string())));
    }

    #[test]
    fn test_invalid_missing_separator() {
        let input = "4spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_invalid_non_digit_length() {
        let input = "a:spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_invalid_negative_length() {
        let input = "-1:spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_invalid_length_exceeds_remaining() {
        let input = "10:spam";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_empty_bencoded_string() {
        let input = "0:";
        let result = decode_string(&input);
        assert_eq!(result, Ok((2, "".to_string())));
    }

    #[test]
    fn test_valid_bencoded_string_with_numbers() {
        let input = "5:12345";
        let result = decode_string(&input);
        assert_eq!(result, Ok((7, "12345".to_string())));
    }

    #[test]
    fn test_valid_bencoded_string_with_special_characters() {
        let input = "6:!@#$%^";
        let result = decode_string(&input);
        assert_eq!(result, Ok((8, "!@#$%^".to_string())));
    }

    #[test]
    fn test_valid_bencoded_string_with_whitespace() {
        let input = "5:hello";
        let result = decode_string(&input);
        assert_eq!(result, Ok((7, "hello".to_string())));
    }

    #[test]
    fn test_invalid_empty_input() {
        let input = "";
        let result = decode_string(&input);
        assert!(matches!(result, Err(Error::InvalidString)));
    }

    #[test]
    fn test_valid_bencoded_integer() {
        let input = "i42e";
        let result = decode_integer(&input);
        assert_eq!(result, Ok((4, 42)));
    }

    #[test]
    fn test_invalid_missing_start() {
        let input = "42e";
        let result = decode_integer(&input);
        assert!(matches!(result, Err(Error::InvalidInteger)));
    }

    #[test]
    fn test_invalid_missing_end() {
        let input = "i42";
        let result = decode_integer(&input);
        assert!(matches!(result, Err(Error::InvalidInteger)));
    }

    #[test]
    fn test_invalid_non_integer() {
        let input = "i42a";
        let result = decode_integer(&input);
        assert!(matches!(result, Err(Error::InvalidInteger)));
    }

    #[test]
    fn test_valid_negative_integer() {
        let input = "i-42e";
        let result = decode_integer(&input);
        assert!(matches!(result, Ok((5, -42))));
    }

    #[test]
    fn test_valid_bencoded_integer_with_zeros() {
        let input = "i000e";
        let result = decode_integer(&input);
        assert_eq!(result, Ok((5, 0)));
    }

    #[test]
    fn test_valid_bencoded_interger_zero() {
        let input = "i0e";
        let result = decode_integer(&input);
        assert_eq!(result, Ok((3, 0)));
    }

    #[test]
    fn test_valid_bencoded_list() {
        let input = "l4:spam4:eggse";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 14);
        assert!(matches!(result.1, BencodedValue::List(_)));
        let list = match result.1 {
            BencodedValue::List(list) => list,
            _ => panic!("Invalid value"),
        };
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], BencodedValue::String("spam".to_string()));
        assert_eq!(list[1], BencodedValue::String("eggs".to_string()));
    }

    #[test]
    fn test_valid_bencoded_dict() {
        let input = "d3:cow3:moo4:spam4:eggse";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 24);
        assert!(matches!(result.1, BencodedValue::Dict(_)));
        let dict = match result.1 {
            BencodedValue::Dict(dict) => dict,
            _ => panic!("Invalid value"),
        };
        assert_eq!(dict.len(), 2);
        assert_eq!(
            dict[0],
            ("cow".to_string(), BencodedValue::String("moo".to_string()))
        );
        assert_eq!(
            dict[1],
            (
                "spam".to_string(),
                BencodedValue::String("eggs".to_string())
            )
        );
    }

    #[test]
    fn test_valid_bencoded_dict_with_list() {
        let input = "d4:spamli4ei-4ei0eee";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 20);
        assert!(matches!(result.1, BencodedValue::Dict(_)));
        let dict = match result.1 {
            BencodedValue::Dict(dict) => dict,
            _ => panic!("Invalid value"),
        };
        assert_eq!(dict.len(), 1);
        assert_eq!(
            dict[0],
            (
                "spam".to_string(),
                BencodedValue::List(vec![
                    BencodedValue::Integer(4),
                    BencodedValue::Integer(-4),
                    BencodedValue::Integer(0),
                ])
            )
        );
    }

    #[test]
    fn test_valid_bencoded_list_in_list() {
        let input = "lli4ei-4ei0eee";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 14);
        assert!(matches!(result.1, BencodedValue::List(_)));
        let list = match result.1 {
            BencodedValue::List(list) => list,
            _ => panic!("Invalid value"),
        };
        assert_eq!(list.len(), 1);
        assert!(matches!(list[0], BencodedValue::List(_)));
        let inner_list = match &list[0] {
            BencodedValue::List(list) => list,
            _ => panic!("Invalid value"),
        };
        assert_eq!(inner_list.len(), 3);
        assert_eq!(inner_list[0], BencodedValue::Integer(4));
        assert_eq!(inner_list[1], BencodedValue::Integer(-4));
        assert_eq!(inner_list[2], BencodedValue::Integer(0));
    }

    #[test]
    fn test_valid_bencoded_dict_in_dict() {
        let input = "d3:cowd3:moo4:spamee";
        let result = decode(&input);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.0, 20);
        assert!(matches!(result.1, BencodedValue::Dict(_)));
        let dict = match result.1 {
            BencodedValue::Dict(dict) => dict,
            _ => panic!("Invalid value"),
        };
        assert_eq!(dict.len(), 1);
        assert_eq!(
            dict[0],
            (
                "cow".to_string(),
                BencodedValue::Dict(vec![(
                    "moo".to_string(),
                    BencodedValue::String("spam".to_string())
                ),])
            )
        );
    }
}
