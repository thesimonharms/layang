use anyhow::{anyhow, Result};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

fn api_error(status: reqwest::StatusCode, body: &str) -> anyhow::Error {
    // Try to extract Laravel's "message" field from JSON error responses
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
            return anyhow!("HTTP {} — {}", status, msg);
        }
    }
    anyhow!("HTTP {} — {}", status, body)
}

use crate::types::{Post, PostsResponse};

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

impl ApiClient {
    pub fn new(base_url: String, token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        Self {
            client,
            base_url,
            token,
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    pub async fn list_posts(&self) -> Result<Vec<Post>> {
        let url = format!("{}/api/posts", self.base_url);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let posts_response: PostsResponse = response.json().await?;
        Ok(posts_response.data)
    }

    pub async fn list_drafts(&self) -> Result<Vec<Post>> {
        let url = format!("{}/api/posts/drafts", self.base_url);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let posts_response: PostsResponse = response.json().await?;
        Ok(posts_response.data)
    }

    pub async fn get_post(&self, slug: &str) -> Result<Post> {
        let url = format!("{}/api/posts/{}", self.base_url, slug);
        let response = self
            .client
            .get(&url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let post: Post = response.json().await?;
        Ok(post)
    }

    pub async fn create_post(
        &self,
        title: &str,
        body: &str,
        excerpt: &str,
    ) -> Result<Post> {
        let url = format!("{}/api/posts", self.base_url);
        let payload = json!({
            "title": title,
            "body": body,
            "excerpt": excerpt,
        });

        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let post: Post = response.json().await?;
        Ok(post)
    }

    pub async fn update_post(
        &self,
        slug: &str,
        title: &str,
        body: &str,
        excerpt: &str,
    ) -> Result<Post> {
        let url = format!("{}/api/posts/{}", self.base_url, slug);
        let payload = json!({
            "title": title,
            "body": body,
            "excerpt": excerpt,
        });

        let response = self
            .client
            .put(&url)
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let post: Post = response.json().await?;
        Ok(post)
    }

    pub async fn delete_post(&self, slug: &str) -> Result<()> {
        let url = format!("{}/api/posts/{}", self.base_url, slug);
        let response = self
            .client
            .delete(&url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        Ok(())
    }

    pub async fn publish_post(&self, slug: &str) -> Result<Post> {
        let url = format!("{}/api/posts/{}/publish", self.base_url, slug);
        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let post: Post = response.json().await?;
        Ok(post)
    }

    pub async fn unpublish_post(&self, slug: &str) -> Result<Post> {
        let url = format!("{}/api/posts/{}/unpublish", self.base_url, slug);
        let response = self
            .client
            .post(&url)
            .header(AUTHORIZATION, self.auth_header())
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(api_error(status, &body_text));
        }

        let post: Post = response.json().await?;
        Ok(post)
    }
}
