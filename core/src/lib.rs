pub mod provider;
pub mod schema;
pub mod search;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub word: String,
    #[serde(default)]
    pub pos: String,
    #[serde(default)]
    pub pronunciation: String,
    pub definition: String,
    pub source_lang: String,
    pub target_lang: String,
    pub source: String,
}
