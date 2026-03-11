use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, FuzzyTermQuery, Occur, TermQuery};
use tantivy::schema::{Field, IndexRecordOption, Value};
use tantivy::{Index, IndexReader, Searcher, Term};

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

    pub fn search_exact(
        &self,
        query: &str,
        lang: Option<(&str, &str)>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let searcher = self.reader.searcher();
        let term = Term::from_field_text(self.word_lower, &query.to_lowercase());
        let term_query = TermQuery::new(term, IndexRecordOption::Basic);

        let combined = self.build_query(Box::new(term_query), lang);
        self.collect_results(&searcher, &combined, limit)
    }

    pub fn search_fuzzy(
        &self,
        query: &str,
        lang: Option<(&str, &str)>,
        limit: usize,
    ) -> Vec<SearchResult> {
        let searcher = self.reader.searcher();
        let term = Term::from_field_text(self.word_lower, &query.to_lowercase());
        let fuzzy = FuzzyTermQuery::new(term, 1, true);

        let combined = self.build_query(Box::new(fuzzy), lang);
        self.collect_results(&searcher, &combined, limit)
    }

    fn build_query(
        &self,
        main_query: Box<dyn tantivy::query::Query>,
        lang: Option<(&str, &str)>,
    ) -> BooleanQuery {
        let mut clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> =
            vec![(Occur::Must, main_query)];

        if let Some((src, tgt)) = lang {
            let src_term = Term::from_field_text(self.source_lang, src);
            let tgt_term = Term::from_field_text(self.target_lang, tgt);
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(src_term, IndexRecordOption::Basic)),
            ));
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(tgt_term, IndexRecordOption::Basic)),
            ));
        }

        BooleanQuery::new(clauses)
    }

    fn collect_results(
        &self,
        searcher: &Searcher,
        query: &BooleanQuery,
        limit: usize,
    ) -> Vec<SearchResult> {
        let top_docs = searcher.search(query, &TopDocs::with_limit(limit)).unwrap();

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
                    source: get(self.source),
                }
            })
            .collect()
    }
}
