# Build all Rust crates in release mode
build:
    cargo build --release

# Download source data, parse to JSONL, and build the Tantivy index
data:
    source data/storage/.venv/bin/activate && cd data/scripts && python download.py
    source data/storage/.venv/bin/activate && cd data/scripts && python parse_all.py
    cargo run -p trans-indexer --release
