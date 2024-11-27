use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub account: Uuid,
    pub text: String,
    pub created_at: u64,
}

impl Post {
    pub fn new(account: Uuid, text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            account: account,
            text,
            created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
        }
    }
}
