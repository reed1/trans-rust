"""Orchestrator: run all parsers, output data/storage/parsed/entries.jsonl."""

import json
import os

import parse_freedict
import parse_wiktionary

PARSED_DIR = os.path.join(os.path.dirname(__file__), "..", "storage", "parsed")
OUTPUT_FILE = os.path.join(PARSED_DIR, "entries.jsonl")


def main() -> None:
    os.makedirs(PARSED_DIR, exist_ok=True)

    entries: list[dict] = []
    entries.extend(parse_freedict.main())
    entries.extend(parse_wiktionary.main())

    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        for entry in entries:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")

    print(f"\nTotal: {len(entries)} entries written to {OUTPUT_FILE}")


if __name__ == "__main__":
    main()
