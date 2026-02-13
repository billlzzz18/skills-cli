use axum::{
    extract::{Path, Query},
    Json,
};
use serde::Deserialize;
use crate::types::{Skill, SearchResult};
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn search_skills(Query(params): Query<SearchQuery>) -> Json<Vec<SearchResult>> {
    // Stub: In real world, query DB/Elastic
    println!("Searching for: {}", params.q);

    let stub_skill = Skill {
        id: Uuid::new_v4(),
        namespace: "google".to_string(),
        name: "search".to_string(),
        full_name: "google/search".to_string(),
        owner_id: None,
        downloads: 100,
        created_at: Utc::now(),
    };

    let result = SearchResult {
        skill: stub_skill,
        score: 1.0,
        latest_version: "1.0.0".to_string(),
        description: "Official Google Search skill".to_string(),
    };

    Json(vec![result])
}

pub async fn get_skill(Path((namespace, name)): Path<(String, String)>) -> Json<Skill> {
    // Stub
    Json(Skill {
        id: Uuid::new_v4(),
        namespace,
        name: name.clone(),
        full_name: format!("{}/{}", "stub", name), // logic fix needed in real impl
        owner_id: None,
        downloads: 50,
        created_at: Utc::now(),
    })
}
