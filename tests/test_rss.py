"""RSS 2.0 feed parses cleanly and contains expected items."""
from __future__ import annotations

from pathlib import Path

import feedparser


def test_rss_exists(site: Path) -> None:
    assert (site / "index.xml").is_file()


def test_rss_parses_without_errors(site: Path) -> None:
    parsed = feedparser.parse(str(site / "index.xml"))
    assert parsed.bozo == 0, f"feedparser bozo: {parsed.bozo_exception}"


def test_rss_has_channel_metadata(site: Path, config: dict) -> None:
    parsed = feedparser.parse(str(site / "index.xml"))
    assert parsed.feed.title == config["title"]
    assert parsed.feed.link.rstrip("/") == config["base_url"].rstrip("/")


def test_rss_has_items(site: Path) -> None:
    parsed = feedparser.parse(str(site / "index.xml"))
    assert len(parsed.entries) >= 1, "RSS feed has no items"
    for entry in parsed.entries:
        assert entry.title
        assert entry.link
        assert getattr(entry, "id", None) or getattr(entry, "guid", None)
        assert getattr(entry, "published_parsed", None) is not None
