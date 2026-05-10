"""URL policy: trailing slashes, no .html in URLs, slug = filename stem."""
from __future__ import annotations

from pathlib import Path


def test_pages_emitted_at_trailing_slash_directories(site: Path) -> None:
    expected = {
        site / "index.html",
        site / "about" / "index.html",
        site / "colophon" / "index.html",
        site / "notes" / "index.html",
        site / "notes" / "how-this-bliki-works" / "index.html",
    }
    missing = [str(p.relative_to(site)) for p in expected if not p.is_file()]
    assert not missing, f"missing expected pages: {missing}"


def test_special_files_emitted(site: Path) -> None:
    for f in ("index.xml", "feed.json", "sitemap.xml", "robots.txt", "404.html"):
        assert (site / f).is_file(), f"missing {f}"


def test_no_html_extension_in_internal_links(html_files: list[Path]) -> None:
    """Internal links use trailing slashes, never explicit .html (except /404.html)."""
    import re

    internal_html = re.compile(r'href="(/[^"#?]*\.html)(?:[#?][^"]*)?"')
    failures: list[str] = []
    for f in html_files:
        for m in internal_html.finditer(f.read_text(encoding="utf-8")):
            href = m.group(1)
            if href == "/404.html":
                continue
            failures.append(f"{f}: -> {href}")
    assert not failures, "internal .html links:\n" + "\n".join(failures)
