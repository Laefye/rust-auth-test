use std::sync::Arc;

use database::Repository;
use user::UserManager;

pub mod database;
pub mod user;
pub mod post;

pub struct Network {
    repository: Arc<dyn Repository>,
}

impl Network
{
    pub fn new<T>(repository: T) -> Self
    where 
        T: Repository + 'static
    {
        Self {
            repository: Arc::new(repository),
        }
    }

    pub fn user_manager(&self) -> UserManager {
        UserManager::new(self)
    }

    pub fn get_repository(&self) -> Arc<dyn Repository> {
        self.repository.clone()
    }
}
