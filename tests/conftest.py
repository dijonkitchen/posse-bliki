"""Shared fixtures.

The ``site`` fixture builds the site once per pytest session into a tmpdir
and yields the output directory. Tests then make assertions against the
emitted files.
"""
from __future__ import annotations

from pathlib import Path
import sys

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
CONTENT_DIR = REPO_ROOT / "content"

# Ensure `build` is importable without installing the project as a package.
sys.path.insert(0, str(REPO_ROOT))


@pytest.fixture(scope="session")
def config() -> dict:
    from build.build import default_config

    return default_config()


@pytest.fixture(scope="session")
def site(tmp_path_factory: pytest.TempPathFactory, config: dict) -> Path:
    from build.build import build_site

    out = tmp_path_factory.mktemp("public")
    build_site(content_dir=CONTENT_DIR, out_dir=out, config=config)
    return out


@pytest.fixture(scope="session")
def html_files(site: Path) -> list[Path]:
    return sorted(site.rglob("*.html"))


@pytest.fixture(scope="session")
def post_html_files(site: Path) -> list[Path]:
    """index.html files for individual posts, excluding the /notes/ list page itself."""
    notes_dir = site / "notes"
    return sorted(p for p in notes_dir.rglob("index.html") if p.parent != notes_dir)
