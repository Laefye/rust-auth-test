use polodb_core::{bson::{self, doc, to_bson}, Collection, CollectionT, Database};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum DatabaseError {
    UnknownError,
} 

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataUser {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: u64,
    pub last_active: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataSession {
    pub id: Uuid,
    pub user: Uuid,
    pub token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataPost {
    pub id: Uuid,
    pub user: Uuid,
    pub text: String,
    pub created_at: u64,
}

pub trait Repository {
    fn push_user(&self, user: DataUser) -> Result<(), DatabaseError>;
    fn get_user_by_username(&self, username: String) -> Result<Option<DataUser>, DatabaseError>;
    fn push_session(&self, session: DataSession) -> Result<(), DatabaseError>;
    fn get_session_by_token(&self, token: String) -> Result<Option<DataSession>, DatabaseError>;
    fn get_user(&self, id: Uuid) -> Result<Option<DataUser>, DatabaseError>;
    fn update_user(&self, id: Uuid, user: DataUser) -> Result<(), DatabaseError>;
    fn push_post(&self, post: DataPost) -> Result<(), DatabaseError>;
    fn get_posts_by_user(&self, user: Uuid, offset: usize, limit: usize) -> Result<Vec<DataPost>, DatabaseError>;
}

pub struct PoloDB(Database);

impl PoloDB {
    pub fn new() -> Self {
        Self(Database::open_path("storage").unwrap())
    }

    fn get_user_collection(&self) -> Collection<DataUser> {
        self.0.collection("user")
    }

    fn get_session_collection(&self) -> Collection<DataSession> {
        self.0.collection("session")
    }

    fn get_post_collection(&self) -> Collection<DataPost> {
        self.0.collection("post")
    }
}

impl From<polodb_core::Error> for DatabaseError {
    fn from(_: polodb_core::Error) -> Self {
        Self::UnknownError
    }
}

impl Repository for PoloDB {
    fn push_user(&self, user: DataUser) -> Result<(), DatabaseError> {
        self.get_user_collection()
            .insert_one(user)
            .map_err(|_| DatabaseError::UnknownError)
            .map(|_| ())
    }
    
    fn get_user_by_username(&self, username: String) -> Result<Option<DataUser>, DatabaseError> {
        let user = self.get_user_collection()
            .find(doc! {})
            .run()
            .map_err(|_| DatabaseError::UnknownError)?
            .map(|x| x.unwrap())
            .find(|x| x.username == username);
        match user {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }
    
    fn push_session(&self, session: DataSession) -> Result<(), DatabaseError> {
        self.get_session_collection()
            .insert_one(session)
            .map_err(|_| DatabaseError::UnknownError)?;
        Ok(())
    }
    
    fn get_session_by_token(&self, token: String) -> Result<Option<DataSession>, DatabaseError> {
        let session = self.get_session_collection()
            .find(doc! {})
            .run()
            .map_err(|_| DatabaseError::UnknownError)?
            .map(|x| x.unwrap())
            .find(|x| x.token == token);
        match session {
            Some(session) => Ok(Some(session)),
            None => Ok(None),
        }
    }
    
    fn get_user(&self, id: Uuid) -> Result<Option<DataUser>, DatabaseError> {
        let user = self.get_user_collection()
            .find(doc! {})
            .run()
            .map_err(|_| DatabaseError::UnknownError)?
            .map(|x| x.unwrap())
            .find(|x| x.id == id);
        match user {
            Some(user) => Ok(Some(user)),
            None => Ok(None),
        }
    }
    
    fn update_user(&self, id: Uuid, user: DataUser) -> Result<(), DatabaseError> {
        self.get_user_collection()
            .update_one(
                doc! {"id": id.to_string()},
                doc! {
                    "$set": to_bson(&user).unwrap()
                }
            )?;
        Ok(())
    }
    
    fn push_post(&self, post: DataPost) -> Result<(), DatabaseError> {
        self.get_post_collection()
            .insert_one(post)?;
        Ok(())
    }
    
    fn get_posts_by_user(&self, user: Uuid, offset: usize, limit: usize) -> Result<Vec<DataPost>, DatabaseError> {
        let mut posts = self.get_post_collection()
            .find(doc! {})
            .run()?
            .map(|x| x.unwrap())
            .filter(|x| x.user == user)
            .collect::<Vec<DataPost>>();
        posts.reverse();
        Ok(
            posts.iter()
                .skip(offset)
                .take(limit)
                .cloned()
                .collect()
        )
    }
}
