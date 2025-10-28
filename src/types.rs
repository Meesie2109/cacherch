use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    title: String,
    path: String,
    score: f32,
}

impl SearchResult {
    pub fn new(title: String, path: String, score: f32) -> Self {
        Self { title, path, score }
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn score(&self) -> &f32 {
        &self.score
    }
}
