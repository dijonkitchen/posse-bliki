"""Every internal href resolves to an emitted file."""
from __future__ import annotations

import re
from pathlib import Path

HREF_RE = re.compile(r'href="([^"#?]+)(?:[#?][^"]*)?"')


def _internal_hrefs(html: str) -> list[str]:
    out = []
    for m in HREF_RE.finditer(html):
        href = m.group(1)
        if href.startswith("/") and not href.startswith("//"):
            out.append(href)
    return out


def _resolve(site: Path, href: str) -> Path | None:
    """Map a URL path to the file that would be served for it."""
    rel = href.lstrip("/")
    candidates = []
    if rel == "":
        candidates.append(site / "index.html")
    else:
        candidates.append(site / rel)
        candidates.append(site / rel / "index.html")
        if not rel.endswith("/"):
            candidates.append(site / (rel + ".html"))
    for c in candidates:
        if c.is_file():
            return c
    return None


def test_no_broken_internal_links(html_files: list[Path], site: Path) -> None:
    failures: list[str] = []
    for f in html_files:
        html = f.read_text(encoding="utf-8")
        for href in _internal_hrefs(html):
            if href.endswith((".xml", ".json", ".txt", ".css")):
                if not _resolve(site, href):
                    failures.append(f"{f.relative_to(site)} -> {href}")
                continue
            if not _resolve(site, href):
                failures.append(f"{f.relative_to(site)} -> {href}")
    assert not failures, "broken internal links:\n" + "\n".join(failures)
