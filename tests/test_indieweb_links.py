"""rel=me / rel=author / rel=canonical / rel=alternate / webmention / pingback."""
from __future__ import annotations

from pathlib import Path

import mf2py


def _rels(path: Path) -> dict:
    return mf2py.parse(doc=path.read_text(encoding="utf-8")).get("rels", {})


def test_home_has_rel_me(site: Path) -> None:
    rels = _rels(site / "index.html")
    assert rels.get("me"), "home page has no rel=me links"


def test_every_page_advertises_webmention(html_files: list[Path]) -> None:
    for f in html_files:
        rels = _rels(f)
        assert rels.get("webmention"), f"{f}: missing rel=webmention"
        assert rels.get("pingback"), f"{f}: missing rel=pingback"


def test_every_post_has_rel_author(post_html_files: list[Path]) -> None:
    for f in post_html_files:
        rels = _rels(f)
        assert rels.get("author"), f"{f}: missing rel=author"


def test_canonical_matches_url(site: Path, config: dict) -> None:
    """The canonical link on /notes/<slug>/index.html points to <base>/notes/<slug>/.

    Alias redirect pages are excepted: per ``spec/url-policy.md`` they
    point their canonical at the target URL, not at themselves.
    """
    base = config["base_url"].rstrip("/")
    for f in site.rglob("index.html"):
        if 'http-equiv="refresh"' in f.read_text(encoding="utf-8"):
            continue
        rels = _rels(f)
        canonical = rels.get("canonical")
        if not canonical:
            continue
        url = canonical[0]
        # Expected URL is base + "/" + relative path of parent dir + "/"
        rel = f.parent.relative_to(site).as_posix()
        expected_path = "/" if rel == "." else f"/{rel}/"
        assert url == base + expected_path, (
            f"{f}: canonical {url!r} != expected {base + expected_path!r}"
        )
