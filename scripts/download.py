"""Download raw dictionary data to scripts/raw/."""

import io
import lzma
import os
import tarfile

import requests

RAW_DIR = os.path.join(os.path.dirname(__file__), "raw")

FREEDICT_TARBALL = "https://download.freedict.org/dictionaries/eng-ind/2025.11.23/freedict-eng-ind-2025.11.23.src.tar.xz"
KAIKKI_SOURCES = {
    "kaikki_en.jsonl.gz": "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl.gz",
    "kaikki_id.jsonl.gz": "https://kaikki.org/dictionary/Indonesian/kaikki.org-dictionary-Indonesian.jsonl.gz",
}


def download_file(url: str, dest: str) -> None:
    if os.path.exists(dest):
        print(f"  Skipping (exists): {dest}")
        return
    print(f"  Downloading: {url}")
    resp = requests.get(url, stream=True, timeout=300)
    resp.raise_for_status()
    with open(dest, "wb") as f:
        for chunk in resp.iter_content(chunk_size=1 << 20):
            f.write(chunk)
    print(f"  Saved: {dest}")


def download_freedict() -> None:
    dest = os.path.join(RAW_DIR, "freedict_eng_ind.tei")
    if os.path.exists(dest):
        print(f"  Skipping (exists): {dest}")
        return

    print(f"  Downloading: {FREEDICT_TARBALL}")
    resp = requests.get(FREEDICT_TARBALL, timeout=300)
    resp.raise_for_status()

    xz_data = lzma.decompress(resp.content)
    with tarfile.open(fileobj=io.BytesIO(xz_data)) as tar:
        for member in tar.getmembers():
            if member.name.endswith("eng-ind.tei"):
                f = tar.extractfile(member)
                if f:
                    with open(dest, "wb") as out:
                        out.write(f.read())
                    print(f"  Extracted: {dest}")
                    return
    raise RuntimeError("eng-ind.tei not found in tarball")


def main() -> None:
    os.makedirs(RAW_DIR, exist_ok=True)
    download_freedict()
    for filename, url in KAIKKI_SOURCES.items():
        download_file(url, os.path.join(RAW_DIR, filename))
    print("All downloads complete.")


if __name__ == "__main__":
    main()
