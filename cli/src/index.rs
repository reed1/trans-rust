use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, FuzzyTermQuery, Occur};
use tantivy::schema::{Field, Value};
use tantivy::{Index, IndexReader, Term};

use trans_core::schema::{build_schema, data_dir};

pub struct SearchIndex {
    reader: IndexReader,
    word: Field,
    word_lower: Field,
    definition: Field,
    pos: Field,
    pronunciation: Field,
    source_lang: Field,
    target_lang: Field,
    source: Field,
}

pub struct SearchResult {
    pub word: String,
    pub definition: String,
    pub pos: String,
    pub pronunciation: String,
    pub source_lang: String,
    pub target_lang: String,
    pub source: String,
}

impl SearchIndex {
    pub fn open() -> Self {
        let data_dir = data_dir();
        if !data_dir.exists() {
            eprintln!(
                "Index not found at {}.\nRun: cargo run -p trans-indexer",
                data_dir.display()
            );
            std::process::exit(1);
        }

        let index = Index::open_in_dir(&data_dir).unwrap_or_else(|e| {
            eprintln!("Failed to open index: {e}");
            std::process::exit(1);
        });

        let schema = build_schema();
        let reader = index.reader().unwrap();

        SearchIndex {
            reader,
            word: schema.get_field("word").unwrap(),
            word_lower: schema.get_field("word_lower").unwrap(),
            definition: schema.get_field("definition").unwrap(),
            pos: schema.get_field("pos").unwrap(),
            pronunciation: schema.get_field("pronunciation").unwrap(),
            source_lang: schema.get_field("source_lang").unwrap(),
            target_lang: schema.get_field("target_lang").unwrap(),
            source: schema.get_field("source").unwrap(),
        }
    }

    pub fn search(
        &self,
        query: &str,
        src_lang: &str,
        tgt_lang: &str,
        limit: usize,
    ) -> Vec<SearchResult> {
        let searcher = self.reader.searcher();
        let term = Term::from_field_text(self.word_lower, &query.to_lowercase());
        let fuzzy = FuzzyTermQuery::new(term, 1, true);

        let src_term = Term::from_field_text(self.source_lang, src_lang);
        let tgt_term = Term::from_field_text(self.target_lang, tgt_lang);
        let src_query = tantivy::query::TermQuery::new(src_term, tantivy::schema::IndexRecordOption::Basic);
        let tgt_query = tantivy::query::TermQuery::new(tgt_term, tantivy::schema::IndexRecordOption::Basic);

        let combined = BooleanQuery::new(vec![
            (Occur::Must, Box::new(fuzzy)),
            (Occur::Must, Box::new(src_query)),
            (Occur::Must, Box::new(tgt_query)),
        ]);

        let top_docs = searcher.search(&combined, &TopDocs::with_limit(limit)).unwrap();

        top_docs
            .into_iter()
            .map(|(_score, addr)| {
                let doc = searcher.doc::<tantivy::TantivyDocument>(addr).unwrap();
                let get = |f: Field| -> String {
                    doc.get_first(f)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                };
                SearchResult {
                    word: get(self.word),
                    definition: get(self.definition),
                    pos: get(self.pos),
                    pronunciation: get(self.pronunciation),
                    source_lang: get(self.source_lang),
                    target_lang: get(self.target_lang),
                    source: get(self.source),
                }
            })
            .collect()
    }
}
