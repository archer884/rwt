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

use rustc_serialize::base64::{self, CharacterSet, FromBase64, Newline, ToBase64};

const BASE_CONFIG: base64::Config = base64::Config {
    char_set: CharacterSet::Standard,
    newline: Newline::LF,
    pad: false,
    line_length: None,
};

#[derive(Debug, Eq, PartialEq)]
pub struct Rwt<T> {
    pub payload: T,
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

    pub fn is_valid<S: AsRef<[u8]>>(&self, secret: S) -> bool {
        match derive_signature(&self.payload, Sha256::new(), secret.as_ref()) {
            Err(_) => false,
            Ok(signature) => {
                crypto::util::fixed_time_eq(self.signature.as_bytes(), signature.as_bytes())
            }
        }
    }
}

impl<T: Deserialize> FromStr for Rwt<T> {
    type Err = RwtError;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split(".");
        let payload = parts.next().ok_or(RwtError::Format(format!("Missing body: {:?}", s)))?;
        let signature = parts.next()
            .ok_or(RwtError::Format(format!("Missing signature: {:?}", s)))?;

        Ok(Rwt {
            payload: json::from_str(&String::from_utf8(payload.from_base64()?)?)?,
            signature: signature.to_owned(),
        })
    }
}

fn derive_signature<D, T, S>(payload: &T, digest: D, secret: S) -> Result<String>
    where T: Serialize,
          D: Digest,
          S: AsRef<[u8]>
{
    let mut hmac = Hmac::new(digest, secret.as_ref());
    hmac.input(json::to_string(payload)?.as_bytes());
    Ok(hmac.result().code().to_base64(BASE_CONFIG))
}

#[cfg(test)]
mod tests {
    use super::Rwt;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Debug, Eq, PartialEq)]
    struct Payload {
        jti: String,
        exp: i64,
    }

    impl Serialize for Payload {
        fn serialize<S: Serializer>(&self, s: &mut S) -> Result<(), S::Error> {
            let mut map_state = s.serialize_map(Some(2))?;

            s.serialize_map_key(&mut map_state, "jti")?;
            s.serialize_map_value(&mut map_state, &self.jti)?;

            s.serialize_map_key(&mut map_state, "exp")?;
            s.serialize_map_value(&mut map_state, &self.exp)?;

            s.serialize_map_end(map_state)
        }
    }

    impl Deserialize for Payload {
        fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
            enum Field {
                Jti,
                Exp,
                Unmapped,
            }

            impl Deserialize for Field {
                fn deserialize<D: Deserializer>(d: &mut D) -> Result<Self, D::Error> {
                    struct FieldVisitor;

                    impl ::serde::de::Visitor for FieldVisitor {
                        type Value = Field;

                        fn visit_str<E: ::serde::de::Error>(&mut self,
                                                            value: &str)
                                                            -> Result<Field, E> {
                            match value {
                                "jti" => Ok(Field::Jti),
                                "exp" => Ok(Field::Exp),

                                // In the event we receive an undesired field, we return `Unmapped` because,
                                // even though we don't really care about this field, this is not an error.
                                _ => Ok(Field::Unmapped),

                                // It is also possible to throw an error in this case, e.g.:
                                // Err(::serde::de::Error::custom("unexpected field"))
                            }
                        }
                    }

                    d.deserialize(FieldVisitor)
                }
            }

            struct PayloadVisitor;

            impl ::serde::de::Visitor for PayloadVisitor {
                type Value = Payload;

                fn visit_map<V: ::serde::de::MapVisitor>(&mut self,
                                                         mut visitor: V)
                                                         -> Result<Self::Value, V::Error> {
                    let mut jti = None;
                    let mut exp = None;

                    loop {
                        match visitor.visit_key()? {
                            Some(Field::Jti) => {
                                jti = visitor.visit_value()?;
                            }
                            Some(Field::Exp) => {
                                exp = visitor.visit_value()?;
                            }
                            Some(Field::Unmapped) => (),
                            None => {
                                break;
                            }
                        }
                    }

                    let jti = match jti {
                        None => visitor.missing_field("jti")?,
                        Some(jti) => jti,
                    };

                    let exp = match exp {
                        None => visitor.missing_field("exp")?,
                        Some(exp) => exp,
                    };

                    visitor.end()?;

                    Ok(Payload {
                        jti: jti,
                        exp: exp,
                    })
                }
            }

            static FIELDS: &'static [&'static str] = &["jti", "exp"];
            d.deserialize_struct("Payload", FIELDS, PayloadVisitor)
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
        assert_eq!("eyJqdGkiOiJ0aGlzIG9uZSIsImV4cCI6MTN9.\
                    Ir9W3KCkyGNmsPFURs4Sj7aQSkuvcqpQ7kTk4F6wCyU",
                   rwt.encode().unwrap());
    }

    #[test]
    fn deserialize_rwt() {
        let rwt = create_rwt().encode().unwrap();
        let rwt: Rwt<Payload> = rwt.parse().unwrap();
        assert_eq!(rwt, create_rwt());
    }

    fn create_rwt() -> Rwt<Payload> {
        Rwt::with_payload(Payload {
                              jti: "this one".to_owned(),
                              exp: 13,
                          },
                          "secret")
            .unwrap()
    }
}
