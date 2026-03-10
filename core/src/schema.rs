use std::path::{Path, PathBuf};

use tantivy::schema::{Schema, STORED, STRING, TEXT};

pub fn build_schema() -> Schema {
    let mut builder = Schema::builder();
    builder.add_text_field("word", STRING | STORED);
    builder.add_text_field("word_lower", STRING);
    builder.add_text_field("definition", TEXT | STORED);
    builder.add_text_field("pos", STRING | STORED);
    builder.add_text_field("pronunciation", STORED);
    builder.add_text_field("source_lang", STRING | STORED);
    builder.add_text_field("target_lang", STRING | STORED);
    builder.add_text_field("source", STRING | STORED);
    builder.build()
}

pub fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("data/storage/index")
}
