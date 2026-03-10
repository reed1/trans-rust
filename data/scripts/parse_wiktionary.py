"""Parse kaikki.org Wiktionary extracts into JSONL entries (en-en, id-id)."""

import gzip
import json
import os

RAW_DIR = os.path.join(os.path.dirname(__file__), "..", "storage", "raw")

FILES = {
    "kaikki_en.jsonl.gz": ("en", "en"),
    "kaikki_id.jsonl.gz": ("id", "id"),
}


def parse_kaikki_file(path: str, source_lang: str, target_lang: str, out_lines: list[dict]) -> int:
    count = 0
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue

            word = obj.get("word", "").strip()
            if not word:
                continue

            pos = obj.get("pos", "")
            # IPA pronunciation
            pronunciation = ""
            for sound in obj.get("sounds", []):
                if "ipa" in sound:
                    pronunciation = sound["ipa"]
                    break

            for sense in obj.get("senses", []):
                glosses = sense.get("glosses", []) or sense.get("raw_glosses", [])
                for gloss in glosses:
                    gloss = gloss.strip()
                    if not gloss:
                        continue
                    out_lines.append({
                        "word": word,
                        "pos": pos,
                        "pronunciation": pronunciation,
                        "definition": gloss,
                        "source_lang": source_lang,
                        "target_lang": target_lang,
                        "source": "wiktionary",
                    })
                    count += 1
    return count


def main() -> list[dict]:
    entries: list[dict] = []
    for filename, (src, tgt) in FILES.items():
        path = os.path.join(RAW_DIR, filename)
        if not os.path.exists(path):
            print(f"  Skipping (not found): {path}")
            continue
        n = parse_kaikki_file(path, src, tgt, entries)
        print(f"Wiktionary {src}-{tgt}: {n} entries")
    return entries


if __name__ == "__main__":
    for e in main():
        print(json.dumps(e, ensure_ascii=False))
