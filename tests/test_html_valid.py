"""Every emitted HTML page parses cleanly under the HTML5 parser
and contains the universal invariants from spec/output-contract.md."""
from __future__ import annotations

from pathlib import Path

import html5lib
import pytest


def _parse_strict(html: str) -> None:
    parser = html5lib.HTMLParser(strict=True)
    parser.parse(html)


def test_at_least_a_few_pages_exist(html_files: list[Path]) -> None:
    assert len(html_files) >= 4, f"expected several HTML files, got {len(html_files)}"


def test_strict_html5_parses(html_files: list[Path]) -> None:
    failures: list[str] = []
    for f in html_files:
        try:
            _parse_strict(f.read_text(encoding="utf-8"))
        except html5lib.html5parser.ParseError as e:
            failures.append(f"{f}: {e}")
    assert not failures, "\n".join(failures)


@pytest.mark.parametrize(
    "needle",
    [
        "<!DOCTYPE html>",
        '<html lang="en"',
        '<meta charset="utf-8">',
        '<meta name="viewport"',
        '<link rel="canonical"',
        '<link rel="alternate" type="application/rss+xml"',
        '<link rel="alternate" type="application/feed+json"',
        '<link rel="webmention"',
        '<link rel="pingback"',
    ],
)
def test_universal_invariants(html_files: list[Path], needle: str) -> None:
    for f in html_files:
        if f.name == "404.html" and needle.startswith('<link rel="canonical"'):
            continue  # 404s don't get a canonical
        assert needle in f.read_text(encoding="utf-8"), f"{f}: missing {needle!r}"


def test_no_script_tags(html_files: list[Path]) -> None:
    """Site must work with JS off."""
    for f in html_files:
        text = f.read_text(encoding="utf-8")
        assert "<script" not in text.lower(), f"{f}: contains <script>"
