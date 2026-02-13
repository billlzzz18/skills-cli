use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: Uuid,
    pub namespace: String,
    pub name: String,
    pub full_name: String,
    pub owner_id: Option<Uuid>,
    pub downloads: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    pub id: Uuid,
    pub skill_id: Uuid,
    pub version: String,
    pub readme: Option<String>,
    pub instructions: Option<String>,
    pub artifact_s3_key: String,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub skill: Skill,
    pub score: f32,
    pub latest_version: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub manifest: String, // TOML content
    // In a real app, multipart would handle the file separately
}
