use uuid::Uuid;

use crate::{post::Post, user::Account};

pub enum RepositoryError {
    NotFound,
    DatabaseError,
}

pub trait Repository {
    fn create_account(&mut self, account: &Account) -> Result<(), RepositoryError>;
    fn get_account_by_username(&self, username: String) -> Result<Account, RepositoryError>;
    fn get_account(&self, id: Uuid) -> Result<Account, RepositoryError>;
    fn login_account(&self, username: String, password: String) -> Result<Account, RepositoryError> {
        self.get_account_by_username(username)
            .and_then(|x| if x.verify_password(password) { Ok(x) } else { Err(RepositoryError::NotFound) })
    }
    fn create_post(&mut self, post: &Post) -> Result<(), RepositoryError>;
    fn get_post(&self, id: Uuid) -> Result<Post, RepositoryError>;
}

#[derive(Debug)]
pub struct MemoryRepository {
    pub accounts: Vec<Account>,
    pub posts: Vec<Post>,
}

impl MemoryRepository {
    pub fn new() -> Self {
        MemoryRepository {
            accounts: Vec::new(),
            posts: Vec::new(),
        }
    }
}

impl Repository for MemoryRepository {
    fn get_account_by_username(&self, username: String) -> Result<Account, RepositoryError> {
        self.accounts.iter()
            .find(|x| x.username == username)
            .ok_or(RepositoryError::NotFound)
            .cloned()
    }

    fn get_account(&self, id: Uuid) -> Result<Account, RepositoryError> {
        self.accounts.iter()
            .find(|x| x.id == id)
            .ok_or(RepositoryError::NotFound)
            .cloned()
    }
    
    fn create_account(&mut self, account: &Account) -> Result<(), RepositoryError> {
        self.accounts.push(account.clone());
        Ok(())
    }
    
    fn create_post(&mut self, post: &Post) -> Result<(), RepositoryError> {
        self.posts.push(post.clone());
        Ok(())
    }
    
    fn get_post(&self, id: Uuid) -> Result<Post, RepositoryError> {
        self.posts.iter()
            .find(|x| x.id == id)
            .ok_or(RepositoryError::NotFound)
            .cloned()
    }
}
