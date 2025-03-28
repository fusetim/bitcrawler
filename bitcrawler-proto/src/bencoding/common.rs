use std::borrow::Cow;

/// Represents a value encoded in the Bencode format, which is commonly used in torrent files.
///
/// # Variants
///
/// - `ByteString(BencodeString)`: Represents a Bencoded string, even if most of the time, it represents binary data and not a printable string.
/// - `Integer(i64)`: Represents a Bencoded integer.
/// - `List(BencodeList)`: Represents a Bencoded list, which is a collection of other Bencoded values.
/// - `Dict(BencodeDict)`: Represents a Bencoded dictionary, which is a collection of key-value pairs where keys are strings and values are other Bencoded values.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BencodeValue {
    ByteString(BencodeString),
    Integer(i128),
    List(BencodeList),
    Dict(BencodeDict),
}

/// Represents a Bencoded (byte) string.
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct BencodeString(pub Vec<u8>);

/// Represents a Bencoded dictionary, which is a collection of key-value pairs where keys are strings and values are other Bencoded values.
/// The keys are sorted to ensure consistent serialization (expected by the spec).
pub type BencodeDict = Vec<(BencodeString, BencodeValue)>;

/// Represents a Bencoded list, which is a collection of other Bencoded values.
/// The order of the elements is preserved.
pub type BencodeList = Vec<BencodeValue>;

impl BencodeValue {
    pub fn from_string(input: String) -> Self {
        BencodeValue::ByteString(BencodeString(input.into_bytes()))
    }

    pub fn from_integer<I>(input: I) -> Self 
    where
        I: Into<i128>,
    {
        BencodeValue::Integer(input.into())
    }

    pub fn from_list(input: Vec<BencodeValue>) -> Self {
        BencodeValue::List(input)
    }

    pub fn from_dict<T: Into<BencodeString>>(input: Vec<(T, BencodeValue)>) -> Self {
        BencodeValue::Dict(
            input
                .into_iter()
                .map(|(key, value)| (key.into(), value))
                .collect(),
        )
    }

    /// Sort the keys of all dictionaries to ensure consistent serialization (expected by the spec).
    pub fn sort_keys(&mut self) {
        match self {
            BencodeValue::Dict(dict) => {
                dict.sort_by(|(a, _), (b, _)| a.cmp(b));
                for (_, value) in dict {
                    value.sort_keys();
                }
            }
            _ => {}
        }
    }
}

impl From<String> for BencodeString {
    fn from(input: String) -> Self {
        BencodeString(input.into_bytes())
    }
}

impl From<&str> for BencodeString {
    fn from(input: &str) -> Self {
        BencodeString(input.as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for BencodeString {
    fn from(input: Vec<u8>) -> Self {
        BencodeString(input)
    }
}

impl From<BencodeString> for Cow<'_, BencodeString> {
    fn from(input: BencodeString) -> Self {
        Cow::Owned(input)
    }
}

impl<'a> From<&'a BencodeString> for Cow<'a, BencodeString> {
    fn from(input: &'a BencodeString) -> Self {
        Cow::Borrowed(input)
    }
}


impl From<BencodeString> for Vec<u8> {
    fn from(input: BencodeString) -> Self {
        input.0
    }
}

impl TryFrom<BencodeString> for String {
    type Error = std::string::FromUtf8Error;

    fn try_from(input: BencodeString) -> Result<Self, Self::Error> {
        String::from_utf8(input.0)
    }
}

impl From<Vec<BencodeValue>> for BencodeValue {
    fn from(input: Vec<BencodeValue>) -> Self {
        BencodeValue::from_list(input)
    }
}

impl<T: Into<BencodeString>> From<Vec<(T, BencodeValue)>> for BencodeValue {
    fn from(input: Vec<(T, BencodeValue)>) -> Self {
        BencodeValue::from_dict(input)
    }
}
