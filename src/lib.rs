mod error;

use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use serde_json as json;
use std::fmt::Display;
use std::str::FromStr;

pub use error::Error;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

/// Decode base64 into a string.
///
/// Useful for converting incoming base64 tokens to json before deserializing. It is now necessary
/// to do this, as far as I can tell, because serde now supports deserializing to a struct that
/// only borrows the data it represents instead of owning it.
pub fn decode_base64(s: &str) -> Option<String> {
    let start_idx = match s.find('.').map(|idx| idx + 1) {
        None => return None,
        Some(idx) => idx,
    };

    let s = &s[start_idx..];
    base64::decode(s)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

/// Represents a web token.
///
/// For optimal usage, your payload should be any struct implementing `Serialize`, `Deserialize`,
/// and `FromStr`, but none of these are technically required.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Rwt<T> {
    pub payload: T,
    signature: String,
}

impl<T: Serialize> Rwt<T> {
    /// Create a web token with the provided payload.
    ///
    /// This function requires that the payload be `Serialize`.
    pub fn with_payload<S: AsRef<[u8]>>(payload: T, secret: S) -> Result<Rwt<T>> {
        let signature = derive_signature(&payload, Sha256::new(), secret.as_ref())?;
        Ok(Rwt {
            payload: payload,
            signature: signature,
        })
    }

    /// Encode the token as base64 in the usual format.
    ///
    /// In this case, "the usual format" means `xxx.xxx` where the left hand side is the token
    /// itself and the right hand side is the signature. The base64 implementation used currently
    /// introduces padding into the equation.
    pub fn encode(&self) -> Result<String> {
        let body = base64::encode(json::to_string(&self.payload)?.as_bytes());
        Ok(format!("{}.{}", body, self.signature))
    }

    /// Validate the token.
    ///
    /// This function compares the token as serialized against a freshly-derived signature to
    /// ensure that it is original and un-tampered-with. This version uses `rust-crypto` to
    /// compare the two results in order to protect against timing attacks.
    pub fn is_valid<S: AsRef<[u8]>>(&self, secret: S) -> bool {
        match derive_signature(&self.payload, Sha256::new(), secret.as_ref()) {
            Err(_) => false,
            Ok(signature) => {
                crypto::util::fixed_time_eq(self.signature.as_bytes(), signature.as_bytes())
            }
        }
    }
}

impl<T, E> FromStr for Rwt<T>
where
    E: Display,
    T: FromStr<Err = E>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        use std::str;

        let mut parts = s.split('.');
        let payload = parts
            .next()
            .ok_or_else(|| Error::Format(format!("Missing body: {:?}", s)))?;
        let signature = parts
            .next()
            .ok_or_else(|| Error::Format(format!("Missing signature: {:?}", s)))?;

        let payload = base64::decode(payload)?;
        let payload = str::from_utf8(&payload)?;
        let payload = payload
            .parse::<T>()
            .map_err(|e| Error::FromStr(format!("Unable to parse body as payload: {}", e)))?;

        Ok(Rwt {
            payload: payload,
            signature: signature.to_owned(),
        })
    }
}

fn derive_signature<D, T, S>(payload: &T, digest: D, secret: S) -> Result<String>
where
    T: Serialize,
    D: Digest,
    S: AsRef<[u8]>,
{
    let mut hmac = Hmac::new(digest, secret.as_ref());
    hmac.input(json::to_string(payload)?.as_bytes());
    Ok(base64::encode(hmac.result().code()))
}

#[cfg(test)]
mod tests {
    use super::Rwt;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::str::FromStr;

    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Payload {
        jti: String,
        exp: i64,
    }

    impl FromStr for Payload {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            serde_json::from_str(s).map_err(|_| "Sorry, Charlie.")
        }
    }

    #[test]
    fn create_rwt_with_payload() {
        create_rwt();
    }

    #[test]
    fn validate_rwt() {
        let rwt = create_rwt();
        assert!(rwt.is_valid("secret"));
    }

    #[test]
    fn invalidate_rwt() {
        let rwt = create_rwt();
        assert!(!rwt.is_valid("other secret"));
    }

    #[test]
    fn serialize_rwt() {
        let rwt = create_rwt();
        assert_eq!(
            "eyJqdGkiOiJ0aGlzIG9uZSIsImV4cCI6MTN9.\
                    Ir9W3KCkyGNmsPFURs4Sj7aQSkuvcqpQ7kTk4F6wCyU=",
            rwt.encode().unwrap()
        );
    }

    #[test]
    fn deserialize_rwt() {
        let rwt = create_rwt().encode().unwrap();
        let rwt = rwt.parse::<Rwt<Payload>>().unwrap();
        assert_eq!(rwt, create_rwt());
    }

    fn create_rwt() -> Rwt<Payload> {
        Rwt::with_payload(
            Payload {
                jti: "this one".to_owned(),
                exp: 13,
            },
            "secret",
        )
        .unwrap()
    }
}
