use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{database::DataPost, user::UserAccess, Network};

pub struct PostAccess<'a, 'b> {
    post: DataPost,
    user: &'b UserAccess<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInfo {
    pub id: Uuid,
    pub user: Uuid,
    pub created_at: u64,
    pub text: String,
}

impl<'a, 'b> PostAccess<'a, 'b> {
    pub fn new(user: &'b UserAccess<'a>, post: DataPost) -> Self {
        Self {
            post,
            user,
        }
    }

    pub fn id(&self) -> Uuid {
        self.post.id
    }
    
    pub fn is_creator(&self) -> bool {
        self.user.id() == self.post.user
    }

    pub fn info(&self) -> PostInfo {
        PostInfo {
            id: self.post.id,
            created_at: self.post.created_at,
            text: self.post.text.clone(),
            user: self.post.user,
        }
    }
}
