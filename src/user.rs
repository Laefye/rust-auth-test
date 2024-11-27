use std::time::SystemTime;

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: u64,
}

fn hash_password(password: String, salt: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password + &salt);
    format!("{}&{:x}", salt, hasher.finalize()).to_string()
}

fn hash_new_password(password: String) -> String {
    let salt = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect::<String>();
    hash_password(password, salt)
}

impl Account {
    pub fn new(username: String, password: String) -> Self {
        Account {
            id: Uuid::new_v4(),
            username,
            password: hash_new_password(password),
            created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
        }
    }

    pub fn verify_password(&self, password: String) -> bool {
        let salt = self.password.split('&').collect::<Vec<&str>>();
        if salt.len() != 2 {
            return false;
        }
        self.password == hash_password(password, salt[0].to_string())
    }
}
