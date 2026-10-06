"""Relocate a built site to the URL it is actually served from.

The build emits root-relative links (``/notes/``) and absolute URLs on the
configured ``base_url``. A GitHub Pages project site is served under a
sub-path (``https://<user>.github.io/<repo>/``), so the deploy workflow runs
this over ``public/`` to point both at the real address. With a custom domain
the sub-path is empty and only the absolute URLs change.

Stdlib only; independent of the build implementation.

    python scripts/relocate.py public --to https://dijonkitchen.github.io/posse-bliki
"""
from __future__ import annotations

import argparse
import re
from pathlib import Path
from urllib.parse import urlsplit

CANONICAL_RE = re.compile(r'<link rel="canonical" href="([^"]+)">')
TEXT_SUFFIXES = {".html", ".xml", ".json", ".txt"}

# Root-relative URL attributes, plain or escaped inside feed bodies, plus the
# meta-refresh target used by alias redirects. ``//host`` is left alone.
ROOT_REL_RE = re.compile(r'((?:href|src)=(?:"|&quot;|\\")|url=)/(?!/)')


def relocate_text(text: str, old_base: str, new_base: str) -> str:
    old_base = old_base.rstrip("/")
    new_base = new_base.rstrip("/")
    prefix = urlsplit(new_base).path.rstrip("/")
    if prefix:
        text = ROOT_REL_RE.sub(lambda m: f"{m.group(1)}{prefix}/", text)
    if old_base != new_base:
        text = text.replace(old_base, new_base)
    return text


def relocate(site: Path, old_base: str, new_base: str) -> int:
    changed = 0
    for path in sorted(site.rglob("*")):
        if not path.is_file() or path.suffix not in TEXT_SUFFIXES:
            continue
        before = path.read_text(encoding="utf-8")
        after = relocate_text(before, old_base, new_base)
        if after != before:
            path.write_text(after, encoding="utf-8")
            changed += 1
    return changed


def built_base_url(site: Path) -> str:
    """The base_url the site was built with, read from the home page canonical."""
    m = CANONICAL_RE.search((site / "index.html").read_text(encoding="utf-8"))
    if not m:
        raise SystemExit(f"no canonical link in {site / 'index.html'}")
    return m.group(1).rstrip("/")


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("site", type=Path, help="built site directory")
    p.add_argument("--from", dest="old_base", help="base_url the site was built with (default: read from index.html)")
    p.add_argument("--to", dest="new_base", required=True, help="URL the site is served from")
    args = p.parse_args()
    args.old_base = args.old_base or built_base_url(args.site)
    n = relocate(args.site, args.old_base, args.new_base)
    print(f"relocated {n} files: {args.old_base} -> {args.new_base}")


if __name__ == "__main__":
    main()
