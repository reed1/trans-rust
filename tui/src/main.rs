mod index;
mod language;

use index::SearchIndex;
use language::LanguagePair;

fn main() {
    let supported_pairs = LanguagePair::supported_pairs();

    println!("trans-cli - Offline Translation Tool");
    println!("Supported language pairs:");
    for pair in &supported_pairs {
        println!("  {} -> {}", pair.from, pair.to);
    }

    let idx = SearchIndex::open();

    // Test query
    let results = idx.search("ability", "en", "id", 5);
    println!("\nTest query: \"ability\" (en->id)");
    if results.is_empty() {
        println!("  No results found.");
    }
    for r in &results {
        let pron = if r.pronunciation.is_empty() {
            String::new()
        } else {
            format!(" {}", r.pronunciation)
        };
        let pos = if r.pos.is_empty() {
            String::new()
        } else {
            format!(" ({})", r.pos)
        };
        println!("  {}{}{} → {} [{}]", r.word, pron, pos, r.definition, r.source);
    }

    // Test fuzzy query (typo)
    let results = idx.search("abilty", "en", "id", 5);
    println!("\nFuzzy test: \"abilty\" (en->id)");
    if results.is_empty() {
        println!("  No results found.");
    }
    for r in &results {
        println!("  {} → {} [{}]", r.word, r.definition, r.source);
    }
}
