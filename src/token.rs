use std::{alloc::System, ops::Add, time::{Duration, SystemTime, UNIX_EPOCH}};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SECRET: &str = "secret";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub account_id: Uuid,
    pub exp: usize,
}

impl Claims {
    pub fn new(account_id: Uuid) -> Self {
        Self {
            account_id,
            exp: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::new(60 * 60 * 24, 0)).as_secs() as usize
        }
    }
}

pub fn encrypt_token(claims: Claims) -> String {
    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET.to_string().as_bytes())).unwrap()
}

pub fn decrypt_token(token: String) -> Option<Claims> {
    let result = decode(token.as_str(), &DecodingKey::from_secret(SECRET.to_string().as_bytes()), &Validation::default())
        .map(|x| x.claims);
    result.ok()
}
