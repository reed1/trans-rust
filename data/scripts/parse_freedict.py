"""Parse FreeDict TEI XML into JSONL entries (en->id + reverse id->en)."""

import json
import os

from lxml import etree

RAW_DIR = os.path.join(os.path.dirname(__file__), "..", "storage", "raw")
TEI_FILE = os.path.join(RAW_DIR, "freedict_eng_ind.tei")
NS = {"tei": "http://www.tei-c.org/ns/1.0"}


def parse(out_lines: list[dict]) -> None:
    tree = etree.parse(TEI_FILE)
    root = tree.getroot()

    for entry_el in root.iter(f"{{{NS['tei']}}}entry"):
        # Headword
        orth_el = entry_el.find(".//tei:form/tei:orth", NS)
        if orth_el is None or not orth_el.text:
            continue
        word = orth_el.text.strip()

        # Pronunciation (optional)
        pron_el = entry_el.find(".//tei:form/tei:pron", NS)
        pronunciation = pron_el.text.strip() if pron_el is not None and pron_el.text else ""

        # POS (optional)
        pos_el = entry_el.find(".//tei:gramGrp/tei:pos", NS)
        pos = pos_el.text.strip() if pos_el is not None and pos_el.text else ""

        # Each sense/translation = one entry
        for cit_el in entry_el.iter(f"{{{NS['tei']}}}cit"):
            quote_el = cit_el.find("tei:quote", NS)
            if quote_el is None or not quote_el.text:
                continue
            definition = quote_el.text.strip()

            base = {
                "word": word,
                "pos": pos,
                "pronunciation": pronunciation,
                "definition": definition,
                "source": "freedict",
            }
            # en -> id
            out_lines.append({**base, "source_lang": "en", "target_lang": "id"})
            # reverse: id -> en
            out_lines.append({
                "word": definition,
                "pos": pos,
                "pronunciation": "",
                "definition": word,
                "source_lang": "id",
                "target_lang": "en",
                "source": "freedict",
            })


def main() -> list[dict]:
    entries: list[dict] = []
    parse(entries)
    print(f"FreeDict: {len(entries)} entries (incl. reverse)")
    return entries


if __name__ == "__main__":
    for e in main():
        print(json.dumps(e, ensure_ascii=False))
