use crate::search::{SearchOutput, SearchResult};

use super::Provider;

pub struct GoogleProvider;

impl GoogleProvider {
    fn parse_response(json: &serde_json::Value, query: &str) -> (Vec<SearchResult>, Option<String>) {
        let mut results = Vec::new();

        // [1] — dictionary entries (from dt=bd)
        // Structure: [[pos, [translations...], [[word, [reverse...], null, freq], ...]], ...]
        if let Some(dict_data) = json.get(1).and_then(|v| v.as_array()) {
            for entry in dict_data {
                let entry_arr = match entry.as_array() {
                    Some(a) if a.len() >= 3 => a,
                    _ => continue,
                };
                let pos = entry_arr
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let detailed = match entry_arr.get(2).and_then(|v| v.as_array()) {
                    Some(d) => d,
                    None => continue,
                };

                for def in detailed {
                    let def_arr = match def.as_array() {
                        Some(a) if a.len() >= 2 => a,
                        _ => continue,
                    };
                    let word = match def_arr.first().and_then(|v| v.as_str()) {
                        Some(w) => w.to_string(),
                        None => continue,
                    };
                    let synonyms: Vec<String> = def_arr
                        .get(1)
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    let definition = if synonyms.is_empty() {
                        word.clone()
                    } else {
                        synonyms.join(", ")
                    };

                    results.push(SearchResult {
                        word,
                        definition,
                        pos: pos.clone(),
                        pronunciation: String::new(),
                        source: "google".to_string(),
                    });
                }
            }
        }

        // If no dictionary data, fall back to primary translation from [0]
        if results.is_empty() {
            if let Some(translations) = json.get(0).and_then(|v| v.as_array()) {
                for t in translations {
                    if let Some(translated) = t.get(0).and_then(|v| v.as_str()) {
                        results.push(SearchResult {
                            word: query.to_string(),
                            definition: translated.to_string(),
                            pos: String::new(),
                            pronunciation: String::new(),
                            source: "google".to_string(),
                        });
                    }
                }
            }
        }

        // Did-you-mean from [7][1] (spell correction suggestion)
        let did_you_mean = json
            .get(7)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.to_lowercase() != query.to_lowercase());

        // Silent auto-correction from [0][0][1] — the word Google actually translated
        let corrected = json
            .get(0)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s.to_lowercase() != query.to_lowercase());

        let suggestion = did_you_mean.or(corrected);

        (results, suggestion)
    }
}

impl Provider for GoogleProvider {
    fn search(&self, query: &str, lang: (&str, &str)) -> SearchOutput {
        let (sl, tl) = lang;
        let body = ureq::get("https://translate.googleapis.com/translate_a/single")
            .query("client", "gtx")
            .query("sl", sl)
            .query("tl", tl)
            .query("dt", "t")
            .query("dt", "bd")
            .query("dt", "ss")
            .query("dt", "qc")
            .query("q", query)
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        let (entries, did_you_mean) = Self::parse_response(&json, query);

        // Passthrough detection: input == output with no dictionary data means
        // Google didn't actually translate anything
        let is_passthrough = entries.len() == 1
            && entries[0].pos.is_empty()
            && entries[0].definition.eq_ignore_ascii_case(query);

        if is_passthrough {
            return SearchOutput {
                query: query.to_string(),
                exact: false,
                suggestion: None,
                entries: vec![],
            };
        }

        SearchOutput {
            query: query.to_string(),
            exact: did_you_mean.is_none(),
            suggestion: did_you_mean,
            entries,
        }
    }

    fn name(&self) -> &str {
        "google"
    }
}
