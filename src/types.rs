use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Post {
    pub id: Option<u64>,
    pub title: String,
    pub slug: String,
    pub body: Option<String>,
    pub excerpt: Option<String>,
    pub published_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostsResponse {
    pub data: Vec<Post>,
    #[allow(dead_code)]
    pub current_page: u64,
    #[allow(dead_code)]
    pub last_page: u64,
}
