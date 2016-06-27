#![feature(box_syntax, question_mark)]

extern crate crypto;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;

mod error;

pub use error::{Result, RwtError};

use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use serde::{Serialize, Deserialize};
use serde_json as json;
use std::str::FromStr;

use rustc_serialize::base64::{
    self,
    CharacterSet,
    FromBase64,
    Newline,
    ToBase64,
};

const BASE_CONFIG: base64::Config = base64::Config {
    char_set: CharacterSet::Standard,
    newline: Newline::LF,
    pad: false,
    line_length: None,
};

#[derive(Debug)]
pub struct Rwt<T> {
    payload: T,
    signature: String,
}

impl<T: Serialize> Rwt<T> {
    pub fn with_payload<S: AsRef<[u8]>>(payload: T, secret: S) -> Result<Rwt<T>> {
        let signature = derive_signature(&payload, Sha256::new(), secret.as_ref())?;
        Ok(Rwt {
            payload: payload,
            signature: signature,
        })
    }

    pub fn encode(&self) -> Result<String> {
        let body = json::to_string(&self.payload)?.as_bytes().to_base64(BASE_CONFIG);
        Ok(format!("{}.{}", body, self.signature))
    }

    pub fn is_valid<S: AsRef<[u8]>>(&self, secret: S) -> Result<bool> {
        let signature = derive_signature(&self.payload, Sha256::new(), secret.as_ref())?;
        Ok(self.signature == signature)
    }
}

impl<T: Deserialize> FromStr for Rwt<T> {
    type Err = RwtError;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split(".");
        let payload = parts.next().ok_or(RwtError::Format(format!("Missing body: {:?}", s)))?;
        let signature = parts.next().ok_or(RwtError::Format(format!("Missing signature: {:?}", s)))?;

        Ok(Rwt {
            payload: json::from_str(&String::from_utf8(payload.from_base64()?)?)?,
            signature: signature.to_owned(),
        })
    } 
}

fn derive_signature<D, T, S>(payload: &T, digest: D, secret: S) -> Result<String>
    where T: Serialize,
          D: Digest,
          S: AsRef<[u8]>,
{
    let mut hmac = Hmac::new(digest, secret.as_ref());
    hmac.input(json::to_string(payload)?.as_bytes());
    Ok(hmac.result().code().to_base64(BASE_CONFIG))
}
