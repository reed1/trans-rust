use crate::search::{SearchIndex, SearchOutput};

use super::Provider;

pub struct LocalProvider {
    index: SearchIndex,
}

impl LocalProvider {
    pub fn new() -> Self {
        Self {
            index: SearchIndex::open(),
        }
    }
}

impl Provider for LocalProvider {
    fn search(&self, query: &str, lang: (&str, &str)) -> SearchOutput {
        let lang_filter = if lang == ("", "") {
            None
        } else {
            Some(lang)
        };
        self.index.search(query, lang_filter)
    }

    fn name(&self) -> &str {
        "local"
    }
}
