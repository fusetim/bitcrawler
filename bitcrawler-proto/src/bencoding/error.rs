use std::fmt::{self, Debug, Display, Formatter};

#[derive(PartialEq)]
pub enum Error {
    InvalidInteger,
    InvalidString,
    InvalidList,
    InvalidDict,
    InvalidValue,
}

impl Error {
    pub fn message(&self) -> &str {
        match self {
            Error::InvalidInteger => "Invalid integer",
            Error::InvalidString => "Invalid string",
            Error::InvalidList => "Invalid list",
            Error::InvalidDict => "Invalid dictionary",
            Error::InvalidValue => "Invalid value",
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
