"""Shared fixtures.

The build is the ``bliki`` binary (Rust, see docs/adr/0007). The harness
drives it as a subprocess: ``cargo build --release`` once per session, then
``target/release/bliki --content DIR --out DIR``. The ``site`` fixture builds
the site once per pytest session into a tmpdir and yields the output
directory. Tests then make assertions against the emitted files.
"""
from __future__ import annotations

from collections.abc import Callable
import json
from pathlib import Path
import subprocess

import pytest

REPO_ROOT = Path(__file__).resolve().parent.parent
CONTENT_DIR = REPO_ROOT / "content"
BINARY = REPO_ROOT / "target" / "release" / "bliki"


class BuildFailed(RuntimeError):
    """The binary exited non-zero; ``stderr`` holds its ``build error: ...`` text."""

    def __init__(self, returncode: int, stderr: str) -> None:
        super().__init__(f"bliki exited {returncode}: {stderr.strip()}")
        self.returncode = returncode
        self.stderr = stderr


@pytest.fixture(scope="session")
def bliki() -> Path:
    """Compile the build once per session and return the binary's path."""
    subprocess.run(
        ["cargo", "build", "--release", "--quiet"], cwd=REPO_ROOT, check=True
    )
    return BINARY


@pytest.fixture(scope="session")
def build_site(bliki: Path) -> Callable[[Path, Path], subprocess.CompletedProcess[str]]:
    """``build_site(content_dir, out_dir)`` runs the binary; raises BuildFailed on error."""

    def run(content_dir: Path, out_dir: Path) -> subprocess.CompletedProcess[str]:
        proc = subprocess.run(
            [str(bliki), "--content", str(content_dir), "--out", str(out_dir)],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise BuildFailed(proc.returncode, proc.stderr)
        return proc

    return run


@pytest.fixture(scope="session")
def config(bliki: Path) -> dict:
    out = subprocess.run(
        ["cargo", "run", "--release", "--quiet", "--", "--print-config"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return json.loads(out)


@pytest.fixture(scope="session")
def site(tmp_path_factory: pytest.TempPathFactory, build_site) -> Path:
    out = tmp_path_factory.mktemp("public")
    build_site(CONTENT_DIR, out)
    return out


@pytest.fixture(scope="session")
def html_files(site: Path) -> list[Path]:
    return sorted(site.rglob("*.html"))


@pytest.fixture(scope="session")
def post_html_files(site: Path) -> list[Path]:
    """index.html files for individual posts, excluding the /notes/ list page itself."""
    notes_dir = site / "notes"
    return sorted(p for p in notes_dir.rglob("index.html") if p.parent != notes_dir)
