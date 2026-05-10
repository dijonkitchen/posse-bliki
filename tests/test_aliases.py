"""Aliases produce HTML redirect pages with both meta-refresh and rel=canonical."""
from __future__ import annotations

from pathlib import Path

import pytest


CONTENT = Path(__file__).resolve().parent.parent / "content"


@pytest.fixture(scope="module")
def aliased_site(tmp_path_factory: pytest.TempPathFactory) -> tuple[Path, str, str]:
    from build.build import build_site, default_config

    src = tmp_path_factory.mktemp("vault")
    for p in CONTENT.rglob("*"):
        rel = p.relative_to(CONTENT)
        dst = src / rel
        if p.is_dir():
            dst.mkdir(parents=True, exist_ok=True)
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            dst.write_bytes(p.read_bytes())

    note = src / "notes" / "moved-note.md"
    note.write_text(
        "---\n"
        "title: A moved note\n"
        "date: 2026-05-08\n"
        "aliases:\n"
        "  - /old-location/\n"
        "  - /even-older/\n"
        "---\n\n"
        "body\n",
        encoding="utf-8",
    )
    out = tmp_path_factory.mktemp("public-with-alias")
    build_site(content_dir=src, out_dir=out, config=default_config())
    return out, "/old-location/", "/notes/moved-note/"


def test_alias_files_exist(aliased_site: tuple[Path, str, str]) -> None:
    out, alias, _ = aliased_site
    redirect = out / "old-location" / "index.html"
    assert redirect.is_file(), f"alias {alias} did not produce a redirect file"


def test_alias_has_meta_refresh(aliased_site: tuple[Path, str, str]) -> None:
    out, _, target = aliased_site
    html = (out / "old-location" / "index.html").read_text(encoding="utf-8")
    assert 'http-equiv="refresh"' in html
    assert target in html


def test_alias_has_canonical(aliased_site: tuple[Path, str, str]) -> None:
    out, _, target = aliased_site
    html = (out / "old-location" / "index.html").read_text(encoding="utf-8")
    assert 'rel="canonical"' in html
    assert target in html
