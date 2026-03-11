# Step 1: Data Source & Offline Database

## Goal
Determine what dictionary data to use and how to store it offline.

## Requirements
- Publicly available / free license
- English <-> Indonesian word pairs
- Suitable for offline usage (embedded in the app)
- Fuzzy search with typo tolerance (including first-character typos)

## Data Sources
- **FreeDict** (freedict.org) — GPL-licensed, TEI XML format
  - Bilingual translation: EN -> ID, ID -> EN
- **Wiktionary dumps** (dumps.wikimedia.org) — CC BY-SA licensed
  - Monolingual definitions: EN -> EN, ID -> ID

## Offline Database
- **Tantivy** — Rust-native full-text search engine
- Built-in fuzzy term queries with configurable Levenshtein distance
- Embedded, no server needed
- Handles first-character typos naturally

## Data Directory
- Location: `data/` at repo root
- Shared by both TUI and Tauri apps
- If `data/` is not initialized, apps error and exit

## Pipeline
- **Python script**: Download and parse raw data into intermediate format (e.g. JSON)
- **Rust binary**: Read parsed data and build Tantivy index into `data/`

## Dev Steps
1. Explore Tantivy library
2. Write Python script to download + parse FreeDict and Wiktionary
3. Write Rust indexer to build Tantivy index from parsed data
4. Verify search works from Rust side
