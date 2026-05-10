"""JSON Feed v1.1 — schema-shaped validation."""
from __future__ import annotations

import json
from pathlib import Path


def test_jsonfeed_exists(site: Path) -> None:
    assert (site / "feed.json").is_file()


def test_jsonfeed_top_level(site: Path, config: dict) -> None:
    feed = json.loads((site / "feed.json").read_text(encoding="utf-8"))
    assert feed["version"] == "https://jsonfeed.org/version/1.1"
    assert feed["title"] == config["title"]
    assert feed["home_page_url"].rstrip("/") == config["base_url"].rstrip("/")
    assert feed["feed_url"].endswith("/feed.json")
    assert feed["language"] == "en"
    assert isinstance(feed["authors"], list) and feed["authors"]
    assert isinstance(feed["items"], list) and feed["items"]


def test_jsonfeed_items(site: Path) -> None:
    feed = json.loads((site / "feed.json").read_text(encoding="utf-8"))
    for item in feed["items"]:
        assert item["id"]
        assert item["url"]
        assert item["title"]
        assert item["content_html"]
        assert "date_published" in item
