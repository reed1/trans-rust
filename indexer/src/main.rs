use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use tantivy::doc;
use tantivy::Index;
use trans_core::Entry;
use trans_core::schema::{build_schema, data_dir};

fn main() {
    let jsonl_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("scripts/parsed/entries.jsonl");

    if !jsonl_path.exists() {
        eprintln!(
            "JSONL file not found: {}\nRun `python scripts/parse_all.py` first.",
            jsonl_path.display()
        );
        std::process::exit(1);
    }

    let data_dir = data_dir();
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir).unwrap();
    }
    fs::create_dir_all(&data_dir).unwrap();

    let schema = build_schema();
    let index = Index::create_in_dir(&data_dir, schema.clone()).unwrap();
    let mut writer = index.writer(50_000_000).unwrap();

    let word = schema.get_field("word").unwrap();
    let word_lower = schema.get_field("word_lower").unwrap();
    let definition = schema.get_field("definition").unwrap();
    let pos = schema.get_field("pos").unwrap();
    let pronunciation = schema.get_field("pronunciation").unwrap();
    let source_lang = schema.get_field("source_lang").unwrap();
    let target_lang = schema.get_field("target_lang").unwrap();
    let source = schema.get_field("source").unwrap();

    let start = Instant::now();
    let file = File::open(&jsonl_path).unwrap();
    let reader = BufReader::new(file);
    let mut count = 0u64;

    for line in reader.lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let entry: Entry = serde_json::from_str(&line).unwrap();
        writer.add_document(doc!(
            word => entry.word.as_str(),
            word_lower => entry.word.to_lowercase().as_str(),
            definition => entry.definition.as_str(),
            pos => entry.pos.as_str(),
            pronunciation => entry.pronunciation.as_str(),
            source_lang => entry.source_lang.as_str(),
            target_lang => entry.target_lang.as_str(),
            source => entry.source.as_str(),
        )).unwrap();
        count += 1;
    }

    writer.commit().unwrap();
    let elapsed = start.elapsed();

    let meta = serde_json::json!({
        "version": 1,
        "entries": count,
        "built_secs": elapsed.as_secs_f64(),
    });
    fs::write(data_dir.join("index_meta.json"), serde_json::to_string_pretty(&meta).unwrap()).unwrap();

    println!("Indexed {count} entries in {elapsed:.2?} → {}", data_dir.display());
}
