"""h-entry on every post; h-card on home; h-feed on lists."""
from __future__ import annotations

from pathlib import Path

import mf2py
import pytest


def _parse(path: Path) -> dict:
    return mf2py.parse(doc=path.read_text(encoding="utf-8"))


def _items_of_type(parsed: dict, t: str) -> list[dict]:
    return [it for it in parsed.get("items", []) if t in it.get("type", [])]


def _children_of_type(parsed_or_item: dict, t: str) -> list[dict]:
    items = parsed_or_item.get("items") or parsed_or_item.get("children") or []
    return [it for it in items if t in it.get("type", [])]


def test_home_has_h_card(site: Path) -> None:
    home = site / "index.html"
    parsed = _parse(home)
    cards = _items_of_type(parsed, "h-card")
    assert cards, "home page has no h-card"
    card = cards[0]
    props = card.get("properties", {})
    assert props.get("name"), "h-card.p-name missing"
    assert props.get("url"), "h-card.u-url missing"


def test_home_has_h_feed(site: Path) -> None:
    parsed = _parse(site / "index.html")
    feeds = _items_of_type(parsed, "h-feed")
    assert feeds, "home page has no h-feed"


def test_every_post_has_h_entry(post_html_files: list[Path]) -> None:
    assert post_html_files, "no post HTML files were emitted"
    for f in post_html_files:
        parsed = _parse(f)
        entries = _items_of_type(parsed, "h-entry")
        assert entries, f"{f}: no h-entry found"


@pytest.mark.parametrize("required_prop", ["name", "published", "content", "url", "author"])
def test_h_entry_has_required_properties(
    post_html_files: list[Path], required_prop: str
) -> None:
    for f in post_html_files:
        parsed = _parse(f)
        entry = _items_of_type(parsed, "h-entry")[0]
        props = entry.get("properties", {})
        assert props.get(required_prop), (
            f"{f}: h-entry missing p-/u-/dt-/e- {required_prop}"
        )


def test_h_entry_author_is_h_card(post_html_files: list[Path]) -> None:
    for f in post_html_files:
        parsed = _parse(f)
        entry = _items_of_type(parsed, "h-entry")[0]
        author = entry["properties"].get("author")
        assert author, f"{f}: h-entry missing p-author"
        first = author[0]
        assert isinstance(first, dict) and "h-card" in first.get("type", []), (
            f"{f}: p-author is not an h-card"
        )


def test_notes_index_has_h_feed(site: Path) -> None:
    parsed = _parse(site / "notes" / "index.html")
    assert _items_of_type(parsed, "h-feed"), "/notes/ has no h-feed"
