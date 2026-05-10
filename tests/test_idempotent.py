"""Build twice → byte-identical output. (build-invariants.md)"""
from __future__ import annotations

import hashlib
from pathlib import Path

CONTENT = Path(__file__).resolve().parent.parent / "content"


def _hash_tree(root: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for p in sorted(root.rglob("*")):
        if p.is_file():
            h = hashlib.sha256(p.read_bytes()).hexdigest()
            out[p.relative_to(root).as_posix()] = h
    return out


def test_idempotent(tmp_path: Path, config: dict) -> None:
    from build.build import build_site

    a = tmp_path / "a"
    b = tmp_path / "b"
    build_site(content_dir=CONTENT, out_dir=a, config=config)
    build_site(content_dir=CONTENT, out_dir=b, config=config)

    ha, hb = _hash_tree(a), _hash_tree(b)
    diff = {k: (ha.get(k), hb.get(k)) for k in set(ha) | set(hb) if ha.get(k) != hb.get(k)}
    assert not diff, f"build is not idempotent. Differences:\n{diff}"
