"""draft: true notes are excluded from the build entirely."""
from __future__ import annotations

from pathlib import Path

import pytest


CONTENT = Path(__file__).resolve().parent.parent / "content"


@pytest.fixture(scope="module")
def draft_note(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """Drop a draft into a vault copy and rebuild — verify it's invisible."""
    from build.build import build_site, default_config

    src = tmp_path_factory.mktemp("vault")
    # Copy content/ into src
    for p in CONTENT.rglob("*"):
        rel = p.relative_to(CONTENT)
        dst = src / rel
        if p.is_dir():
            dst.mkdir(parents=True, exist_ok=True)
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(p.read_bytes())

    draft = src / "notes" / "secret-draft.md"
    draft.write_text(
        "---\n"
        "title: Secret draft\n"
        "date: 2026-05-09\n"
        "draft: true\n"
        "---\n\n"
        "this should never appear in output\n",
        encoding="utf-8",
    )
    out = tmp_path_factory.mktemp("public-with-draft")
    build_site(content_dir=src, out_dir=out, config=default_config())
    return out


def test_draft_html_not_emitted(draft_note: Path) -> None:
    assert not (draft_note / "notes" / "secret-draft" / "index.html").exists()


def test_draft_not_in_rss(draft_note: Path) -> None:
    rss = (draft_note / "index.xml").read_text(encoding="utf-8")
    assert "secret-draft" not in rss
    assert "Secret draft" not in rss


def test_draft_not_in_jsonfeed(draft_note: Path) -> None:
    feed = (draft_note / "feed.json").read_text(encoding="utf-8")
    assert "secret-draft" not in feed


def test_draft_not_in_sitemap(draft_note: Path) -> None:
    sitemap = (draft_note / "sitemap.xml").read_text(encoding="utf-8")
    assert "secret-draft" not in sitemap
