use crate::bencode::{BencodeString, BencodeValue};

/// Represents an error message in a KRPC response.
///
/// # Fields
///
/// - `transaction_id`: The transaction ID of the request that caused the error.
/// - `code`: The error code.
/// - `message`: The error message.
#[derive(Debug, Clone, Eq)]
pub struct ErrorMessage {
    /// The transaction ID of the request that caused the error.
    pub transaction_id: BencodeString,
    /// The error code.
    pub code: ErrorCode,
    /// The error message.
    pub message: String,
}

/// Represents an error code in a KRPC error message.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ErrorCode {
    /// The generic error code.
    GenericError = 201,
    /// The server error code.
    ServerError = 202,
    /// The protocol error code.
    ProtocolError = 203,
    /// The method unknown error code.
    MethodUnknown = 204,
}

impl ErrorMessage {
    /// Constructs a new `ErrorMessage`.
    ///
    /// # Parameters
    ///
    /// - `transaction_id`: The transaction ID of the request that caused the error.
    /// - `code`: The error code.
    /// - `message`: The error message.
    ///
    /// # Returns
    ///
    /// A new instance of `ErrorMessage`.
    pub fn new(transaction_id: impl Into<BencodeString>, code: ErrorCode, message: String) -> Self {
        Self {
            transaction_id: transaction_id.into(),
            code,
            message,
        }
    }

    /// Converts the `ErrorMessage` into a `BencodedValue`.
    pub fn to_bencoded(&self) -> BencodeValue {
        let mut dict = Vec::new();
        dict.push((
            "t".into(),
            BencodeValue::ByteString(self.transaction_id.clone()),
        ));
        dict.push(("y".into(), BencodeValue::ByteString("e".into())));
        dict.push((
            "e".into(),
            BencodeValue::List(vec![
                BencodeValue::Integer(self.code as i128),
                BencodeValue::ByteString(BencodeString::from(self.message.as_str())),
            ]),
        ));
        BencodeValue::Dict(dict)
    }

    /// Constructs an instance of `ErrorMessage` from a `BencodedValue`.
    pub fn try_from_bencoded(input: &BencodeValue) -> Result<Self, &'static str> {
        let dict = match input {
            BencodeValue::Dict(dict) => dict,
            _ => return Err("expected dictionary"),
        };

        let mut transaction_id = None;
        let mut code = None;
        let mut message = None;

        for (key, value) in dict {
            match key.as_ref() {
                b"t" => {
                    transaction_id = match value {
                        BencodeValue::ByteString(s) => Some(s.clone()),
                        _ => return Err("expected string"),
                    };
                }
                b"e" => {
                    let list = match value {
                        BencodeValue::List(list) => list,
                        _ => return Err("expected list"),
                    };

                    if list.len() != 2 {
                        return Err("expected list of length 2");
                    }

                    let code_ = match &list[0] {
                        BencodeValue::Integer(i) => *i,
                        _ => return Err("expected integer"),
                    };
                    code = match ErrorCode::try_from(code_) {
                        Ok(code) => Some(code),
                        Err(_) => return Err("invalid error code"),
                    };

                    message = match &list[1] {
                        BencodeValue::ByteString(s) => Some(s.clone()),
                        _ => return Err("expected string"),
                    };
                }
                _ => { /* Ignore */ }
            }
        }

        let transaction_id = transaction_id.ok_or("missing transaction ID")?;
        let code = code.ok_or("missing error code")?;
        let message = message.ok_or("missing error message")?;

        match String::try_from(message) {
            Ok(message) => Ok(Self {
                transaction_id,
                code,
                message,
            }),
            Err(_) => Err("invalid error message"),
        }
    }
}

impl PartialEq for ErrorMessage {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl TryFrom<i128> for ErrorCode {
    type Error = ();

    fn try_from(value: i128) -> Result<Self, Self::Error> {
        match value {
            201 => Ok(Self::GenericError),
            202 => Ok(Self::ServerError),
            203 => Ok(Self::ProtocolError),
            204 => Ok(Self::MethodUnknown),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_error_message_to_bencoded() {
        let error = ErrorMessage::new(
            "123".to_string(),
            ErrorCode::GenericError,
            "error message".to_string(),
        );
        let bencoded = error.to_bencoded();
        assert_eq!(
            bencoded,
            BencodeValue::Dict(vec![
                ("t".into(), BencodeValue::ByteString("123".into())),
                ("y".into(), BencodeValue::ByteString("e".into())),
                (
                    "e".into(),
                    BencodeValue::List(vec![
                        BencodeValue::Integer(201),
                        BencodeValue::ByteString("error message".into()),
                    ])
                ),
            ])
        );
    }

    #[test]
    fn test_error_message_try_from_bencoded() {
        let bencoded = BencodeValue::Dict(vec![
            ("t".into(), BencodeValue::ByteString("123".into())),
            ("y".into(), BencodeValue::ByteString("e".into())),
            (
                "e".into(),
                BencodeValue::List(vec![
                    BencodeValue::Integer(201),
                    BencodeValue::ByteString("error message".into()),
                ]),
            ),
        ]);
        let error = ErrorMessage::try_from_bencoded(&bencoded).unwrap();
        assert_eq!(
            error,
            ErrorMessage::new(
                "123",
                ErrorCode::GenericError,
                "error message".to_string()
            )
        );
    }
}
