"""Every content/*.md front-matter validates against spec/content-schema.json."""
from __future__ import annotations

import json
from pathlib import Path

import pytest
import yaml
from jsonschema import Draft202012Validator


class _StringDatesLoader(yaml.SafeLoader):
    """Match build/build.py: keep ISO dates as strings (per spec/content-schema.md)."""


_StringDatesLoader.yaml_implicit_resolvers = {
    k: [(t, r) for t, r in v if t != "tag:yaml.org,2002:timestamp"]
    for k, v in _StringDatesLoader.yaml_implicit_resolvers.items()
}

REPO = Path(__file__).resolve().parent.parent
CONTENT = REPO / "content"
SCHEMA_PATH = REPO / "spec" / "content-schema.json"


def _front_matter(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return {}
    _, fm, _ = text.split("---\n", 2)
    data = yaml.load(fm, Loader=_StringDatesLoader) or {}
    if not isinstance(data, dict):
        raise AssertionError(f"{path}: front-matter is not a mapping")
    return data


@pytest.fixture(scope="session")
def validator() -> Draft202012Validator:
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    Draft202012Validator.check_schema(schema)
    return Draft202012Validator(schema)


@pytest.mark.parametrize(
    "md_file",
    sorted(CONTENT.rglob("*.md")),
    ids=lambda p: str(p.relative_to(CONTENT)),
)
def test_frontmatter_validates(md_file: Path, validator: Draft202012Validator) -> None:
    fm = _front_matter(md_file)
    errors = sorted(validator.iter_errors(fm), key=lambda e: e.path)
    assert not errors, "\n".join(f"{list(e.path)}: {e.message}" for e in errors)


def test_posts_have_date() -> None:
    """Posts (anything in content/notes/) must have a date for h-entry dt-published."""
    for md in sorted((CONTENT / "notes").rglob("*.md")):
        fm = _front_matter(md)
        if fm.get("draft"):
            continue
        assert "date" in fm, f"{md.relative_to(CONTENT)} is a post but has no `date`"
