"""A site relocated under a Pages sub-path keeps every internal link working."""
from __future__ import annotations

import re
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "scripts"))

from relocate import relocate  # noqa: E402

PREVIEW = "https://dijonkitchen.github.io/posse-bliki"
PREFIX = "/posse-bliki/"
URL_RE = re.compile(r'href="([^"#?]+)|url=([^"]+)"')


def test_relocated_links_resolve(site: Path, config: dict, tmp_path: Path) -> None:
    out = tmp_path / "public"
    shutil.copytree(site, out)
    relocate(out, config["base_url"], PREVIEW)

    failures: list[str] = []
    for f in sorted(out.rglob("*.html")):
        html = f.read_text(encoding="utf-8")
        for m in URL_RE.finditer(html):
            url = m.group(1) or m.group(2)
            if url.startswith(PREVIEW):
                url = url[len(PREVIEW):] or "/"
                url = PREFIX + url.lstrip("/")
            if not url.startswith("/") or url.startswith("//"):
                continue
            if not url.startswith(PREFIX):
                failures.append(f"{f.relative_to(out)} -> {url} (not under {PREFIX})")
                continue
            rel = url[len(PREFIX):]
            if not ((out / rel).is_file() or (out / rel / "index.html").is_file()):
                failures.append(f"{f.relative_to(out)} -> {url}")
    assert not failures, "broken links after relocation:\n" + "\n".join(failures[:50])


def test_relocated_feeds_use_new_base(site: Path, config: dict, tmp_path: Path) -> None:
    out = tmp_path / "public"
    shutil.copytree(site, out)
    relocate(out, config["base_url"], PREVIEW)
    for name in ("index.xml", "feed.json", "sitemap.xml"):
        text = (out / name).read_text(encoding="utf-8")
        assert config["base_url"].rstrip("/") + "/" not in text, name
        assert PREVIEW + "/" in text, name
