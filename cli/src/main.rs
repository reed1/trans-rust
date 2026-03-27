use trans_core::provider::{GoogleProvider, LocalProvider, Provider};
use trans_core::search::SearchResult;

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

fn usage() -> ! {
    eprintln!("Usage: trans [-l en-id|id-en] [-p local|google] <word>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut lang_str: Option<&str> = None;
    let mut provider_name = "local";
    let mut word: Option<&str> = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-l" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                lang_str = Some(args[i].as_str());
            }
            "-p" => {
                i += 1;
                if i >= args.len() {
                    usage();
                }
                provider_name = args[i].as_str();
            }
            _ if word.is_none() => {
                word = Some(args[i].as_str());
            }
            _ => usage(),
        }
        i += 1;
    }

    let word = match word {
        Some(w) => w,
        None => usage(),
    };

    let lang = match (lang_str, provider_name) {
        (Some(l), _) => parse_lang_pair(l),
        (None, "google") => ("id", "en"),
        (None, _) => ("", ""),
    };

    let provider: Box<dyn Provider> = match provider_name {
        "local" => Box::new(LocalProvider::new()),
        "google" => Box::new(GoogleProvider),
        _ => {
            eprintln!("Unknown provider: {provider_name}\nSupported: local, google");
            std::process::exit(1);
        }
    };

    let output = provider.search(word, lang);

    if output.entries.is_empty() {
        eprintln!("No results found.");
        std::process::exit(1);
    }

    if !output.exact {
        if let Some(suggestion) = &output.suggestion {
            println!("Did you mean: {}?", suggestion);
            println!();
        }
    }

    print_results(&output.entries);
}
