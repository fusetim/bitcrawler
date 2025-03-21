/// Represents a value encoded in the Bencode format, which is commonly used in torrent files.
/// 
/// # Variants
/// 
/// - `String(String)`: Represents a Bencoded string.
/// - `Integer(i64)`: Represents a Bencoded integer.
/// - `List(Vec<BencodedValue>)`: Represents a Bencoded list, which is a collection of other Bencoded values.
/// - `Dict(Vec<(String, BencodedValue)>)`: Represents a Bencoded dictionary, which is a collection of key-value pairs where keys are strings and values are other Bencoded values.
pub enum BencodedValue {
    String(String),
    Integer(i64),
    List(Vec<BencodedValue>),
    Dict(Vec<(String, BencodedValue)>),
}

impl BencodedValue {
    pub fn from_string(input: String) -> Self {
        BencodedValue::String(input)
    }

    pub fn from_integer(input: i64) -> Self {
        BencodedValue::Integer(input)
    }

    pub fn from_list(input: Vec<BencodedValue>) -> Self {
        BencodedValue::List(input)
    }

    pub fn from_dict(input: Vec<(String, BencodedValue)>) -> Self {
        BencodedValue::Dict(input)
    }
}