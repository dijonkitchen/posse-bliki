"""Portability / enshittification-resistance invariants. (spec/portability.md)

P2 (no JavaScript) lives in test_html_valid.py; JSON Feed full content
(half of P6) lives in test_jsonfeed.py.
"""
from __future__ import annotations

import re
from pathlib import Path

import feedparser
import pytest
import yaml

REPO_ROOT = Path(__file__).resolve().parent.parent
CONTENT_DIR = REPO_ROOT / "content"

# Hostnames of the default providers in build.default_config(). Building
# with ALT_CONFIG must leave no trace of them (P4): service URLs reach the
# output through config only, never through templates or build code.
DEFAULT_PROVIDER_HOSTS = ("example.com", "webmention.io")

ALT_CONFIG = {
    "title": "alt-bliki",
    "tagline": "Same vault, different providers.",
    "base_url": "https://newhome.example.net",
    "language": "en",
    "author": {
        "name": "Alt Author",
        "url": "https://newhome.example.net",
        "rel_me": ["https://codeberg.org/alt-author"],
    },
    "webmention": {
        "endpoint": "https://mentions.example.net/webmention",
        "pingback": "https://mentions.example.net/xmlrpc",
    },
}

STATIC_EXTENSIONS = {".html", ".xml", ".json", ".txt", ".css"}


@pytest.fixture(scope="module")
def alt_site(tmp_path_factory: pytest.TempPathFactory) -> Path:
    from build.build import build_site

    out = tmp_path_factory.mktemp("alt_public")
    build_site(content_dir=CONTENT_DIR, out_dir=out, config=ALT_CONFIG)
    return out


# --- P1: output is pure static files ---


def test_output_is_pure_static_files(site: Path) -> None:
    offenders = [
        p.relative_to(site).as_posix()
        for p in site.rglob("*")
        if p.is_file() and p.suffix not in STATIC_EXTENSIONS
    ]
    assert not offenders, f"non-static files in output: {offenders}"


# --- P3: no third-party active content ---


def test_no_third_party_active_content(html_files: list[Path]) -> None:
    failures: list[str] = []
    for f in html_files:
        html = f.read_text(encoding="utf-8")
        for tag in ("<iframe", "<embed", "<object"):
            if tag in html:
                failures.append(f"{f.name}: active content tag {tag}")
        for hint in ("preconnect", "dns-prefetch"):
            if hint in html:
                failures.append(f"{f.name}: resource hint {hint}")
        for href in re.findall(
            r'<link[^>]*rel="stylesheet"[^>]*href="([^"]*)"', html
        ):
            if not href.startswith("/") or href.startswith("//"):
                failures.append(f"{f.name}: non-root-relative stylesheet {href}")
    assert not failures, "\n".join(failures)


# --- P4: provider swap leaves zero residue ---


def test_provider_swap_leaves_no_residue(alt_site: Path) -> None:
    failures: list[str] = []
    for p in sorted(alt_site.rglob("*")):
        if not p.is_file():
            continue
        text = p.read_text(encoding="utf-8")
        for host in DEFAULT_PROVIDER_HOSTS:
            if host in text:
                failures.append(f"{p.relative_to(alt_site).as_posix()}: {host}")
    assert not failures, (
        "default-provider residue after config swap:\n" + "\n".join(failures)
    )


# --- P5: all absolute self-URLs derive from base_url ---


def test_self_urls_derive_from_base_url(alt_site: Path) -> None:
    base = ALT_CONFIG["base_url"]
    failures: list[str] = []

    for f in sorted(alt_site.rglob("*.html")):
        for href in re.findall(r'<link rel="canonical" href="([^"]*)"', f.read_text(encoding="utf-8")):
            if not href.startswith(base):
                failures.append(f"{f.relative_to(alt_site).as_posix()}: canonical {href}")

    parsed = feedparser.parse(str(alt_site / "index.xml"))
    if not parsed.feed.link.startswith(base):
        failures.append(f"index.xml: channel link {parsed.feed.link}")
    for entry in parsed.entries:
        for url in (entry.link, entry.id):
            if not url.startswith(base):
                failures.append(f"index.xml: item URL {url}")

    import json

    feed = json.loads((alt_site / "feed.json").read_text(encoding="utf-8"))
    for url in (feed["home_page_url"], feed["feed_url"]):
        if not url.startswith(base):
            failures.append(f"feed.json: {url}")
    for item in feed["items"]:
        for url in (item["id"], item["url"]):
            if not url.startswith(base):
                failures.append(f"feed.json: item URL {url}")

    for loc in re.findall(r"<loc>([^<]*)</loc>", (alt_site / "sitemap.xml").read_text(encoding="utf-8")):
        if not loc.startswith(base):
            failures.append(f"sitemap.xml: {loc}")

    for line in (alt_site / "robots.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith("Sitemap:") and base not in line:
            failures.append(f"robots.txt: {line}")

    assert not failures, "\n".join(failures)


# --- P7: content is portable plain markdown ---


def test_content_is_plain_markdown() -> None:
    offenders: list[str] = []
    for p in sorted(CONTENT_DIR.rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(CONTENT_DIR)
        if any(part.startswith(".") for part in rel.parts):
            continue
        if p.suffix != ".md":
            offenders.append(f"{rel.as_posix()}: not markdown")
            continue
        try:
            p.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            offenders.append(f"{rel.as_posix()}: not UTF-8")
    assert not offenders, "\n".join(offenders)


# --- P8: CI workflow is a thin `make ci` wrapper ---


def test_ci_workflow_is_thin_make_wrapper() -> None:
    workflow = yaml.safe_load(
        (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
    )
    run_steps = [
        step["run"].strip()
        for job in workflow["jobs"].values()
        for step in job["steps"]
        if "run" in step
    ]
    assert run_steps == ["make ci"], (
        f"CI must do nothing but `make ci`, found: {run_steps}"
    )
