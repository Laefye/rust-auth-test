use std::time::SystemTime;

use hmac::digest::KeyInit;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{database::{DataPost, DataSession, DataUser, DatabaseError}, post::PostAccess, Network};

pub struct UserManager<'a> {
    network: &'a Network,
}

#[derive(Debug, Clone)]
pub enum UserError {
    DatabaseError(DatabaseError),
    UserExists,
    InvalidCreditinals,
    InvalidToken,
}

impl From<DatabaseError> for UserError {
    fn from(value: DatabaseError) -> Self {
        Self::DatabaseError(value)
    }
}

pub struct UserAccess<'a> {
    network: &'a Network,
    user: DataUser,
}

impl<'a> UserManager<'a> {
    pub fn new(network: &'a Network) -> Self {
        Self {
            network,
        }
    }

    fn hash_password(salt: String, password: String) -> String {
        let mut sha = Sha256::default();
        sha.update(salt.as_bytes());
        sha.update(password.as_bytes());
        let hash = sha.finalize();
        format!("{} {:x}", salt, hash).to_string()
    }
    
    fn new_hash_password(password: String) -> String {
        let salt = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect::<String>();
        Self::hash_password(salt, password)
    }

    fn verify_password(input: String, password: String) -> bool {
        let parts = password.split(" ").collect::<Vec<&str>>();
        if parts.len() != 2 {
            return false;
        }
        password == Self::hash_password(parts[0].to_string(), input)
    }

    pub fn create_user(&self, username: String, password: String) -> Result<(), UserError> {
        if self.network.get_repository()
            .get_user_by_username(username.clone())?
            .is_some() {
            return Err(UserError::UserExists);
        }
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let user = DataUser {
            id: Uuid::new_v4(),
            username,
            password: Self::new_hash_password(password),
            created_at: created_at,
            last_active: created_at,
        };
        println!("{}", user.password);
        self.network.get_repository()
            .push_user(user)?;
        Ok(())
    }

    pub fn login(&self, username: String, password: String) -> Result<String, UserError> {
        let user = self.network.get_repository()
            .get_user_by_username(username)?;
        if user.is_none() {
            return Err(UserError::InvalidCreditinals)
        }
        let user = user.unwrap();
        if !Self::verify_password(password, user.password) {
            return Err(UserError::InvalidCreditinals);
        }
        let session = DataSession {
            id: Uuid::new_v4(),
            token: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(64)
                .map(char::from)
                .collect::<String>(),
            user: user.id,
        };
        self.network.get_repository().push_session(session.clone())?;
        Ok(session.token)
    }

    pub fn get_user_access(&self, token: String) -> Result<UserAccess, UserError> {
        let session = self.network.get_repository()
            .get_session_by_token(token)?;
        if session.is_none() {
            return Err(UserError::InvalidToken);
        }
        let user = self.network
            .get_repository()
            .get_user(session.unwrap().user)?;
        if user.is_none() {
            return Err(UserError::InvalidToken);
        }
        Ok(UserAccess::new(self.network, user.unwrap()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Me {
    pub id: Uuid,
    pub username: String,
    pub created_at: u64,
    pub last_active: u64,
}

impl<'a> UserAccess<'a> {
    fn new(network: &'a Network, user: DataUser) -> Self {
        Self {
            network,
            user,
        }
    }

    pub fn get_me(&self) -> Me {
        Me {
            id: self.user.id,
            username: self.user.username.clone(),
            created_at: self.user.created_at,
            last_active: self.user.last_active,
        }
    }

    pub fn update_active(&mut self) -> Result<(), UserError> {
        self.user.last_active = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.network.repository.update_user(self.id(), self.user.clone())?;
        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.user.id
    }

    pub fn post(&mut self, text: String) -> Result<PostAccess, UserError> {
        let post = DataPost {
            id: Uuid::new_v4(),
            text,
            user: self.id(),
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        self.update_active()?;
        self.network.get_repository().push_post(post.clone())?;
        Ok(PostAccess::new(self, post))
    }

    pub fn get_posts(&self, offset: usize, limit: usize) -> Result<Vec<PostAccess>, UserError> {
        Ok(
            self.network.get_repository()
                .get_posts_by_user(self.id(), offset, limit)?
                .iter()
                .map(|x| {
                    PostAccess::new(self, x.clone())
                })
                .collect()
        )
    }

    pub fn get_network(&self) -> &'a Network {
        self.network
    }
} 
