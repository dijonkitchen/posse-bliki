"""Front-matter: a strict YAML subset, plus validation against the schema.

Supported YAML: a flat mapping of ``key: value`` lines where each value is
a scalar (plain, 'single' or "double" quoted), a flow list ``[a, b]``, or a
block list of ``- item`` lines. Anything else (nested mappings, anchors,
block scalars, multi-line plain scalars) is a build error rather than a
silent misread. Dates stay strings, per ``spec/content-schema.md``.

Scalar typing follows YAML 1.1 (what PyYAML's SafeLoader does), so
``draft: yes`` is a boolean and ``title: 2020`` is an integer that the
schema then rejects.
"""
from __future__ import annotations

import re

__all__ = ["FrontMatterError", "parse", "validate"]


class FrontMatterError(ValueError):
    pass


_KEY_LINE_RE = re.compile(r"^([A-Za-z_][\w-]*)[ \t]*:(?:[ \t]+(.*?))?[ \t]*$")
_ITEM_LINE_RE = re.compile(r"^([ \t]*)-(?:[ \t]+(.*?))?[ \t]*$")

_NULL_RE = re.compile(r"^(?:~|null|Null|NULL|)$")
_BOOL_TRUE_RE = re.compile(r"^(?:yes|Yes|YES|true|True|TRUE|on|On|ON)$")
_BOOL_FALSE_RE = re.compile(r"^(?:no|No|NO|false|False|FALSE|off|Off|OFF)$")
_INT_RE = re.compile(r"^[-+]?(?:0|[1-9][0-9_]*)$")
_FLOAT_RE = re.compile(r"^[-+]?(?:[0-9][0-9_]*)?\.[0-9_]*(?:[eE][-+][0-9]+)?$|^[-+]?\.(?:inf|Inf|INF)$|^\.(?:nan|NaN|NAN)$")

_DQ_ESCAPES = {
    "0": "\0", "a": "\a", "b": "\b", "t": "\t", "\t": "\t", "n": "\n", "v": "\v",
    "f": "\f", "r": "\r", "e": "\x1b", " ": " ", '"': '"', "/": "/", "\\": "\\",
    "N": "\x85", "_": "\xa0", "L": " ", "P": " ",
}


def _strip_comment(s: str) -> str:
    m = re.search(r"[ \t]#", s)
    return s[: m.start()].rstrip() if m else s


def _double_quoted(s: str) -> str:
    out = []
    i = 1
    while i < len(s) - 1:
        ch = s[i]
        if ch == "\\":
            i += 1
            esc = s[i]
            width = {"x": 2, "u": 4, "U": 8}.get(esc)
            if width:
                out.append(chr(int(s[i + 1 : i + 1 + width], 16)))
                i += width
            elif esc in _DQ_ESCAPES:
                out.append(_DQ_ESCAPES[esc])
            else:
                raise FrontMatterError(f"unknown escape \\{esc}")
        else:
            out.append(ch)
        i += 1
    return "".join(out)


def _scalar(raw: str):
    s = raw.strip()
    if s[:1] == '"':
        if len(s) < 2 or s[-1] != '"' or re.search(r'(?<!\\)(?:\\\\)*"', s[1:-1]):
            raise FrontMatterError(f"malformed double-quoted string: {raw}")
        return _double_quoted(s)
    if s[:1] == "'":
        inner = s[1:-1]
        if len(s) < 2 or s[-1] != "'" or re.search(r"(?<!')'(?!')", inner.replace("''", "")):
            raise FrontMatterError(f"malformed single-quoted string: {raw}")
        return inner.replace("''", "'")
    s = _strip_comment(s)
    if s[:1] in ("&", "*", "!", "|", ">", "{", "%", "@", "`") or s[:2] in ("- ", "? "):
        raise FrontMatterError(f"unsupported YAML value: {raw}")
    if ": " in s or s.endswith(":"):
        raise FrontMatterError(f"unsupported YAML value: {raw}")
    if _NULL_RE.match(s):
        return None
    if _BOOL_TRUE_RE.match(s):
        return True
    if _BOOL_FALSE_RE.match(s):
        return False
    if _INT_RE.match(s):
        return int(s.replace("_", ""))
    if _FLOAT_RE.match(s) and re.search(r"[0-9]", s):
        return float(s.replace("_", ""))
    return s


def _split_flow(inner: str) -> list[str]:
    items: list[str] = []
    buf = ""
    quote = ""
    for ch in inner:
        if quote:
            buf += ch
            if ch == quote:
                quote = ""
        elif ch in "\"'":
            quote = ch
            buf += ch
        elif ch == ",":
            items.append(buf)
            buf = ""
        elif ch in "[]{}":
            raise FrontMatterError("nested flow collections are not supported")
        else:
            buf += ch
    if quote:
        raise FrontMatterError("unterminated quote in flow list")
    if buf.strip() or items:
        items.append(buf)
    if items and not items[-1].strip():
        items.pop()  # trailing comma
    return items


def _value(raw: str):
    s = raw.strip()
    if s.startswith("["):
        if not _strip_comment(s).endswith("]"):
            raise FrontMatterError(f"unterminated flow list: {raw}")
        return [_scalar(x) for x in _split_flow(_strip_comment(s)[1:-1])]
    return _scalar(s)


def parse(text: str) -> dict:
    """Parse the text between the ``---`` fences into a dict."""
    data: dict = {}
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip() or line.lstrip().startswith("#"):
            i += 1
            continue
        if line[:1] in (" ", "\t"):
            raise FrontMatterError(f"unexpected indented line: {line.strip()}")
        m = _KEY_LINE_RE.match(line)
        if not m:
            raise FrontMatterError(f"not a 'key: value' line: {line}")
        key, raw = m.group(1), m.group(2)
        i += 1
        if raw is not None and _strip_comment(raw.strip()) != "":
            data[key] = _value(raw)
            continue
        items = []
        while i < len(lines):
            nxt = lines[i]
            if not nxt.strip() or nxt.lstrip().startswith("#"):
                i += 1
                continue
            im = _ITEM_LINE_RE.match(nxt)
            if not im:
                break
            if im.group(2) is None:
                raise FrontMatterError("empty or nested list items are not supported")
            items.append(_scalar(im.group(2)))
            i += 1
        data[key] = items if items else None
    return data


# --- schema ---

_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}(:\d{2})?(Z|[+-]\d{2}:?\d{2})?)?$")
_TAG_RE = re.compile(r"^[a-z0-9][a-z0-9-]*$")
_ALIAS_RE = re.compile(r"^/.+")

_STRING_FIELDS = {"title", "date", "updated", "summary"}
_LIST_FIELDS = {"tags": _TAG_RE, "aliases": _ALIAS_RE, "syndication": None}
_ALLOWED = _STRING_FIELDS | set(_LIST_FIELDS) | {"draft"}


def validate(fm: dict) -> list[str]:
    """Check ``fm`` against ``spec/content-schema.json``. Returns error strings."""
    errs: list[str] = []
    for key in sorted(fm):
        if key not in _ALLOWED:
            errs.append(f"unexpected property {key!r}")
    if "title" not in fm:
        errs.append("'title' is a required property")
    for key in sorted(_STRING_FIELDS & fm.keys()):
        v = fm[key]
        if not isinstance(v, str):
            errs.append(f"[{key!r}]: {v!r} is not of type 'string'")
        elif key == "title" and not v:
            errs.append("['title']: should be non-empty")
        elif key in ("date", "updated") and not _DATE_RE.match(v):
            errs.append(f"[{key!r}]: {v!r} is not an ISO date")
    if "draft" in fm and not isinstance(fm["draft"], bool):
        errs.append(f"['draft']: {fm['draft']!r} is not of type 'boolean'")
    for key, pattern in _LIST_FIELDS.items():
        if key not in fm:
            continue
        v = fm[key]
        if not isinstance(v, list):
            errs.append(f"[{key!r}]: {v!r} is not of type 'array'")
            continue
        for i, item in enumerate(v):
            if not isinstance(item, str):
                errs.append(f"[{key!r}, {i}]: {item!r} is not of type 'string'")
            elif pattern is not None and not pattern.match(item):
                errs.append(f"[{key!r}, {i}]: {item!r} does not match {pattern.pattern!r}")
        if len({repr(x) for x in v}) != len(v):
            errs.append(f"[{key!r}]: has non-unique elements")
    return errs
