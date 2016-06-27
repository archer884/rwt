use rustc_serialize::base64::FromBase64Error as Base64Error;
use serde_json::Error as JsonError;
use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error as EncodingError;

pub type Result<T> = ::std::result::Result<T, RwtError>;

#[derive(Debug)]
pub enum RwtError {
    Base64(Box<Base64Error>),
    Encoding(Box<EncodingError>),
    Format(String),
    Json(Box<JsonError>),
}

impl fmt::Display for RwtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RwtError::Base64(ref e) => write!(f, "Error in base64 encoding: {}", e),
            RwtError::Encoding(ref e) => write!(f, "Error in utf8 encoding: {}", e),
            RwtError::Format(ref e) => write!(f, "Error in token format: {}", e),
            RwtError::Json(ref e) => write!(f, "Error in json serialization: {}", e),
        }
    }
}

impl Error for RwtError {
    fn description(&self) -> &str {
        match *self {
            RwtError::Base64(_) => "Error in base64 encoding",
            RwtError::Encoding(_) => "Error in utf8 encoding",
            RwtError::Format(_) => "Error in token format",
            RwtError::Json(_) => "Error in json serialization",
        }
    }
}

impl From<Base64Error> for RwtError {
    fn from(error: Base64Error) -> Self {
        RwtError::Base64(box error)
    }
}

impl From<EncodingError> for RwtError {
    fn from(error: EncodingError) -> Self {
        RwtError::Encoding(box error)
    }
}

impl From<JsonError> for RwtError {
    fn from(error: JsonError) -> Self {
        RwtError::Json(box error)
    }
}
