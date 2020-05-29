use base64::DecodeError as Base64Error;
use serde_json::Error as JsonError;
use std::{error, fmt};
use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    Base64(Base64Error),
    Encoding(Utf8Error),
    Format(String),
    FromStr(String),
    Json(JsonError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Base64(ref e) => write!(f, "Error in base64 encoding: {}", e),
            Error::Encoding(ref e) => write!(f, "Error in utf8 encoding: {}", e),
            Error::Format(ref e) => write!(f, "Error in token format: {}", e),
            Error::FromStr(ref e) => write!(f, "Error in parsing value: {}", e),
            Error::Json(ref e) => write!(f, "Error in json serialization: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Base64(_) => "Error in base64 encoding",
            Error::Encoding(_) => "Error in utf8 encoding",
            Error::Format(_) => "Error in token format",
            Error::FromStr(_) => "Error in parsing value",
            Error::Json(_) => "Error in json serialization",
        }
    }
}

impl From<Base64Error> for Error {
    fn from(error: Base64Error) -> Self {
        Error::Base64(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Encoding(error)
    }
}

impl From<JsonError> for Error {
    fn from(error: JsonError) -> Self {
        Error::Json(error)
    }
}
