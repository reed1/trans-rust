use trans_core::search::{SearchIndex, SearchResult};

fn parse_lang_pair(s: &str) -> (&str, &str) {
    match s {
        "en-id" => ("en", "id"),
        "id-en" => ("id", "en"),
        _ => {
            eprintln!("Unknown language pair: {s}\nSupported: en-id, id-en");
            std::process::exit(1);
        }
    }
}

fn print_results(results: &[SearchResult]) {
    let mut last_word = String::new();
    for r in results {
        if r.word != last_word {
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
            println!("{}{}{}", r.word, pron, pos);
            last_word = r.word.clone();
        }
        println!("  {} [{}]", r.definition, r.source);
    }
}

fn lookup(idx: &SearchIndex, word: &str, lang: Option<(&str, &str)>) {
    let output = idx.search(word, lang);

    if output.entries.is_empty() {
        eprintln!("No results found.");
        std::process::exit(1);
    }

    if !output.exact {
        println!("Did you mean: {}?", output.entries[0].word);
        println!();
    }

    print_results(&output.entries);
}

fn usage() -> ! {
    eprintln!("Usage: trans [-l en-id|id-en] <word>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (lang, word) = match args.len() {
        1 => (None, args[0].as_str()),
        3 if args[0] == "-l" => {
            let (src, tgt) = parse_lang_pair(&args[1]);
            (Some((src, tgt)), args[2].as_str())
        }
        _ => usage(),
    };

    let idx = SearchIndex::open();
    lookup(&idx, word, lang);
}
