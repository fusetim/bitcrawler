use crate::bencoding::BencodedValue;

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
    pub transaction_id: String,
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
    pub fn new(transaction_id: String, code: ErrorCode, message: String) -> Self {
        Self {
            transaction_id,
            code,
            message,
        }
    }

    /// Converts the `ErrorMessage` into a `BencodedValue`.
    pub fn to_bencoded(&self) -> BencodedValue {
        let mut dict = Vec::new();
        dict.push((
            "t".to_string(),
            BencodedValue::String(self.transaction_id.clone()),
        ));
        dict.push(("y".to_string(), BencodedValue::String("e".to_string())));
        dict.push((
            "e".to_string(),
            BencodedValue::List(vec![
                BencodedValue::Integer(self.code as i64),
                BencodedValue::String(self.message.clone()),
            ]),
        ));
        BencodedValue::Dict(dict)
    }

    /// Constructs an instance of `ErrorMessage` from a `BencodedValue`.
    pub fn try_from_bencoded(input: &BencodedValue) -> Result<Self, &'static str> {
        let dict = match input {
            BencodedValue::Dict(dict) => dict,
            _ => return Err("expected dictionary"),
        };

        let mut transaction_id = None;
        let mut code = None;
        let mut message = None;

        for (key, value) in dict {
            match key.as_str() {
                "t" => {
                    transaction_id = match value {
                        BencodedValue::String(s) => Some(s.clone()),
                        _ => return Err("expected string"),
                    };
                }
                "e" => {
                    let list = match value {
                        BencodedValue::List(list) => list,
                        _ => return Err("expected list"),
                    };

                    if list.len() != 2 {
                        return Err("expected list of length 2");
                    }

                    let code_ = match &list[0] {
                        BencodedValue::Integer(i) => *i,
                        _ => return Err("expected integer"),
                    };
                    code = match ErrorCode::try_from(code_) {
                        Ok(code) => Some(code),
                        Err(_) => return Err("invalid error code"),
                    };

                    message = match &list[1] {
                        BencodedValue::String(s) => Some(s.clone()),
                        _ => return Err("expected string"),
                    };
                }
                _ => { /* Ignore */ }
            }
        }

        let transaction_id = transaction_id.ok_or("missing transaction ID")?;
        let code = code.ok_or("missing error code")?;
        let message = message.ok_or("missing error message")?;

        Ok(Self::new(transaction_id, code, message))
    }
}

impl PartialEq for ErrorMessage {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl TryFrom<i64> for ErrorCode {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
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
            BencodedValue::Dict(vec![
                ("t".to_string(), BencodedValue::String("123".to_string())),
                ("y".to_string(), BencodedValue::String("e".to_string())),
                (
                    "e".to_string(),
                    BencodedValue::List(vec![
                        BencodedValue::Integer(201),
                        BencodedValue::String("error message".to_string()),
                    ])
                ),
            ])
        );
    }

    #[test]
    fn test_error_message_try_from_bencoded() {
        let bencoded = BencodedValue::Dict(vec![
            ("t".to_string(), BencodedValue::String("123".to_string())),
            ("y".to_string(), BencodedValue::String("e".to_string())),
            (
                "e".to_string(),
                BencodedValue::List(vec![
                    BencodedValue::Integer(201),
                    BencodedValue::String("error message".to_string()),
                ]),
            ),
        ]);
        let error = ErrorMessage::try_from_bencoded(&bencoded).unwrap();
        assert_eq!(
            error,
            ErrorMessage::new(
                "123".to_string(),
                ErrorCode::GenericError,
                "error message".to_string()
            )
        );
    }
}
