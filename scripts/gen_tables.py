"""Regenerate build/entities.rs and the tables in build/unicode.rs.

The Rust build has no dependencies, so the HTML5 entity list and the
Unicode character classes it needs are checked in as generated data,
taken from this Python's standard library (html.entities, unicodedata).
Run with the Python version named in build/unicode.rs's header to get a
byte-identical result; a newer Python updates the Unicode version.

    python3 scripts/gen_tables.py
"""
from __future__ import annotations

import html.entities
import platform
import re
import sys
import unicodedata
from pathlib import Path

BUILD = Path(__file__).resolve().parent.parent / "build"
WIDTH = 100


def rust_str(s: str) -> str:
    out = []
    for ch in s:
        if " " <= ch <= "~" and ch not in '"\\':
            out.append(ch)
        else:
            out.append(f"\\u{{{ord(ch):x}}}")
    return '"' + "".join(out) + '"'


def ranges(pred) -> list[tuple[int, int]]:
    result: list[tuple[int, int]] = []
    start = None
    for cp in range(sys.maxunicode + 2):
        hit = cp <= sys.maxunicode and pred(chr(cp))
        if hit and start is None:
            start = cp
        elif not hit and start is not None:
            result.append((start, cp - 1))
            start = None
    return result


def wrap(items: list[str]) -> str:
    lines, line = [], "   "
    for item in items:
        if len(line) + 1 + len(item) > WIDTH:
            lines.append(line)
            line = "   "
        line += " " + item
    lines.append(line)
    return "\n".join(lines)


def table(name: str, doc: str, pred) -> str:
    items = [f"(0x{a:x}, 0x{b:x})," for a, b in ranges(pred)]
    return f"/// {doc}\nstatic {name}: &[(u32, u32)] = &[\n{wrap(items)}\n];\n"


def entities() -> str:
    names = sorted({k[:-1] for k in html.entities.html5 if k.endswith(";")})
    rows = "\n".join(
        f"    ({rust_str(n)}, {rust_str(html.entities.html5[n + ';'])})," for n in names
    )
    return (
        "//! HTML5 named character references (generated from Python's html.entities.html5;\n"
        "//! names have the trailing ';' stripped, as markdown-it-py does). Sorted for binary search.\n"
        "\n"
        "pub static ENTITIES: &[(&str, &str)] = &[\n"
        f"{rows}\n"
        "];\n"
        "\n"
        "pub fn lookup(name: &str) -> Option<&'static str> {\n"
        "    ENTITIES.binary_search_by(|(k, _)| (*k).cmp(name)).ok().map(|i| ENTITIES[i].1)\n"
        "}\n"
    )


def unicode_tables() -> str:
    header = (
        f"//! Unicode property tables (generated from Python {platform.python_version()}, "
        f"Unicode {unicodedata.unidata_version}) so that\n"
        "//! character classes match the reference implementation exactly.\n"
        "\n"
    )
    punct = table(
        "PUNCT_OR_SYMBOL",
        "General category P* or S* (markdown-it `isPunctChar`).",
        lambda c: unicodedata.category(c)[0] in "PS",
    )
    word = table(
        "WORD",
        "Python `re` word characters: `str.isalnum()` or `_`.",
        lambda c: c.isalnum() or c == "_",
    )
    return header + punct + "\n" + word


def main() -> None:
    (BUILD / "entities.rs").write_text(entities(), encoding="utf-8")
    path = BUILD / "unicode.rs"
    text = path.read_text(encoding="utf-8")
    tail = text[text.index("\nfn in_table") :]
    path.write_text(unicode_tables() + tail, encoding="utf-8")


if __name__ == "__main__":
    main()
