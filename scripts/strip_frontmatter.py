#!/usr/bin/env python3
"""mdBook preprocessor that strips YAML frontmatter from Markdown chapters."""

import json
import re
import sys

if len(sys.argv) > 1 and sys.argv[1] == "supports":
    # Signal support for all renderers
    sys.exit(0)

FRONTMATTER_RE = re.compile(r"^---\s*\r?\n.*?\r?\n---\s*\r?\n", re.DOTALL)


def strip_frontmatter(content: str) -> str:
    return FRONTMATTER_RE.sub("", content, count=1).lstrip()


def process_section(section: dict) -> None:
    if "Chapter" in section:
        chapter = section["Chapter"]
        if "content" in chapter:
            chapter["content"] = strip_frontmatter(chapter["content"])
        for sub in chapter.get("sub_items", []):
            process_section(sub)


def main() -> None:
    try:
        raw = sys.stdin.read()
        data = json.loads(raw)
        if isinstance(data, list):
            _context, book = data
        else:
            book = data

        items = book.get("items", book.get("sections", []))
        for item in items:
            process_section(item)

        json.dump(book, sys.stdout)
    except Exception as e:
        sys.stderr.write(f"Frontmatter preprocessor error: {e}\n")
        sys.exit(1)


if __name__ == "__main__":
    main()
