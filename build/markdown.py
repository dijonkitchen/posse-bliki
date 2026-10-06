"""Markdown → HTML, standard library only.

A small CommonMark + GFM-tables + strikethrough renderer modelled on the
markdown-it algorithm (the block rules walk line ranges; the inline rules
collect emphasis delimiters and balance them afterwards). Raw HTML is
always escaped, never passed through. Output for this vault is checked
byte-for-byte by the harness, so behaviour changes show up as test diffs.
"""
from __future__ import annotations

import html.entities
import re
import unicodedata

__all__ = ["render"]

MAX_NESTING = 20


# --- shared helpers ---


def _is_space(ch: str) -> bool:
    return ch == " " or ch == "\t"


_WHITESPACE = set("\t\n\x0b\x0c\r \xa0\u1680\u202f\u205f\u3000") | {
    chr(c) for c in range(0x2000, 0x200B)
}
_ASCII_PUNCT = set("!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~")


def _is_white(ch: str) -> bool:
    return ch in _WHITESPACE


def _is_punct(ch: str) -> bool:
    if ch in _ASCII_PUNCT:
        return True
    cat = unicodedata.category(ch)
    return cat[0] == "P" or cat[0] == "S"


def escape_html(s: str) -> str:
    if not any(c in s for c in '&<>"'):
        return s
    return (
        s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace('"', "&quot;")
    )


def _valid_entity_code(c: int) -> bool:
    if 0xD800 <= c <= 0xDFFF or 0xFDD0 <= c <= 0xFDEF:
        return False
    if (c & 0xFFFF) in (0xFFFF, 0xFFFE):
        return False
    if 0x00 <= c <= 0x08 or c == 0x0B or 0x0E <= c <= 0x1F or 0x7F <= c <= 0x9F:
        return False
    return c <= 0x10FFFF


def _decode_entity(name: str) -> str | None:
    """``name`` without ``&``/``;``. Returns the decoded text, or None."""
    if name.startswith("#"):
        num = name[1:]
        code = int(num[1:], 16) if num[:1] in ("x", "X") else int(num)
        return chr(code) if _valid_entity_code(code) else None
    return html.entities.html5.get(name + ";")


_UNESCAPE_RE = re.compile(
    r"\\([!\"#$%&'()*+,\-./:;<=>?@\[\\\]^_`{|}~])|&([a-z#][a-z0-9]{1,31});", re.I
)
_DIGITAL_ENTITY_RE = re.compile(r"^#((?:x[a-f0-9]{1,6}|[0-9]{1,7}))$", re.I)


def unescape_all(s: str) -> str:
    if "\\" not in s and "&" not in s:
        return s

    def repl(m: re.Match) -> str:
        if m.group(1):
            return m.group(1)
        name = m.group(2)
        if name.startswith("#"):
            if _DIGITAL_ENTITY_RE.match(name):
                decoded = _decode_entity(name)
                if decoded is not None:
                    return decoded
            return m.group(0)
        decoded = _decode_entity(name)
        return decoded if decoded is not None else m.group(0)

    return _UNESCAPE_RE.sub(repl, s)


def _normalize_reference(s: str) -> str:
    s = re.sub(r"\s+", " ", s.strip())
    # Unicode case folding, with the same special case markdown-it applies.
    return s.lower().upper().replace("ẞ", "SS").lower() if s else s


# --- URLs ---

_URL_SAFE = set(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789;/?:@&=+$,-_.!~*'()#"
)
_HEX2_RE = re.compile(r"[0-9a-fA-F]{2}")


def _encode_url(s: str) -> str:
    out = []
    i, n = 0, len(s)
    while i < n:
        ch = s[i]
        if ch == "%" and i + 2 < n and _HEX2_RE.fullmatch(s[i + 1 : i + 3]):
            out.append(s[i : i + 3])
            i += 3
            continue
        if ch in _URL_SAFE:
            out.append(ch)
        else:
            out.append("".join(f"%{b:02X}" for b in ch.encode("utf-8", "surrogatepass")))
        i += 1
    return "".join(out)


def _decode_url_text(s: str) -> str:
    """Percent-decode for display, keeping reserved characters encoded."""
    keep = set(";/?:@&=+$,#%")

    def repl(m: re.Match) -> str:
        raw = m.group(0)
        try:
            text = bytes.fromhex(raw.replace("%", "")).decode("utf-8")
        except UnicodeDecodeError:
            return raw
        return "".join(f"%{ord(c):02X}" if c in keep else c for c in text)

    return re.sub(r"(?:%[0-9a-fA-F]{2})+", repl, s)


_BAD_PROTO_RE = re.compile(r"^(vbscript|javascript|file|data):")
_GOOD_DATA_RE = re.compile(r"^data:image/(gif|png|jpeg|webp);")


def normalize_link(url: str) -> str:
    return _encode_url(url)


def normalize_link_text(url: str) -> str:
    return _decode_url_text(url)


def validate_link(url: str) -> bool:
    s = url.strip().lower()
    return bool(_GOOD_DATA_RE.match(s)) if _BAD_PROTO_RE.match(s) else True


def _parse_link_destination(s: str, pos: int, end: int) -> tuple[bool, int, str]:
    start = pos
    if pos < end and s[pos] == "<":
        pos += 1
        while pos < end:
            ch = s[pos]
            if ch == "\n" or ch == "<":
                return False, 0, ""
            if ch == ">":
                return True, pos + 1, unescape_all(s[start + 1 : pos])
            if ch == "\\" and pos + 1 < end:
                pos += 2
                continue
            pos += 1
        return False, 0, ""
    level = 0
    while pos < end:
        ch = s[pos]
        if ch == " ":
            break
        if ord(ch) < 0x20 or ord(ch) == 0x7F:
            break
        if ch == "\\" and pos + 1 < end:
            if s[pos + 1] == " ":
                break
            pos += 2
            continue
        if ch == "(":
            level += 1
            if level > 32:
                return False, 0, ""
        if ch == ")":
            if level == 0:
                break
            level -= 1
        pos += 1
    if start == pos or level != 0:
        return False, 0, ""
    return True, pos, unescape_all(s[start:pos])


def _parse_link_title(s: str, pos: int, end: int) -> tuple[bool, int, int, str]:
    """Returns (ok, pos, lines, title)."""
    start = pos
    lines = 0
    if pos >= end:
        return False, 0, 0, ""
    marker = s[pos]
    if marker not in "\"'(":
        return False, 0, 0, ""
    pos += 1
    if marker == "(":
        marker = ")"
    while pos < end:
        ch = s[pos]
        if ch == marker:
            return True, pos + 1, lines, unescape_all(s[start + 1 : pos])
        if ch == "(" and marker == ")":
            return False, 0, 0, ""
        if ch == "\n":
            lines += 1
        elif ch == "\\" and pos + 1 < end:
            pos += 1
            if s[pos] == "\n":
                lines += 1
        pos += 1
    return False, 0, 0, ""


def _parse_link_title_ext(s: str, pos: int, end: int) -> tuple[bool, int, str, bool]:
    """Like _parse_link_title, plus whether more lines could still close it."""
    if pos >= end or s[pos] not in "\"'(":
        return False, 0, "", False
    marker = ")" if s[pos] == "(" else s[pos]
    p = pos + 1
    while p < end:
        ch = s[p]
        if ch == marker:
            return True, p + 1, unescape_all(s[pos + 1 : p]), False
        if ch == "(" and marker == ")":
            return False, 0, "", False
        if ch == "\\" and p + 1 < end:
            p += 1
        p += 1
    return False, 0, "", True


# --- tokens ---


class Token:
    __slots__ = ("type", "tag", "nesting", "attrs", "level", "children", "content",
                 "markup", "info", "block", "hidden")

    def __init__(self, type_: str, tag: str, nesting: int) -> None:
        self.type = type_
        self.tag = tag
        self.nesting = nesting
        self.attrs: list[tuple[str, str]] = []
        self.level = 0
        self.children: list[Token] | None = None
        self.content = ""
        self.markup = ""
        self.info = ""
        self.block = False
        self.hidden = False


# --- block level ---


class _BlockState:
    def __init__(self, src: str, env: dict, tokens: list[Token]) -> None:
        self.src = src
        self.env = env
        self.tokens = tokens
        self.b_marks: list[int] = []
        self.e_marks: list[int] = []
        self.t_shift: list[int] = []
        self.s_count: list[int] = []
        self.bs_count: list[int] = []
        self.blk_indent = 0
        self.line = 0
        self.tight = False
        self.parent_type = "root"
        self.level = 0
        self.list_indent = -1

        start = indent = offset = 0
        indent_found = False
        n = len(src)
        for pos, ch in enumerate(src):
            if not indent_found:
                if _is_space(ch):
                    indent += 1
                    offset += 4 - offset % 4 if ch == "\t" else 1
                    continue
                indent_found = True
            if ch == "\n" or pos == n - 1:
                if ch != "\n":
                    pos += 1
                self.b_marks.append(start)
                self.e_marks.append(pos)
                self.t_shift.append(indent)
                self.s_count.append(offset)
                self.bs_count.append(0)
                indent_found = False
                indent = offset = 0
                start = pos + 1
        for arr, v in ((self.b_marks, n), (self.e_marks, n), (self.t_shift, 0),
                       (self.s_count, 0), (self.bs_count, 0)):
            arr.append(v)
        self.line_max = len(self.b_marks) - 1

    def ch(self, pos: int) -> str:
        return self.src[pos] if 0 <= pos < len(self.src) else ""

    def push(self, type_: str, tag: str, nesting: int) -> Token:
        t = Token(type_, tag, nesting)
        t.block = True
        if nesting < 0:
            self.level -= 1
        t.level = self.level
        if nesting > 0:
            self.level += 1
        self.tokens.append(t)
        return t

    def is_empty(self, line: int) -> bool:
        return self.b_marks[line] + self.t_shift[line] >= self.e_marks[line]

    def skip_empty_lines(self, line: int) -> int:
        while line < self.line_max:
            if self.b_marks[line] + self.t_shift[line] < self.e_marks[line]:
                break
            line += 1
        return line

    def skip_spaces(self, pos: int) -> int:
        while pos < len(self.src) and _is_space(self.src[pos]):
            pos += 1
        return pos

    def skip_chars(self, pos: int, ch: str) -> int:
        while pos < len(self.src) and self.src[pos] == ch:
            pos += 1
        return pos

    def skip_spaces_back(self, pos: int, minimum: int) -> int:
        if pos <= minimum:
            return pos
        while pos > minimum:
            pos -= 1
            if not _is_space(self.src[pos]):
                return pos + 1
        return pos

    def skip_chars_back(self, pos: int, ch: str, minimum: int) -> int:
        if pos <= minimum:
            return pos
        while pos > minimum:
            pos -= 1
            if self.src[pos] != ch:
                return pos + 1
        return pos

    def get_lines(self, begin: int, end: int, indent: int, keep_last_lf: bool) -> str:
        if begin >= end:
            return ""
        out = []
        for line in range(begin, end):
            line_indent = 0
            line_start = first = self.b_marks[line]
            last = self.e_marks[line] + 1 if (line + 1 < end or keep_last_lf) else self.e_marks[line]
            while first < last and line_indent < indent:
                ch = self.src[first] if first < len(self.src) else ""
                if _is_space(ch):
                    if ch == "\t":
                        line_indent += 4 - (line_indent + self.bs_count[line]) % 4
                    else:
                        line_indent += 1
                elif first - line_start < self.t_shift[line]:
                    line_indent += 1
                else:
                    break
                first += 1
            if line_indent > indent:
                out.append(" " * (line_indent - indent) + self.src[first:last])
            else:
                out.append(self.src[first:last])
        return "".join(out)

    def get_line(self, line: int) -> str:
        return self.src[self.b_marks[line] + self.t_shift[line] : self.e_marks[line]]


def _tokenize_block(state: _BlockState, start_line: int, end_line: int) -> None:
    line = start_line
    has_empty_lines = False
    while line < end_line:
        state.line = line = state.skip_empty_lines(line)
        if line >= end_line:
            break
        if state.s_count[line] < state.blk_indent:
            break
        if state.level >= MAX_NESTING:
            state.line = end_line
            break
        for rule in _BLOCK_RULES:
            if rule(state, line, end_line, False):
                break
        state.tight = not has_empty_lines
        if state.is_empty(state.line - 1):
            has_empty_lines = True
        line = state.line
        if line < end_line and state.is_empty(line):
            has_empty_lines = True
            line += 1
            state.line = line


def _escaped_split(s: str) -> list[str]:
    result: list[str] = []
    current = ""
    last_pos = 0
    escaped = False
    for pos, ch in enumerate(s):
        if ch == "|":
            if not escaped:
                result.append(current + s[last_pos:pos])
                current = ""
                last_pos = pos + 1
            else:
                current += s[last_pos : pos - 1]
                last_pos = pos
        escaped = ch == "\\"
    result.append(current + s[last_pos:])
    return result


_ALIGN_RE = re.compile(r"^:?-+:?$")


def _rule_table(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    if start_line + 2 > end_line:
        return False
    next_line = start_line + 1
    if state.s_count[next_line] < state.blk_indent:
        return False
    if state.s_count[next_line] - state.blk_indent >= 4:
        return False
    pos = state.b_marks[next_line] + state.t_shift[next_line]
    if pos >= state.e_marks[next_line]:
        return False
    first = state.src[pos]
    pos += 1
    if first not in "|-:":
        return False
    if pos >= state.e_marks[next_line]:
        return False
    second = state.src[pos]
    pos += 1
    if second not in "|-:" and not _is_space(second):
        return False
    if first == "-" and _is_space(second):
        return False
    while pos < state.e_marks[next_line]:
        ch = state.src[pos]
        if ch not in "|-:" and not _is_space(ch):
            return False
        pos += 1

    columns = state.get_line(start_line + 1).split("|")
    aligns: list[str] = []
    for i, t in enumerate(columns):
        t = t.strip()
        if not t:
            if i == 0 or i == len(columns) - 1:
                continue
            return False
        if not _ALIGN_RE.match(t):
            return False
        if t[-1] == ":":
            aligns.append("center" if t[0] == ":" else "right")
        elif t[0] == ":":
            aligns.append("left")
        else:
            aligns.append("")

    line_text = state.get_line(start_line).strip()
    if "|" not in line_text:
        return False
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    columns = _escaped_split(line_text)
    if columns and columns[0] == "":
        columns.pop(0)
    if columns and columns[-1] == "":
        columns.pop()
    column_count = len(columns)
    if column_count == 0 or column_count != len(aligns):
        return False
    if silent:
        return True

    old_parent = state.parent_type
    state.parent_type = "table"
    terminators = _TERMINATORS["blockquote"]

    def cell(tag: str, i: int, text: str) -> None:
        t = state.push(f"{tag}_open", tag, 1)
        if aligns[i]:
            t.attrs = [("style", f"text-align:{aligns[i]}")]
        t = state.push("inline", "", 0)
        t.content = text.strip()
        t.children = []
        state.push(f"{tag}_close", tag, -1)

    state.push("table_open", "table", 1)
    state.push("thead_open", "thead", 1)
    state.push("tr_open", "tr", 1)
    for i, col in enumerate(columns):
        cell("th", i, col)
    state.push("tr_close", "tr", -1)
    state.push("thead_close", "thead", -1)

    has_body = False
    next_line = start_line + 2
    while next_line < end_line:
        if state.s_count[next_line] < state.blk_indent:
            break
        if any(rule(state, next_line, end_line, True) for rule in terminators):
            break
        line_text = state.get_line(next_line).strip()
        if not line_text:
            break
        if state.s_count[next_line] - state.blk_indent >= 4:
            break
        columns = _escaped_split(line_text)
        if columns and columns[0] == "":
            columns.pop(0)
        if columns and columns[-1] == "":
            columns.pop()
        if not has_body:
            state.push("tbody_open", "tbody", 1)
            has_body = True
        state.push("tr_open", "tr", 1)
        for i in range(column_count):
            cell("td", i, columns[i] if i < len(columns) else "")
        state.push("tr_close", "tr", -1)
        next_line += 1
    if has_body:
        state.push("tbody_close", "tbody", -1)
    state.push("table_close", "table", -1)
    state.parent_type = old_parent
    state.line = next_line
    return True


def _rule_code(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    if state.s_count[start_line] - state.blk_indent < 4:
        return False
    next_line = last = start_line + 1
    while next_line < end_line:
        if state.is_empty(next_line):
            next_line += 1
            continue
        if state.s_count[next_line] - state.blk_indent >= 4:
            next_line += 1
            last = next_line
            continue
        break
    state.line = last
    t = state.push("code_block", "code", 0)
    t.content = state.get_lines(start_line, last, 4 + state.blk_indent, False) + "\n"
    return True


def _rule_fence(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    pos = state.b_marks[start_line] + state.t_shift[start_line]
    maximum = state.e_marks[start_line]
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    if pos + 3 > maximum:
        return False
    marker = state.src[pos]
    if marker not in "~`":
        return False
    mem = pos
    pos = state.skip_chars(pos, marker)
    length = pos - mem
    if length < 3:
        return False
    markup = state.src[mem:pos]
    params = state.src[pos:maximum]
    if marker == "`" and "`" in params:
        return False
    if silent:
        return True

    next_line = start_line
    have_end = False
    while True:
        next_line += 1
        if next_line >= end_line:
            break
        pos = mem = state.b_marks[next_line] + state.t_shift[next_line]
        maximum = state.e_marks[next_line]
        if pos < maximum and state.s_count[next_line] < state.blk_indent:
            break
        if pos >= len(state.src):
            break
        if state.src[pos] != marker:
            continue
        if state.s_count[next_line] - state.blk_indent >= 4:
            continue
        pos = state.skip_chars(pos, marker)
        if pos - mem < length:
            continue
        pos = state.skip_spaces(pos)
        if pos < maximum:
            continue
        have_end = True
        break

    length = state.s_count[start_line]
    state.line = next_line + (1 if have_end else 0)
    t = state.push("fence", "code", 0)
    t.info = params
    t.content = state.get_lines(start_line + 1, next_line, length, True)
    t.markup = markup
    return True


def _rule_blockquote(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    pos = state.b_marks[start_line] + state.t_shift[start_line]
    maximum = state.e_marks[start_line]
    old_line_max = state.line_max
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    if state.ch(pos) != ">":
        return False
    if silent:
        return True

    old_b_marks: list[int] = []
    old_bs_count: list[int] = []
    old_s_count: list[int] = []
    old_t_shift: list[int] = []
    terminators = _TERMINATORS["blockquote"]
    old_parent = state.parent_type
    state.parent_type = "blockquote"
    last_line_empty = False

    next_line = start_line
    while next_line < end_line:
        is_outdented = state.s_count[next_line] < state.blk_indent
        pos = state.b_marks[next_line] + state.t_shift[next_line]
        maximum = state.e_marks[next_line]
        if pos >= maximum:
            break
        if state.src[pos] == ">" and not is_outdented:
            pos += 1
            initial = state.s_count[next_line] + 1
            if state.ch(pos) == " ":
                pos += 1
                initial += 1
                adjust_tab = False
                space_after = True
            elif state.ch(pos) == "\t":
                space_after = True
                if (state.bs_count[next_line] + initial) % 4 == 3:
                    pos += 1
                    initial += 1
                    adjust_tab = False
                else:
                    adjust_tab = True
            else:
                space_after = False
                adjust_tab = False
            offset = initial
            old_b_marks.append(state.b_marks[next_line])
            state.b_marks[next_line] = pos
            while pos < maximum:
                ch = state.src[pos]
                if _is_space(ch):
                    if ch == "\t":
                        offset += 4 - (offset + state.bs_count[next_line] + (1 if adjust_tab else 0)) % 4
                    else:
                        offset += 1
                else:
                    break
                pos += 1
            last_line_empty = pos >= maximum
            old_bs_count.append(state.bs_count[next_line])
            state.bs_count[next_line] = state.s_count[next_line] + 1 + (1 if space_after else 0)
            old_s_count.append(state.s_count[next_line])
            state.s_count[next_line] = offset - initial
            old_t_shift.append(state.t_shift[next_line])
            state.t_shift[next_line] = pos - state.b_marks[next_line]
            next_line += 1
            continue

        if last_line_empty:
            break
        if any(rule(state, next_line, end_line, True) for rule in terminators):
            state.line_max = next_line
            if state.blk_indent != 0:
                old_b_marks.append(state.b_marks[next_line])
                old_bs_count.append(state.bs_count[next_line])
                old_t_shift.append(state.t_shift[next_line])
                old_s_count.append(state.s_count[next_line])
                state.s_count[next_line] -= state.blk_indent
            break
        old_b_marks.append(state.b_marks[next_line])
        old_bs_count.append(state.bs_count[next_line])
        old_t_shift.append(state.t_shift[next_line])
        old_s_count.append(state.s_count[next_line])
        # Negative indentation marks a lazy paragraph continuation.
        state.s_count[next_line] = -1
        next_line += 1

    old_indent = state.blk_indent
    state.blk_indent = 0
    state.push("blockquote_open", "blockquote", 1)
    _tokenize_block(state, start_line, next_line)
    state.push("blockquote_close", "blockquote", -1)
    state.line_max = old_line_max
    state.parent_type = old_parent
    for i in range(len(old_t_shift)):
        state.b_marks[i + start_line] = old_b_marks[i]
        state.t_shift[i + start_line] = old_t_shift[i]
        state.s_count[i + start_line] = old_s_count[i]
        state.bs_count[i + start_line] = old_bs_count[i]
    state.blk_indent = old_indent
    return True


def _rule_hr(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    pos = state.b_marks[start_line] + state.t_shift[start_line]
    maximum = state.e_marks[start_line]
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    marker = state.ch(pos)
    if not marker or marker not in "*-_":
        return False
    cnt = 1
    pos += 1
    while pos < maximum:
        ch = state.src[pos]
        pos += 1
        if ch != marker and not _is_space(ch):
            return False
        if ch == marker:
            cnt += 1
    if cnt < 3:
        return False
    if silent:
        return True
    state.line = start_line + 1
    t = state.push("hr", "hr", 0)
    t.markup = marker * (cnt + 1)
    return True


def _skip_bullet_marker(state: _BlockState, line: int) -> int:
    pos = state.b_marks[line] + state.t_shift[line]
    maximum = state.e_marks[line]
    if pos >= maximum:
        return -1
    marker = state.src[pos]
    pos += 1
    if marker not in "*-+":
        return -1
    if pos < maximum and not _is_space(state.src[pos]):
        return -1
    return pos


def _skip_ordered_marker(state: _BlockState, line: int) -> int:
    start = pos = state.b_marks[line] + state.t_shift[line]
    maximum = state.e_marks[line]
    if pos + 1 >= maximum:
        return -1
    ch = state.src[pos]
    pos += 1
    if not "0" <= ch <= "9":
        return -1
    while True:
        if pos >= maximum:
            return -1
        ch = state.src[pos]
        pos += 1
        if "0" <= ch <= "9":
            if pos - start >= 10:
                return -1
            continue
        if ch in ").":
            break
        return -1
    if pos < maximum and not _is_space(state.src[pos]):
        return -1
    return pos


def _mark_tight_paragraphs(state: _BlockState, idx: int) -> None:
    level = state.level + 2
    i = idx + 2
    n = len(state.tokens) - 2
    while i < n:
        if state.tokens[i].level == level and state.tokens[i].type == "paragraph_open":
            state.tokens[i + 2].hidden = True
            state.tokens[i].hidden = True
            i += 2
        i += 1


def _rule_list(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    is_terminating_paragraph = False
    tight = True
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    if (
        state.list_indent >= 0
        and state.s_count[start_line] - state.list_indent >= 4
        and state.s_count[start_line] < state.blk_indent
    ):
        return False
    if silent and state.parent_type == "paragraph":
        if state.s_count[start_line] >= state.blk_indent:
            is_terminating_paragraph = True

    pos_after = _skip_ordered_marker(state, start_line)
    if pos_after >= 0:
        is_ordered = True
        start = state.b_marks[start_line] + state.t_shift[start_line]
        marker_value = int(state.src[start : pos_after - 1])
        if is_terminating_paragraph and marker_value != 1:
            return False
    else:
        pos_after = _skip_bullet_marker(state, start_line)
        if pos_after < 0:
            return False
        is_ordered = False
        marker_value = 1

    if is_terminating_paragraph:
        if state.skip_spaces(pos_after) >= state.e_marks[start_line]:
            return False
    if silent:
        return True

    marker_char = state.src[pos_after - 1]
    list_tok_idx = len(state.tokens)
    if is_ordered:
        t = state.push("ordered_list_open", "ol", 1)
        if marker_value != 1:
            t.attrs = [("start", str(marker_value))]
    else:
        t = state.push("bullet_list_open", "ul", 1)
    t.markup = marker_char

    next_line = start_line
    prev_empty_end = False
    terminators = _TERMINATORS["list"]
    old_parent = state.parent_type
    state.parent_type = "list"

    while next_line < end_line:
        pos = pos_after
        maximum = state.e_marks[next_line]
        initial = offset = (
            state.s_count[next_line]
            + pos_after
            - (state.b_marks[next_line] + state.t_shift[next_line])
        )
        while pos < maximum:
            ch = state.src[pos]
            if ch == "\t":
                offset += 4 - (offset + state.bs_count[next_line]) % 4
            elif ch == " ":
                offset += 1
            else:
                break
            pos += 1
        content_start = pos
        indent_after = 1 if content_start >= maximum else offset - initial
        if indent_after > 4:
            indent_after = 1
        indent = initial + indent_after

        t = state.push("list_item_open", "li", 1)
        t.markup = marker_char

        old_tight = state.tight
        old_t_shift = state.t_shift[next_line]
        old_s_count = state.s_count[next_line]
        old_list_indent = state.list_indent
        state.list_indent = state.blk_indent
        state.blk_indent = indent
        state.tight = True
        state.t_shift[next_line] = content_start - state.b_marks[next_line]
        state.s_count[next_line] = offset

        if content_start >= maximum and state.is_empty(next_line + 1):
            # Empty item followed by a blank line: the list ends here.
            state.line = min(state.line + 2, end_line)
        else:
            _tokenize_block(state, next_line, end_line)

        if not state.tight or prev_empty_end:
            tight = False
        prev_empty_end = (state.line - start_line) > 1 and state.is_empty(state.line - 1)

        state.blk_indent = state.list_indent
        state.list_indent = old_list_indent
        state.t_shift[next_line] = old_t_shift
        state.s_count[next_line] = old_s_count
        state.tight = old_tight

        state.push("list_item_close", "li", -1)

        next_line = start_line = state.line
        if next_line >= end_line:
            break
        if state.s_count[next_line] < state.blk_indent:
            break
        if state.s_count[start_line] - state.blk_indent >= 4:
            break
        if any(rule(state, next_line, end_line, True) for rule in terminators):
            break
        if is_ordered:
            pos_after = _skip_ordered_marker(state, next_line)
        else:
            pos_after = _skip_bullet_marker(state, next_line)
        if pos_after < 0:
            break
        if marker_char != state.src[pos_after - 1]:
            break

    state.push("ordered_list_close" if is_ordered else "bullet_list_close",
               "ol" if is_ordered else "ul", -1)
    state.line = next_line
    state.parent_type = old_parent
    if tight:
        _mark_tight_paragraphs(state, list_tok_idx)
    return True


def _rule_reference(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    pos = state.b_marks[start_line] + state.t_shift[start_line]
    maximum = state.e_marks[start_line]
    next_line = start_line + 1
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    if state.ch(pos) != "[":
        return False

    def get_next_line(line: int) -> str | None:
        if line >= state.line_max or state.is_empty(line):
            return None
        is_continuation = False
        if state.s_count[line] - state.blk_indent > 3:
            is_continuation = True
        if state.s_count[line] < 0:
            is_continuation = True
        if not is_continuation:
            old_parent = state.parent_type
            state.parent_type = "reference"
            terminate = any(r(state, line, state.line_max, True) for r in _TERMINATORS["reference"])
            state.parent_type = old_parent
            if terminate:
                return None
        p = state.b_marks[line] + state.t_shift[line]
        return state.src[p : state.e_marks[line] + 1]

    s = state.src[pos : maximum + 1]
    n = len(s)
    label_end = -1
    p = 1
    while p < n:
        ch = s[p]
        if ch == "[":
            return False
        if ch == "]":
            label_end = p
            break
        if ch == "\n":
            nxt = get_next_line(next_line)
            if nxt is not None:
                s += nxt
                n = len(s)
                next_line += 1
        elif ch == "\\":
            p += 1
            if p < n and s[p] == "\n":
                nxt = get_next_line(next_line)
                if nxt is not None:
                    s += nxt
                    n = len(s)
                    next_line += 1
        p += 1
    if label_end < 0 or label_end + 1 >= n or s[label_end + 1] != ":":
        return False

    p = label_end + 2
    while p < n:
        ch = s[p]
        if ch == "\n":
            nxt = get_next_line(next_line)
            if nxt is not None:
                s += nxt
                n = len(s)
                next_line += 1
        elif _is_space(ch):
            pass
        else:
            break
        p += 1

    ok, res_pos, res_str = _parse_link_destination(s, p, n)
    if not ok:
        return False
    href = normalize_link(res_str)
    if not validate_link(href):
        return False
    p = res_pos
    dest_end = p
    dest_end_line = next_line
    start = p
    while p < n:
        ch = s[p]
        if ch == "\n":
            nxt = get_next_line(next_line)
            if nxt is not None:
                s += nxt
                n = len(s)
                next_line += 1
        elif _is_space(ch):
            pass
        else:
            break
        p += 1

    # A quoted title may span lines; keep pulling lines in until it closes.
    ok_t, t_pos, t_str, can_continue = _parse_link_title_ext(s, p, n)
    while not ok_t and can_continue:
        nxt = get_next_line(next_line)
        if nxt is None:
            break
        s += nxt
        n = len(s)
        next_line += 1
        ok_t, t_pos, t_str, can_continue = _parse_link_title_ext(s, p, n)
    if p < n and start != p and ok_t:
        title = t_str
        p = t_pos
    else:
        title = ""
        p = dest_end
        next_line = dest_end_line

    while p < n and _is_space(s[p]):
        p += 1
    if p < n and s[p] != "\n":
        if title:
            title = ""
            p = dest_end
            next_line = dest_end_line
            while p < n and _is_space(s[p]):
                p += 1
    if p < n and s[p] != "\n":
        return False

    label = _normalize_reference(s[1:label_end])
    if not label:
        return False
    if silent:
        return True
    refs = state.env.setdefault("references", {})
    refs.setdefault(label, {"title": title, "href": href})
    state.line = next_line
    return True


def _rule_heading(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    pos = state.b_marks[start_line] + state.t_shift[start_line]
    maximum = state.e_marks[start_line]
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    if pos >= maximum or state.src[pos] != "#":
        return False
    level = 1
    pos += 1
    ch = state.ch(pos)
    while ch == "#" and pos < maximum and level <= 6:
        level += 1
        pos += 1
        ch = state.ch(pos)
    if level > 6 or (pos < maximum and not _is_space(ch)):
        return False
    if silent:
        return True
    maximum = state.skip_spaces_back(maximum, pos)
    tmp = state.skip_chars_back(maximum, "#", pos)
    if tmp > pos and _is_space(state.src[tmp - 1]):
        maximum = tmp
    state.line = start_line + 1
    t = state.push("heading_open", f"h{level}", 1)
    t.markup = "#" * level
    t = state.push("inline", "", 0)
    t.content = state.src[pos:maximum].strip()
    t.children = []
    state.push("heading_close", f"h{level}", -1)
    return True


def _rule_lheading(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    terminators = _TERMINATORS["paragraph"]
    if state.s_count[start_line] - state.blk_indent >= 4:
        return False
    old_parent = state.parent_type
    state.parent_type = "paragraph"
    level = 0
    marker = ""
    next_line = start_line + 1
    while next_line < end_line and not state.is_empty(next_line):
        if state.s_count[next_line] - state.blk_indent > 3:
            next_line += 1
            continue
        if state.s_count[next_line] >= state.blk_indent:
            pos = state.b_marks[next_line] + state.t_shift[next_line]
            maximum = state.e_marks[next_line]
            if pos < maximum:
                marker = state.src[pos]
                if marker in "-=":
                    pos = state.skip_chars(pos, marker)
                    pos = state.skip_spaces(pos)
                    if pos >= maximum:
                        level = 1 if marker == "=" else 2
                        break
        if state.s_count[next_line] < 0:
            next_line += 1
            continue
        if any(rule(state, next_line, end_line, True) for rule in terminators):
            break
        next_line += 1
    if not level:
        return False
    content = state.get_lines(start_line, next_line, state.blk_indent, False).strip()
    state.line = next_line + 1
    t = state.push("heading_open", f"h{level}", 1)
    t.markup = marker
    t = state.push("inline", "", 0)
    t.content = content
    t.children = []
    state.push("heading_close", f"h{level}", -1)
    state.parent_type = old_parent
    return True


def _rule_paragraph(state: _BlockState, start_line: int, end_line: int, silent: bool) -> bool:
    terminators = _TERMINATORS["paragraph"]
    end_line = state.line_max
    next_line = start_line + 1
    old_parent = state.parent_type
    state.parent_type = "paragraph"
    while next_line < end_line and not state.is_empty(next_line):
        if state.s_count[next_line] - state.blk_indent > 3:
            next_line += 1
            continue
        if state.s_count[next_line] < 0:
            next_line += 1
            continue
        if any(rule(state, next_line, end_line, True) for rule in terminators):
            break
        next_line += 1
    content = state.get_lines(start_line, next_line, state.blk_indent, False).strip()
    state.line = next_line
    state.push("paragraph_open", "p", 1)
    t = state.push("inline", "", 0)
    t.content = content
    t.children = []
    state.push("paragraph_close", "p", -1)
    state.parent_type = old_parent
    return True


_BLOCK_RULES = [
    _rule_table, _rule_code, _rule_fence, _rule_blockquote, _rule_hr, _rule_list,
    _rule_reference, _rule_heading, _rule_lheading, _rule_paragraph,
]
_TERMINATORS = {
    "paragraph": [_rule_table, _rule_fence, _rule_blockquote, _rule_hr, _rule_list, _rule_heading],
    "reference": [_rule_table, _rule_fence, _rule_blockquote, _rule_hr, _rule_list, _rule_heading],
    "blockquote": [_rule_fence, _rule_blockquote, _rule_hr, _rule_list, _rule_heading],
    "list": [_rule_fence, _rule_blockquote, _rule_hr],
}


# --- inline level ---


class _Delim:
    __slots__ = ("marker", "length", "token", "end", "open", "close")

    def __init__(self, marker: str, length: int, token: int, can_open: bool, can_close: bool):
        self.marker = marker
        self.length = length
        self.token = token
        self.end = -1
        self.open = can_open
        self.close = can_close


class _InlineState:
    def __init__(self, src: str, env: dict, tokens: list[Token]) -> None:
        self.src = src
        self.env = env
        self.tokens = tokens
        self.tokens_meta: list[list[_Delim] | None] = []
        self.pos = 0
        self.pos_max = len(src)
        self.level = 0
        self.pending = ""
        self.pending_level = 0
        self.cache: dict[int, int] = {}
        self.delimiters: list[_Delim] = []
        self.prev_delimiters: list[list[_Delim]] = []
        self.backticks: dict[int, int] = {}
        self.backticks_scanned = False
        self.link_level = 0

    def ch(self, pos: int) -> str:
        return self.src[pos] if 0 <= pos < len(self.src) else ""

    def push_pending(self) -> Token:
        t = Token("text", "", 0)
        t.content = self.pending
        t.level = self.pending_level
        self.tokens.append(t)
        self.tokens_meta.append(None)
        self.pending = ""
        return t

    def push(self, type_: str, tag: str, nesting: int) -> Token:
        if self.pending:
            self.push_pending()
        t = Token(type_, tag, nesting)
        meta = None
        if nesting < 0:
            self.level -= 1
            self.delimiters = self.prev_delimiters.pop()
        t.level = self.level
        if nesting > 0:
            self.level += 1
            self.prev_delimiters.append(self.delimiters)
            self.delimiters = []
            meta = self.delimiters
        self.pending_level = self.level
        self.tokens.append(t)
        self.tokens_meta.append(meta)
        return t

    def scan_delims(self, start: int, can_split_word: bool) -> tuple[bool, bool, int]:
        marker = self.src[start]
        last_char = self.src[start - 1] if start > 0 else " "
        pos = start
        while pos < self.pos_max and self.src[pos] == marker:
            pos += 1
        count = pos - start
        next_char = self.src[pos] if pos < self.pos_max else " "
        last_punct = _is_punct(last_char)
        next_punct = _is_punct(next_char)
        last_white = _is_white(last_char)
        next_white = _is_white(next_char)
        left = not next_white and (not next_punct or last_white or last_punct)
        right = not last_white and (not last_punct or next_white or next_punct)
        can_open = left and (can_split_word or not right or last_punct)
        can_close = right and (can_split_word or not left or next_punct)
        return can_open, can_close, count


_TERMINATOR_CHARS = set("\n!#$%&*+-:<=>@[\\]^_`{}~")


def _inline_text(state: _InlineState, silent: bool) -> bool:
    pos = state.pos
    while pos < state.pos_max and state.src[pos] not in _TERMINATOR_CHARS:
        pos += 1
    if pos == state.pos:
        return False
    if not silent:
        state.pending += state.src[state.pos : pos]
    state.pos = pos
    return True


def _inline_newline(state: _InlineState, silent: bool) -> bool:
    pos = state.pos
    if state.src[pos] != "\n":
        return False
    pmax = len(state.pending) - 1
    if not silent:
        if pmax >= 0 and state.pending[pmax] == " ":
            if pmax >= 1 and state.pending[pmax - 1] == " ":
                ws = pmax - 1
                while ws >= 1 and state.pending[ws - 1] == " ":
                    ws -= 1
                state.pending = state.pending[:ws]
                state.push("hardbreak", "br", 0)
            else:
                state.pending = state.pending[:-1]
                state.push("softbreak", "br", 0)
        else:
            state.push("softbreak", "br", 0)
    pos += 1
    while pos < state.pos_max and _is_space(state.src[pos]):
        pos += 1
    state.pos = pos
    return True


_ESCAPABLE = set("\\!\"#$%&'()*+,./:;<=>?@[]^_`{|}~-")


def _inline_escape(state: _InlineState, silent: bool) -> bool:
    pos = state.pos
    if state.src[pos] != "\\":
        return False
    pos += 1
    if pos >= state.pos_max:
        return False
    ch = state.src[pos]
    if ch == "\n":
        if not silent:
            state.push("hardbreak", "br", 0)
        pos += 1
        while pos < state.pos_max and _is_space(state.src[pos]):
            pos += 1
        state.pos = pos
        return True
    if not silent:
        t = state.push("text_special", "", 0)
        t.content = ch if ch in _ESCAPABLE else "\\" + ch
    state.pos = pos + 1
    return True


def _inline_backticks(state: _InlineState, silent: bool) -> bool:
    pos = state.pos
    if state.src[pos] != "`":
        return False
    start = pos
    pos += 1
    maximum = state.pos_max
    while pos < maximum and state.src[pos] == "`":
        pos += 1
    marker = state.src[start:pos]
    opener_len = len(marker)
    if state.backticks_scanned and state.backticks.get(opener_len, 0) <= start:
        if not silent:
            state.pending += marker
        state.pos += opener_len
        return True
    match_end = pos
    while True:
        match_start = state.src.find("`", match_end)
        if match_start == -1:
            break
        match_end = match_start + 1
        while match_end < maximum and state.src[match_end] == "`":
            match_end += 1
        closer_len = match_end - match_start
        if closer_len == opener_len:
            if not silent:
                t = state.push("code_inline", "code", 0)
                t.markup = marker
                content = state.src[pos:match_start].replace("\n", " ")
                if len(content) > 2 and content[0] == content[-1] == " " and content.strip(" "):
                    content = content[1:-1]
                t.content = content
            state.pos = match_end
            return True
        state.backticks[closer_len] = match_start
    state.backticks_scanned = True
    if not silent:
        state.pending += marker
    state.pos += opener_len
    return True


def _inline_strikethrough(state: _InlineState, silent: bool) -> bool:
    if silent:
        return False
    marker = state.src[state.pos]
    if marker != "~":
        return False
    can_open, can_close, length = state.scan_delims(state.pos, True)
    if length < 2:
        return False
    if length % 2:
        t = state.push("text", "", 0)
        t.content = marker
        length -= 1
    for _ in range(0, length, 2):
        t = state.push("text", "", 0)
        t.content = marker * 2
        state.delimiters.append(_Delim(marker, 0, len(state.tokens) - 1, can_open, can_close))
    state.pos += state.scan_delims(state.pos, True)[2]
    return True


def _inline_emphasis(state: _InlineState, silent: bool) -> bool:
    if silent:
        return False
    marker = state.src[state.pos]
    if marker not in "_*":
        return False
    can_open, can_close, length = state.scan_delims(state.pos, marker == "*")
    for _ in range(length):
        t = state.push("text", "", 0)
        t.content = marker
        state.delimiters.append(_Delim(marker, length, len(state.tokens) - 1, can_open, can_close))
    state.pos += length
    return True


def _parse_link_label(state: _InlineState, start: int, disable_nested: bool) -> int:
    old_pos = state.pos
    state.pos = start + 1
    level = 1
    found = False
    while state.pos < state.pos_max:
        marker = state.src[state.pos]
        if marker == "]":
            level -= 1
            if level == 0:
                found = True
                break
        prev_pos = state.pos
        _skip_token(state)
        if marker == "[":
            if prev_pos == state.pos - 1:
                level += 1
            elif disable_nested:
                state.pos = old_pos
                return -1
    label_end = state.pos if found else -1
    state.pos = old_pos
    return label_end


def _reference_target(state: _InlineState, label_start: int, label_end: int,
                      pos: int) -> tuple[int, dict | None]:
    refs = state.env.get("references")
    if refs is None:
        return pos, None
    maximum = state.pos_max
    label = None
    if pos < maximum and state.src[pos] == "[":
        start = pos + 1
        pos = _parse_link_label(state, pos, False)
        if pos >= 0:
            label = state.src[start:pos]
            pos += 1
        else:
            pos = label_end + 1
    else:
        pos = label_end + 1
    if not label:
        label = state.src[label_start:label_end]
    return pos, refs.get(_normalize_reference(label))


def _skip_ws(state: _InlineState, pos: int) -> int:
    while pos < state.pos_max and (_is_space(state.src[pos]) or state.src[pos] == "\n"):
        pos += 1
    return pos


def _inline_link(state: _InlineState, silent: bool) -> bool:
    href = ""
    title = ""
    old_pos = state.pos
    maximum = state.pos_max
    parse_reference = True
    if state.src[state.pos] != "[":
        return False
    label_start = state.pos + 1
    label_end = _parse_link_label(state, state.pos, True)
    if label_end < 0:
        return False
    pos = label_end + 1
    if pos < maximum and state.src[pos] == "(":
        parse_reference = False
        pos = _skip_ws(state, pos + 1)
        if pos >= maximum:
            return False
        ok, res_pos, res_str = _parse_link_destination(state.src, pos, state.pos_max)
        if ok:
            href = normalize_link(res_str)
            if validate_link(href):
                pos = res_pos
            else:
                href = ""
            start = pos
            pos = _skip_ws(state, pos)
            ok_t, t_pos, _l, t_str = _parse_link_title(state.src, pos, state.pos_max)
            if pos < maximum and start != pos and ok_t:
                title = t_str
                pos = _skip_ws(state, t_pos)
        if pos >= maximum or state.src[pos] != ")":
            parse_reference = True
        pos += 1
    if parse_reference:
        if "references" not in state.env:
            return False
        pos, ref = _reference_target(state, label_start, label_end, pos)
        if not ref:
            state.pos = old_pos
            return False
        href = ref["href"]
        title = ref["title"]
    if not silent:
        state.pos = label_start
        state.pos_max = label_end
        t = state.push("link_open", "a", 1)
        t.attrs = [("href", href)]
        if title:
            t.attrs.append(("title", title))
        state.link_level += 1
        _tokenize_inline(state)
        state.link_level -= 1
        state.push("link_close", "a", -1)
    state.pos = pos
    state.pos_max = maximum
    return True


def _inline_image(state: _InlineState, silent: bool) -> bool:
    href = ""
    old_pos = state.pos
    maximum = state.pos_max
    if state.src[state.pos] != "!" or state.ch(state.pos + 1) != "[":
        return False
    label_start = state.pos + 2
    label_end = _parse_link_label(state, state.pos + 1, False)
    if label_end < 0:
        return False
    pos = label_end + 1
    if pos < maximum and state.src[pos] == "(":
        pos = _skip_ws(state, pos + 1)
        if pos >= maximum:
            return False
        ok, res_pos, res_str = _parse_link_destination(state.src, pos, state.pos_max)
        if ok:
            href = normalize_link(res_str)
            if validate_link(href):
                pos = res_pos
            else:
                href = ""
        start = pos
        pos = _skip_ws(state, pos)
        ok_t, t_pos, _l, t_str = _parse_link_title(state.src, pos, state.pos_max)
        if pos < maximum and start != pos and ok_t:
            title = t_str
            pos = _skip_ws(state, t_pos)
        else:
            title = ""
        if pos >= maximum or state.src[pos] != ")":
            state.pos = old_pos
            return False
        pos += 1
    else:
        if "references" not in state.env:
            return False
        pos, ref = _reference_target(state, label_start, label_end, pos)
        if not ref:
            state.pos = old_pos
            return False
        href = ref["href"]
        title = ref["title"]
    if not silent:
        content = state.src[label_start:label_end]
        children: list[Token] = []
        _parse_inline(content, state.env, children)
        t = state.push("image", "img", 0)
        t.attrs = [("src", href), ("alt", "")]
        t.children = children
        t.content = content
        if title:
            t.attrs.append(("title", title))
    state.pos = pos
    state.pos_max = maximum
    return True


_AUTOLINK_RE = re.compile(r"^([a-zA-Z][a-zA-Z0-9+.\-]{1,31}):([^<>\x00-\x20]*)$")
_EMAIL_RE = re.compile(
    r"^([a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?"
    r"(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*)$"
)


def _inline_autolink(state: _InlineState, silent: bool) -> bool:
    pos = state.pos
    if state.src[pos] != "<":
        return False
    start = pos
    maximum = state.pos_max
    while True:
        pos += 1
        if pos >= maximum:
            return False
        ch = state.src[pos]
        if ch == "<":
            return False
        if ch == ">":
            break
    url = state.src[start + 1 : pos]
    if _AUTOLINK_RE.match(url):
        full = normalize_link(url)
    elif _EMAIL_RE.match(url):
        full = normalize_link("mailto:" + url)
    else:
        return False
    if not validate_link(full):
        return False
    if not silent:
        t = state.push("link_open", "a", 1)
        t.attrs = [("href", full)]
        t = state.push("text", "", 0)
        t.content = normalize_link_text(url)
        state.push("link_close", "a", -1)
    state.pos += len(url) + 2
    return True


_DIGITAL_RE = re.compile(r"^&#((?:x[a-f0-9]{1,6}|[0-9]{1,7}));", re.I)
_NAMED_RE = re.compile(r"^&([a-z][a-z0-9]{1,31});", re.I)


def _inline_entity(state: _InlineState, silent: bool) -> bool:
    pos = state.pos
    if state.src[pos] != "&" or pos + 1 >= state.pos_max:
        return False
    if state.src[pos + 1] == "#":
        m = _DIGITAL_RE.match(state.src[pos:])
        if m:
            if not silent:
                num = m.group(1)
                code = int(num[1:], 16) if num[0] in "xX" else int(num)
                t = state.push("text_special", "", 0)
                t.content = chr(code) if _valid_entity_code(code) else "\ufffd"
            state.pos += len(m.group(0))
            return True
    else:
        m = _NAMED_RE.match(state.src[pos:])
        if m:
            decoded = html.entities.html5.get(m.group(1) + ";")
            if decoded is not None:
                if not silent:
                    t = state.push("text_special", "", 0)
                    t.content = decoded
                state.pos += len(m.group(0))
                return True
    return False


_INLINE_RULES = [
    _inline_text, _inline_newline, _inline_escape, _inline_backticks, _inline_strikethrough,
    _inline_emphasis, _inline_link, _inline_image, _inline_autolink, _inline_entity,
]


def _skip_token(state: _InlineState) -> None:
    pos = state.pos
    if pos in state.cache:
        state.pos = state.cache[pos]
        return
    ok = False
    if state.level < MAX_NESTING:
        for rule in _INLINE_RULES:
            state.level += 1
            ok = rule(state, True)
            state.level -= 1
            if ok:
                break
    else:
        state.pos = state.pos_max
    if not ok:
        state.pos += 1
    state.cache[pos] = state.pos


def _tokenize_inline(state: _InlineState) -> None:
    end = state.pos_max
    while state.pos < end:
        ok = False
        if state.level < MAX_NESTING:
            for rule in _INLINE_RULES:
                if rule(state, False):
                    ok = True
                    break
        if ok:
            if state.pos >= end:
                break
            continue
        state.pending += state.src[state.pos]
        state.pos += 1
    if state.pending:
        state.push_pending()


def _process_delimiters(delims: list[_Delim]) -> None:
    openers_bottom: dict[str, list[int]] = {}
    n = len(delims)
    if not n:
        return
    header_idx = 0
    last_token_idx = -2
    jumps: list[int] = []
    for closer_idx in range(n):
        closer = delims[closer_idx]
        jumps.append(0)
        if delims[header_idx].marker != closer.marker or last_token_idx != closer.token - 1:
            header_idx = closer_idx
        last_token_idx = closer.token
        closer.length = closer.length or 0
        if not closer.close:
            continue
        bottoms = openers_bottom.setdefault(closer.marker, [-1] * 6)
        min_opener_idx = bottoms[(3 if closer.open else 0) + closer.length % 3]
        opener_idx = header_idx - jumps[header_idx] - 1
        new_min = opener_idx
        while opener_idx > min_opener_idx:
            opener = delims[opener_idx]
            if opener.marker != closer.marker:
                opener_idx -= jumps[opener_idx] + 1
                continue
            if opener.open and opener.end < 0:
                odd_match = False
                if opener.close or closer.open:
                    if (opener.length + closer.length) % 3 == 0:
                        if opener.length % 3 != 0 or closer.length % 3 != 0:
                            odd_match = True
                if not odd_match:
                    last_jump = (
                        jumps[opener_idx - 1] + 1
                        if opener_idx > 0 and not delims[opener_idx - 1].open
                        else 0
                    )
                    jumps[closer_idx] = closer_idx - opener_idx + last_jump
                    jumps[opener_idx] = last_jump
                    closer.open = False
                    opener.end = closer_idx
                    opener.close = False
                    new_min = -1
                    last_token_idx = -2
                    break
            opener_idx -= jumps[opener_idx] + 1
        if new_min != -1:
            bottoms[(3 if closer.open else 0) + (closer.length or 0) % 3] = new_min


def _post_strikethrough(tokens: list[Token], delims: list[_Delim]) -> None:
    lone: list[int] = []
    for start in delims:
        if start.marker != "~" or start.end == -1:
            continue
        end = delims[start.end]
        t = tokens[start.token]
        t.type, t.tag, t.nesting, t.content = "s_open", "s", 1, ""
        t = tokens[end.token]
        t.type, t.tag, t.nesting, t.content = "s_close", "s", -1, ""
        prev = tokens[end.token - 1]
        if prev.type == "text" and prev.content == "~":
            lone.append(end.token - 1)
    while lone:
        i = lone.pop()
        j = i + 1
        while j < len(tokens) and tokens[j].type == "s_close":
            j += 1
        j -= 1
        if i != j:
            tokens[i], tokens[j] = tokens[j], tokens[i]


def _post_emphasis(tokens: list[Token], delims: list[_Delim]) -> None:
    i = len(delims) - 1
    while i >= 0:
        start = delims[i]
        if start.marker not in "_*" or start.end == -1:
            i -= 1
            continue
        end = delims[start.end]
        is_strong = (
            i > 0
            and delims[i - 1].end == start.end + 1
            and delims[i - 1].marker == start.marker
            and delims[i - 1].token == start.token - 1
            and delims[start.end + 1].token == end.token + 1
        )
        tag = "strong" if is_strong else "em"
        t = tokens[start.token]
        t.type, t.tag, t.nesting, t.content = f"{tag}_open", tag, 1, ""
        t = tokens[end.token]
        t.type, t.tag, t.nesting, t.content = f"{tag}_close", tag, -1, ""
        if is_strong:
            tokens[delims[i - 1].token].content = ""
            tokens[delims[start.end + 1].token].content = ""
            i -= 1
        i -= 1


def _parse_inline(src: str, env: dict, out: list[Token]) -> None:
    state = _InlineState(src, env, out)
    _tokenize_inline(state)
    all_delims = [state.delimiters] + [m for m in state.tokens_meta if m]
    for d in all_delims:
        _process_delimiters(d)
    for d in all_delims:
        _post_strikethrough(state.tokens, d)
    for d in all_delims:
        _post_emphasis(state.tokens, d)


# --- renderer ---


def _render_attrs(t: Token) -> str:
    return "".join(f' {k}="{escape_html(v)}"' for k, v in t.attrs)


def _render_token(tokens: list[Token], idx: int) -> str:
    t = tokens[idx]
    if t.hidden:
        return ""
    out = ""
    if t.block and t.nesting != -1 and idx and tokens[idx - 1].hidden:
        out += "\n"
    out += ("</" if t.nesting == -1 else "<") + t.tag + _render_attrs(t)
    if t.nesting == 0:
        out += " /"
    need_lf = False
    if t.block:
        need_lf = True
        if t.nesting == 1 and idx + 1 < len(tokens):
            nxt = tokens[idx + 1]
            if nxt.type == "inline" or nxt.hidden:
                need_lf = False
            elif nxt.nesting == -1 and nxt.tag == t.tag:
                need_lf = False
    return out + (">\n" if need_lf else ">")


def _inline_as_text(tokens: list[Token]) -> str:
    out = []
    for t in tokens:
        if t.type == "text":
            out.append(t.content)
        elif t.type == "image":
            out.append(_inline_as_text(t.children or []))
        elif t.type in ("softbreak", "hardbreak"):
            out.append("\n")
    return "".join(out)


def _render_inline(tokens: list[Token]) -> str:
    out = []
    for i, t in enumerate(tokens):
        if t.type in ("text", "text_special"):
            out.append(escape_html(t.content))
        elif t.type == "code_inline":
            out.append(f"<code{_render_attrs(t)}>{escape_html(t.content)}</code>")
        elif t.type == "hardbreak":
            out.append("<br />\n")
        elif t.type == "softbreak":
            out.append("\n")
        elif t.type == "image":
            t.attrs = [(k, _inline_as_text(t.children or []) if k == "alt" else v) for k, v in t.attrs]
            out.append(_render_token(tokens, i))
        else:
            out.append(_render_token(tokens, i))
    return "".join(out)


def _render_fence(t: Token) -> str:
    info = unescape_all(t.info).strip()
    code = escape_html(t.content)
    if info:
        lang = re.split(r"\s+", info)[0]
        return f'<pre><code class="language-{escape_html(lang)}">{code}</code></pre>\n'
    return f"<pre><code>{code}</code></pre>\n"


def render(src: str) -> str:
    src = re.sub(r"\r\n?|\n", "\n", src).replace("\0", "\ufffd")
    env: dict = {}
    tokens: list[Token] = []
    if src:
        state = _BlockState(src, env, tokens)
        _tokenize_block(state, state.line, state.line_max)
    for t in tokens:
        if t.type == "inline":
            children: list[Token] = []
            _parse_inline(t.content, env, children)
            t.children = children
    out = []
    for i, t in enumerate(tokens):
        if t.type == "inline":
            out.append(_render_inline(t.children or []))
        elif t.type == "fence":
            out.append(_render_fence(t))
        elif t.type == "code_block":
            out.append(f"<pre><code>{escape_html(t.content)}</code></pre>\n")
        else:
            out.append(_render_token(tokens, i))
    return "".join(out)
