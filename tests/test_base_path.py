"""Built for a GitHub Pages project URL, every internal link stays under the sub-path."""
from __future__ import annotations

import re
import subprocess
from pathlib import Path

import pytest

from conftest import CONTENT_DIR, REPO_ROOT

PREVIEW = "https://dijonkitchen.github.io/posse-bliki"
PREFIX = "/posse-bliki/"
URL_RE = re.compile(r'href="([^"#?]+)|url=([^"]+)"')


@pytest.fixture(scope="module")
def sub_site(bliki: Path, tmp_path_factory: pytest.TempPathFactory) -> Path:
    out = tmp_path_factory.mktemp("sub")
    subprocess.run(
        [str(bliki), "--content", str(CONTENT_DIR), "--out", str(out), "--base-url", PREVIEW],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
    )
    return out


def test_links_resolve_under_sub_path(sub_site: Path) -> None:
    failures: list[str] = []
    for f in sorted(sub_site.rglob("*.html")):
        for m in URL_RE.finditer(f.read_text(encoding="utf-8")):
            url = m.group(1) or m.group(2)
            if url.startswith(PREVIEW):
                url = PREFIX + url[len(PREVIEW):].lstrip("/")
            if not url.startswith("/") or url.startswith("//"):
                continue
            if not url.startswith(PREFIX):
                failures.append(f"{f.relative_to(sub_site)} -> {url} (not under {PREFIX})")
                continue
            rel = url[len(PREFIX):]
            if not ((sub_site / rel).is_file() or (sub_site / rel / "index.html").is_file()):
                failures.append(f"{f.relative_to(sub_site)} -> {url}")
    assert not failures, "broken links under sub-path:\n" + "\n".join(failures[:50])


def test_feeds_use_base_url(sub_site: Path, config: dict) -> None:
    for name in ("index.xml", "feed.json", "sitemap.xml"):
        text = (sub_site / name).read_text(encoding="utf-8")
        assert config["base_url"].rstrip("/") + "/" not in text, name
        assert PREVIEW + "/" in text, name
    assert 'href=\\"/notes/' not in (sub_site / "feed.json").read_text(encoding="utf-8")
    assert "href=&quot;/notes/" not in (sub_site / "index.xml").read_text(encoding="utf-8")
