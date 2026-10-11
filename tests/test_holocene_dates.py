"""Dates people read show Holocene years; machine-readable dates stay ISO.

See spec/output-contract.md and docs/adr/0011-holocene-years-on-the-page.md.
"""
from __future__ import annotations

from pathlib import Path
import re

TIME = re.compile(r'<time class="dt-[a-z]+" datetime="(\d{4})(-\d{2}-\d{2})[^"]*">(?:updated )?([^<]*)</time>')


def test_visible_years_are_holocene(site: Path) -> None:
    seen = 0
    for f in site.rglob("*.html"):
        for year, rest, text in TIME.findall(f.read_text(encoding="utf-8")):
            assert text == f"{int(year) + 10000}{rest}", f"{f}: shows {text!r} for {year}{rest}"
            seen += 1
    assert seen, "no <time> elements found"


def test_machine_dates_stay_iso(site: Path) -> None:
    for name in ("index.xml", "feed.json", "sitemap.xml"):
        text = (site / name).read_text(encoding="utf-8")
        assert not re.search(r"\b1\d{4}-\d{2}-\d{2}\b|\b\d{2} [A-Z][a-z]{2} 1\d{4}\b", text), name
