
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
where T: AsRef<str> {
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
where T: AsRef<str>,
      U: AsRef<str> {
    let mut result = String::from("d");
    for (key, value) in input {
        result.push_str(key.as_ref());
        result.push_str(value.as_ref());
    }
    result.push('e');
    result
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
}