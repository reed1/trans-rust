# trans-rust

Offline Indonesian-English dictionary. Tantivy-powered search index shared across CLI and future TUI/Tauri frontends.

## Workspace Crates

- `core/` — shared library: Tantivy schema, `Entry` struct, data path helpers
- `indexer/` — binary: reads JSONL → builds Tantivy index
- `cli/` — one-shot lookup: `trans-cli <word>` prints results and exits
- `gui/` — GPUI desktop app with live search and dark/light theme

## Data Pipeline

1. `source data/storage/.venv/bin/activate && cd data/scripts && python download.py`
2. `python parse_all.py` → `data/storage/parsed/entries.jsonl`
3. `cargo run -p trans-indexer` → `data/storage/index/`

All generated/downloaded files live under `data/storage/` (gitignored). Python scripts in `data/scripts/` are tracked.
