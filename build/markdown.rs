//! CommonMark + GFM tables/strikethrough renderer.
//!
//! This is a line-by-line port of markdown-it-py's "gfm-like" preset with
//! `html=False, linkify=False, typographer=False`, so that output is
//! byte-identical to the reference build. Indices are code-point offsets
//! (Python string semantics), so sources are held as `Vec<char>`.

use std::collections::HashMap;

use crate::entities;
use crate::mdurl::{normalize_link, normalize_link_text, validate_link};
use crate::unicode;
use crate::util::{chars_to_string, escape_html, py_split_ws, py_strip};

const MAX_NESTING: i64 = 20;

#[derive(Clone, Debug, Default)]
pub struct Token {
    pub ty: &'static str,
    pub tag: &'static str,
    pub nesting: i8,
    pub attrs: Vec<(&'static str, String)>,
    pub level: i64,
    pub children: Option<Vec<Token>>,
    pub content: String,
    pub markup: String,
    pub info: String,
    pub block: bool,
    pub hidden: bool,
}

impl Token {
    fn new(ty: &'static str, tag: &'static str, nesting: i8) -> Token {
        Token { ty, tag, nesting, ..Default::default() }
    }

    fn attr_set(&mut self, name: &'static str, value: String) {
        if let Some(a) = self.attrs.iter_mut().find(|a| a.0 == name) {
            a.1 = value;
        } else {
            self.attrs.push((name, value));
        }
    }
}

#[derive(Default)]
struct Env {
    /// `None` until the first reference definition is seen (mirrors `"references" in env`).
    references: Option<HashMap<String, (String, String)>>,
}

/// Render markdown source to HTML.
pub fn render(src: &str) -> String {
    // core: normalize
    let src = src.replace("\r\n", "\n").replace('\r', "\n").replace('\0', "\u{FFFD}");
    let mut env = Env::default();
    let mut tokens = Vec::new();
    // core: block
    if !src.is_empty() {
        let mut st = BlockState::new(&src, &mut env);
        let end = st.line_max;
        st.tokenize(0, end);
        tokens = st.tokens;
    }
    // core: inline
    for t in tokens.iter_mut() {
        if t.ty == "inline" {
            let content: Vec<char> = t.content.chars().collect();
            t.children = Some(parse_inline(content, &env));
        }
    }
    // core: text_join
    for t in tokens.iter_mut() {
        if t.ty != "inline" {
            continue;
        }
        let children = t.children.take().unwrap_or_default();
        let mut out: Vec<Token> = Vec::with_capacity(children.len());
        for mut c in children {
            if c.ty == "text_special" {
                c.ty = "text";
            }
            if c.ty == "text" {
                if let Some(last) = out.last_mut() {
                    if last.ty == "text" {
                        last.content.push_str(&c.content);
                        continue;
                    }
                }
            }
            out.push(c);
        }
        t.children = Some(out);
    }
    render_tokens(&mut tokens)
}

// ---------------------------------------------------------------------------
// Block level
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Parent {
    Root,
    Blockquote,
    Paragraph,
    List,
    Reference,
    Table,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rule {
    Table,
    Code,
    Fence,
    Blockquote,
    Hr,
    List,
    Reference,
    Heading,
    Lheading,
    Paragraph,
}

const RULES_ALL: &[Rule] = &[
    Rule::Table,
    Rule::Code,
    Rule::Fence,
    Rule::Blockquote,
    Rule::Hr,
    Rule::List,
    Rule::Reference,
    Rule::Heading,
    Rule::Lheading,
    Rule::Paragraph,
];
// html_block is disabled (html=False) and so omitted from the chains.
const TERM_PARAGRAPH: &[Rule] = &[Rule::Table, Rule::Fence, Rule::Blockquote, Rule::Hr, Rule::List, Rule::Heading];
const TERM_REFERENCE: &[Rule] = TERM_PARAGRAPH;
const TERM_BLOCKQUOTE: &[Rule] = &[Rule::Fence, Rule::Blockquote, Rule::Hr, Rule::List, Rule::Heading];
const TERM_LIST: &[Rule] = &[Rule::Fence, Rule::Blockquote, Rule::Hr];

fn is_sp(c: Option<char>) -> bool {
    matches!(c, Some(' ') | Some('\t'))
}

struct BlockState<'e> {
    src: Vec<char>,
    env: &'e mut Env,
    tokens: Vec<Token>,
    b_marks: Vec<i64>,
    e_marks: Vec<i64>,
    t_shift: Vec<i64>,
    s_count: Vec<i64>,
    bs_count: Vec<i64>,
    blk_indent: i64,
    line: i64,
    line_max: i64,
    tight: bool,
    list_indent: i64,
    parent: Parent,
    level: i64,
}

impl<'e> BlockState<'e> {
    fn new(src: &str, env: &'e mut Env) -> BlockState<'e> {
        let chars: Vec<char> = src.chars().collect();
        let mut st = BlockState {
            src: Vec::new(),
            env,
            tokens: Vec::new(),
            b_marks: Vec::new(),
            e_marks: Vec::new(),
            t_shift: Vec::new(),
            s_count: Vec::new(),
            bs_count: Vec::new(),
            blk_indent: 0,
            line: 0,
            line_max: 0,
            tight: false,
            list_indent: -1,
            parent: Parent::Root,
            level: 0,
        };
        let length = chars.len() as i64;
        let mut indent_found = false;
        let (mut start, mut indent, mut offset) = (0i64, 0i64, 0i64);
        for (p, &ch) in chars.iter().enumerate() {
            let mut pos = p as i64;
            if !indent_found {
                if ch == ' ' || ch == '\t' {
                    indent += 1;
                    if ch == '\t' {
                        offset += 4 - offset % 4;
                    } else {
                        offset += 1;
                    }
                    continue;
                } else {
                    indent_found = true;
                }
            }
            if ch == '\n' || pos == length - 1 {
                if ch != '\n' {
                    pos += 1;
                }
                st.b_marks.push(start);
                st.e_marks.push(pos);
                st.t_shift.push(indent);
                st.s_count.push(offset);
                st.bs_count.push(0);
                indent_found = false;
                indent = 0;
                offset = 0;
                start = pos + 1;
            }
        }
        st.b_marks.push(length);
        st.e_marks.push(length);
        st.t_shift.push(0);
        st.s_count.push(0);
        st.bs_count.push(0);
        st.line_max = st.b_marks.len() as i64 - 1;
        st.src = chars;
        st
    }

    // --- accessors ---
    fn c(&self, i: i64) -> Option<char> {
        if i < 0 {
            None
        } else {
            self.src.get(i as usize).copied()
        }
    }
    fn bm(&self, l: i64) -> i64 {
        self.b_marks[l as usize]
    }
    fn em(&self, l: i64) -> i64 {
        self.e_marks[l as usize]
    }
    fn ts(&self, l: i64) -> i64 {
        self.t_shift[l as usize]
    }
    fn sc(&self, l: i64) -> i64 {
        self.s_count[l as usize]
    }
    fn bsc(&self, l: i64) -> i64 {
        self.bs_count[l as usize]
    }
    fn slice(&self, a: i64, b: i64) -> String {
        let n = self.src.len() as i64;
        let a = a.clamp(0, n) as usize;
        let b = b.clamp(0, n) as usize;
        if a >= b {
            String::new()
        } else {
            chars_to_string(&self.src[a..b])
        }
    }

    fn push(&mut self, ty: &'static str, tag: &'static str, nesting: i8) -> usize {
        let mut t = Token::new(ty, tag, nesting);
        t.block = true;
        if nesting < 0 {
            self.level -= 1;
        }
        t.level = self.level;
        if nesting > 0 {
            self.level += 1;
        }
        self.tokens.push(t);
        self.tokens.len() - 1
    }

    fn is_empty(&self, line: i64) -> bool {
        self.bm(line) + self.ts(line) >= self.em(line)
    }

    fn skip_empty_lines(&self, mut from: i64) -> i64 {
        while from < self.line_max {
            if self.bm(from) + self.ts(from) < self.em(from) {
                break;
            }
            from += 1;
        }
        from
    }

    fn skip_spaces(&self, mut pos: i64) -> i64 {
        while is_sp(self.c(pos)) {
            pos += 1;
        }
        pos
    }

    fn skip_spaces_back(&self, mut pos: i64, min: i64) -> i64 {
        if pos <= min {
            return pos;
        }
        while pos > min {
            pos -= 1;
            if !is_sp(self.c(pos)) {
                return pos + 1;
            }
        }
        pos
    }

    fn skip_chars(&self, mut pos: i64, ch: char) -> i64 {
        while self.c(pos) == Some(ch) {
            pos += 1;
        }
        pos
    }

    fn skip_chars_back(&self, mut pos: i64, ch: char, min: i64) -> i64 {
        if pos <= min {
            return pos;
        }
        while pos > min {
            pos -= 1;
            if self.c(pos) != Some(ch) {
                return pos + 1;
            }
        }
        pos
    }

    fn get_lines(&self, begin: i64, end: i64, indent: i64, keep_last_lf: bool) -> String {
        if begin >= end {
            return String::new();
        }
        let mut out = String::new();
        let mut line = begin;
        while line < end {
            let mut line_indent = 0;
            let line_start = self.bm(line);
            let mut first = line_start;
            let last = if line + 1 < end || keep_last_lf { self.em(line) + 1 } else { self.em(line) };
            while first < last && line_indent < indent {
                let ch = self.c(first);
                if is_sp(ch) {
                    if ch == Some('\t') {
                        line_indent += 4 - (line_indent + self.bsc(line)) % 4;
                    } else {
                        line_indent += 1;
                    }
                } else if first - line_start < self.ts(line) {
                    line_indent += 1;
                } else {
                    break;
                }
                first += 1;
            }
            if line_indent > indent {
                for _ in 0..(line_indent - indent) {
                    out.push(' ');
                }
            }
            out.push_str(&self.slice(first, last));
            line += 1;
        }
        out
    }

    fn is_code_block(&self, line: i64) -> bool {
        self.sc(line) - self.blk_indent >= 4
    }

    fn tokenize(&mut self, start_line: i64, end_line: i64) {
        let mut line = start_line;
        let mut has_empty_lines = false;
        while line < end_line {
            line = self.skip_empty_lines(line);
            self.line = line;
            if line >= end_line {
                break;
            }
            if self.sc(line) < self.blk_indent {
                break;
            }
            if self.level >= MAX_NESTING {
                self.line = end_line;
                break;
            }
            for &r in RULES_ALL {
                if self.run(r, line, end_line, false) {
                    break;
                }
            }
            self.tight = !has_empty_lines;
            line = self.line;
            if line - 1 < end_line && self.is_empty(line - 1) {
                has_empty_lines = true;
            }
            if line < end_line && self.is_empty(line) {
                has_empty_lines = true;
                line += 1;
                self.line = line;
            }
        }
    }

    fn run(&mut self, r: Rule, start: i64, end: i64, silent: bool) -> bool {
        match r {
            Rule::Table => self.rule_table(start, end, silent),
            Rule::Code => self.rule_code(start, end, silent),
            Rule::Fence => self.rule_fence(start, end, silent),
            Rule::Blockquote => self.rule_blockquote(start, end, silent),
            Rule::Hr => self.rule_hr(start, end, silent),
            Rule::List => self.rule_list(start, end, silent),
            Rule::Reference => self.rule_reference(start, end, silent),
            Rule::Heading => self.rule_heading(start, end, silent),
            Rule::Lheading => self.rule_lheading(start, end, silent),
            Rule::Paragraph => self.rule_paragraph(start, end, silent),
        }
    }

    fn terminates(&mut self, rules: &[Rule], line: i64, end: i64) -> bool {
        for &r in rules {
            if self.run(r, line, end, true) {
                return true;
            }
        }
        false
    }

    // --- table ---
    fn get_line(&self, line: i64) -> String {
        self.slice(self.bm(line) + self.ts(line), self.em(line))
    }

    fn rule_table(&mut self, start_line: i64, end_line: i64, silent: bool) -> bool {
        if start_line + 2 > end_line {
            return false;
        }
        let mut next_line = start_line + 1;
        if self.sc(next_line) < self.blk_indent {
            return false;
        }
        if self.is_code_block(next_line) {
            return false;
        }
        let mut pos = self.bm(next_line) + self.ts(next_line);
        if pos >= self.em(next_line) {
            return false;
        }
        let first_ch = self.c(pos);
        pos += 1;
        if !matches!(first_ch, Some('|') | Some('-') | Some(':')) {
            return false;
        }
        if pos >= self.em(next_line) {
            return false;
        }
        let second_ch = self.c(pos);
        pos += 1;
        if !matches!(second_ch, Some('|') | Some('-') | Some(':')) && !is_sp(second_ch) {
            return false;
        }
        if first_ch == Some('-') && is_sp(second_ch) {
            return false;
        }
        while pos < self.em(next_line) {
            let ch = self.c(pos);
            if !matches!(ch, Some('|') | Some('-') | Some(':')) && !is_sp(ch) {
                return false;
            }
            pos += 1;
        }
        let line_text = self.get_line(start_line + 1);
        let columns: Vec<&str> = line_text.split('|').collect();
        let mut aligns: Vec<&'static str> = Vec::new();
        for (i, col) in columns.iter().enumerate() {
            let t = py_strip(col);
            if t.is_empty() {
                if i == 0 || i == columns.len() - 1 {
                    continue;
                } else {
                    return false;
                }
            }
            if !header_line_ok(t) {
                return false;
            }
            if t.ends_with(':') {
                aligns.push(if t.starts_with(':') { "center" } else { "right" });
            } else if t.starts_with(':') {
                aligns.push("left");
            } else {
                aligns.push("");
            }
        }
        let line_text = self.get_line(start_line);
        let line_text = py_strip(&line_text);
        if !line_text.contains('|') {
            return false;
        }
        if self.is_code_block(start_line) {
            return false;
        }
        let mut columns = escaped_split(line_text);
        if columns.first().map_or(false, |c| c.is_empty()) {
            columns.remove(0);
        }
        if columns.last().map_or(false, |c| c.is_empty()) {
            columns.pop();
        }
        let column_count = columns.len();
        if column_count == 0 || column_count != aligns.len() {
            return false;
        }
        if silent {
            return true;
        }
        let old_parent = self.parent;
        self.parent = Parent::Table;
        self.push("table_open", "table", 1);
        self.push("thead_open", "thead", 1);
        self.push("tr_open", "tr", 1);
        for (i, col) in columns.iter().enumerate() {
            let t = self.push("th_open", "th", 1);
            if !aligns[i].is_empty() {
                self.tokens[t].attrs = vec![("style", format!("text-align:{}", aligns[i]))];
            }
            let t = self.push("inline", "", 0);
            self.tokens[t].content = py_strip(col).to_string();
            self.tokens[t].children = Some(Vec::new());
            self.push("th_close", "th", -1);
        }
        self.push("tr_close", "tr", -1);
        self.push("thead_close", "thead", -1);
        let mut autocompleted: i64 = 0;
        next_line = start_line + 2;
        let mut tbody = false;
        while next_line < end_line {
            if self.sc(next_line) < self.blk_indent {
                break;
            }
            if self.terminates(TERM_BLOCKQUOTE, next_line, end_line) {
                break;
            }
            let lt = self.get_line(next_line);
            let lt = py_strip(&lt).to_string();
            if lt.is_empty() {
                break;
            }
            if self.is_code_block(next_line) {
                break;
            }
            let mut cols = escaped_split(&lt);
            if cols.first().map_or(false, |c| c.is_empty()) {
                cols.remove(0);
            }
            if cols.last().map_or(false, |c| c.is_empty()) {
                cols.pop();
            }
            autocompleted += column_count as i64 - cols.len() as i64;
            if autocompleted > 0x10000 {
                break;
            }
            if next_line == start_line + 2 {
                self.push("tbody_open", "tbody", 1);
                tbody = true;
            }
            self.push("tr_open", "tr", 1);
            for (i, align) in aligns.iter().enumerate().take(column_count) {
                let t = self.push("td_open", "td", 1);
                if !align.is_empty() {
                    self.tokens[t].attrs = vec![("style", format!("text-align:{}", align))];
                }
                let t = self.push("inline", "", 0);
                self.tokens[t].content = cols.get(i).map(|c| py_strip(c).to_string()).unwrap_or_default();
                self.tokens[t].children = Some(Vec::new());
                self.push("td_close", "td", -1);
            }
            self.push("tr_close", "tr", -1);
            next_line += 1;
        }
        if tbody {
            self.push("tbody_close", "tbody", -1);
        }
        self.push("table_close", "table", -1);
        self.parent = old_parent;
        self.line = next_line;
        true
    }

    // --- code ---
    fn rule_code(&mut self, start_line: i64, end_line: i64, _silent: bool) -> bool {
        if !self.is_code_block(start_line) {
            return false;
        }
        let mut next_line = start_line + 1;
        let mut last = next_line;
        while next_line < end_line {
            if self.is_empty(next_line) {
                next_line += 1;
                continue;
            }
            if self.is_code_block(next_line) {
                next_line += 1;
                last = next_line;
                continue;
            }
            break;
        }
        self.line = last;
        let content = self.get_lines(start_line, last, 4 + self.blk_indent, false) + "\n";
        let t = self.push("code_block", "code", 0);
        self.tokens[t].content = content;
        true
    }

    // --- fence ---
    fn rule_fence(&mut self, start_line: i64, end_line: i64, silent: bool) -> bool {
        let mut have_end = false;
        let mut pos = self.bm(start_line) + self.ts(start_line);
        let mut max = self.em(start_line);
        if self.is_code_block(start_line) {
            return false;
        }
        if pos + 3 > max {
            return false;
        }
        let marker = match self.c(pos) {
            Some(c @ ('~' | '`')) => c,
            _ => return false,
        };
        let mut mem = pos;
        pos = self.skip_chars(pos, marker);
        let len = pos - mem;
        if len < 3 {
            return false;
        }
        let markup = self.slice(mem, pos);
        let params = self.slice(pos, max);
        if marker == '`' && params.contains('`') {
            return false;
        }
        if silent {
            return true;
        }
        let mut next_line = start_line;
        loop {
            next_line += 1;
            if next_line >= end_line {
                break;
            }
            pos = self.bm(next_line) + self.ts(next_line);
            mem = pos;
            max = self.em(next_line);
            if pos < max && self.sc(next_line) < self.blk_indent {
                break;
            }
            match self.c(pos) {
                None => break,
                Some(c) if c != marker => continue,
                _ => {}
            }
            if self.is_code_block(next_line) {
                continue;
            }
            pos = self.skip_chars(pos, marker);
            if pos - mem < len {
                continue;
            }
            pos = self.skip_spaces(pos);
            if pos < max {
                continue;
            }
            have_end = true;
            break;
        }
        let indent = self.sc(start_line);
        self.line = next_line + if have_end { 1 } else { 0 };
        let content = self.get_lines(start_line + 1, next_line, indent, true);
        let t = self.push("fence", "code", 0);
        self.tokens[t].info = params;
        self.tokens[t].content = content;
        self.tokens[t].markup = markup;
        true
    }

    // --- blockquote ---
    fn rule_blockquote(&mut self, start_line: i64, end_line: i64, silent: bool) -> bool {
        let old_line_max = self.line_max;
        let mut pos = self.bm(start_line) + self.ts(start_line);
        let mut max = self.em(start_line);
        if self.is_code_block(start_line) {
            return false;
        }
        if self.c(pos) != Some('>') {
            return false;
        }
        pos += 1;
        if silent {
            return true;
        }
        let mut initial = self.sc(start_line) + 1;
        let mut offset = initial;
        let mut adjust_tab = false;
        let mut space_after_marker;
        match self.c(pos) {
            Some(' ') => {
                pos += 1;
                initial += 1;
                offset += 1;
                adjust_tab = false;
                space_after_marker = true;
            }
            Some('\t') => {
                space_after_marker = true;
                if (self.bsc(start_line) + offset) % 4 == 3 {
                    pos += 1;
                    initial += 1;
                    offset += 1;
                    adjust_tab = false;
                } else {
                    adjust_tab = true;
                }
            }
            _ => space_after_marker = false,
        }
        let mut old_b_marks = vec![self.bm(start_line)];
        self.b_marks[start_line as usize] = pos;
        while pos < max {
            let ch = self.c(pos);
            if is_sp(ch) {
                if ch == Some('\t') {
                    offset += 4 - (offset + self.bsc(start_line) + if adjust_tab { 1 } else { 0 }) % 4;
                } else {
                    offset += 1;
                }
            } else {
                break;
            }
            pos += 1;
        }
        let mut old_bs_count = vec![self.bsc(start_line)];
        self.bs_count[start_line as usize] = self.sc(start_line) + 1 + if space_after_marker { 1 } else { 0 };
        let mut last_line_empty = pos >= max;
        let mut old_s_count = vec![self.sc(start_line)];
        self.s_count[start_line as usize] = offset - initial;
        let mut old_t_shift = vec![self.ts(start_line)];
        self.t_shift[start_line as usize] = pos - self.bm(start_line);

        let old_parent = self.parent;
        self.parent = Parent::Blockquote;
        let mut next_line = start_line + 1;
        while next_line < end_line {
            let is_outdented = self.sc(next_line) < self.blk_indent;
            pos = self.bm(next_line) + self.ts(next_line);
            max = self.em(next_line);
            if pos >= max {
                break;
            }
            let evaluates_true = self.c(pos) == Some('>') && !is_outdented;
            pos += 1;
            if evaluates_true {
                initial = self.sc(next_line) + 1;
                offset = initial;
                match self.c(pos) {
                    Some(' ') => {
                        pos += 1;
                        initial += 1;
                        offset += 1;
                        adjust_tab = false;
                        space_after_marker = true;
                    }
                    Some('\t') => {
                        space_after_marker = true;
                        if (self.bsc(next_line) + offset) % 4 == 3 {
                            pos += 1;
                            initial += 1;
                            offset += 1;
                            adjust_tab = false;
                        } else {
                            adjust_tab = true;
                        }
                    }
                    _ => space_after_marker = false,
                }
                old_b_marks.push(self.bm(next_line));
                self.b_marks[next_line as usize] = pos;
                while pos < max {
                    let ch = self.c(pos);
                    if is_sp(ch) {
                        if ch == Some('\t') {
                            offset += 4 - (offset + self.bsc(next_line) + if adjust_tab { 1 } else { 0 }) % 4;
                        } else {
                            offset += 1;
                        }
                    } else {
                        break;
                    }
                    pos += 1;
                }
                last_line_empty = pos >= max;
                old_bs_count.push(self.bsc(next_line));
                self.bs_count[next_line as usize] =
                    self.sc(next_line) + 1 + if space_after_marker { 1 } else { 0 };
                old_s_count.push(self.sc(next_line));
                self.s_count[next_line as usize] = offset - initial;
                old_t_shift.push(self.ts(next_line));
                self.t_shift[next_line as usize] = pos - self.bm(next_line);
                next_line += 1;
                continue;
            }
            if last_line_empty {
                break;
            }
            if self.terminates(TERM_BLOCKQUOTE, next_line, end_line) {
                self.line_max = next_line;
                if self.blk_indent != 0 {
                    old_b_marks.push(self.bm(next_line));
                    old_bs_count.push(self.bsc(next_line));
                    old_t_shift.push(self.ts(next_line));
                    old_s_count.push(self.sc(next_line));
                    self.s_count[next_line as usize] -= self.blk_indent;
                }
                break;
            }
            old_b_marks.push(self.bm(next_line));
            old_bs_count.push(self.bsc(next_line));
            old_t_shift.push(self.ts(next_line));
            old_s_count.push(self.sc(next_line));
            self.s_count[next_line as usize] = -1;
            next_line += 1;
        }
        let old_indent = self.blk_indent;
        self.blk_indent = 0;
        let t = self.push("blockquote_open", "blockquote", 1);
        self.tokens[t].markup = ">".into();
        self.tokenize(start_line, next_line);
        let t = self.push("blockquote_close", "blockquote", -1);
        self.tokens[t].markup = ">".into();
        self.line_max = old_line_max;
        self.parent = old_parent;
        for (i, item) in old_t_shift.iter().enumerate() {
            let l = i + start_line as usize;
            self.b_marks[l] = old_b_marks[i];
            self.t_shift[l] = *item;
            self.s_count[l] = old_s_count[i];
            self.bs_count[l] = old_bs_count[i];
        }
        self.blk_indent = old_indent;
        true
    }

    // --- hr ---
    fn rule_hr(&mut self, start_line: i64, _end: i64, silent: bool) -> bool {
        let mut pos = self.bm(start_line) + self.ts(start_line);
        let max = self.em(start_line);
        if self.is_code_block(start_line) {
            return false;
        }
        let marker = match self.c(pos) {
            Some(c) => c,
            None => return false,
        };
        pos += 1;
        if !matches!(marker, '*' | '-' | '_') {
            return false;
        }
        let mut cnt = 1;
        while pos < max {
            let ch = self.c(pos);
            pos += 1;
            if ch != Some(marker) && !is_sp(ch) {
                return false;
            }
            if ch == Some(marker) {
                cnt += 1;
            }
        }
        if cnt < 3 {
            return false;
        }
        if silent {
            return true;
        }
        self.line = start_line + 1;
        let t = self.push("hr", "hr", 0);
        self.tokens[t].markup = std::iter::repeat(marker).take(cnt + 1).collect();
        true
    }

    // --- list ---
    fn skip_bullet_marker(&self, line: i64) -> i64 {
        let mut pos = self.bm(line) + self.ts(line);
        let max = self.em(line);
        let marker = match self.c(pos) {
            Some(c) => c,
            None => return -1,
        };
        pos += 1;
        if !matches!(marker, '*' | '-' | '+') {
            return -1;
        }
        if pos < max && !is_sp(self.c(pos)) {
            return -1;
        }
        pos
    }

    fn skip_ordered_marker(&self, line: i64) -> i64 {
        let start = self.bm(line) + self.ts(line);
        let mut pos = start;
        let max = self.em(line);
        if pos + 1 >= max {
            return -1;
        }
        let ch = self.c(pos);
        pos += 1;
        if !ch.map_or(false, |c| c.is_ascii_digit()) {
            return -1;
        }
        loop {
            if pos >= max {
                return -1;
            }
            let ch = self.c(pos);
            pos += 1;
            if ch.map_or(false, |c| c.is_ascii_digit()) {
                if pos - start >= 10 {
                    return -1;
                }
                continue;
            }
            if matches!(ch, Some(')') | Some('.')) {
                break;
            }
            return -1;
        }
        if pos < max && !is_sp(self.c(pos)) {
            return -1;
        }
        pos
    }

    fn mark_tight_paragraphs(&mut self, idx: usize) {
        let level = self.level + 2;
        let mut i = idx + 2;
        let length = self.tokens.len().saturating_sub(2);
        while i < length {
            if self.tokens[i].level == level && self.tokens[i].ty == "paragraph_open" {
                self.tokens[i + 2].hidden = true;
                self.tokens[i].hidden = true;
                i += 2;
            }
            i += 1;
        }
    }

    fn rule_list(&mut self, mut start_line: i64, end_line: i64, silent: bool) -> bool {
        let mut is_terminating_paragraph = false;
        let mut tight = true;
        if self.is_code_block(start_line) {
            return false;
        }
        if self.list_indent >= 0
            && self.sc(start_line) - self.list_indent >= 4
            && self.sc(start_line) < self.blk_indent
        {
            return false;
        }
        if silent && self.parent == Parent::Paragraph && self.sc(start_line) >= self.blk_indent {
            is_terminating_paragraph = true;
        }
        let is_ordered;
        let mut marker_value: i64 = 0;
        let mut start;
        let mut pos_after_marker = self.skip_ordered_marker(start_line);
        if pos_after_marker >= 0 {
            is_ordered = true;
            start = self.bm(start_line) + self.ts(start_line);
            marker_value = self.slice(start, pos_after_marker - 1).parse::<i64>().unwrap_or(0);
            if is_terminating_paragraph && marker_value != 1 {
                return false;
            }
        } else {
            pos_after_marker = self.skip_bullet_marker(start_line);
            if pos_after_marker >= 0 {
                is_ordered = false;
                start = 0;
            } else {
                return false;
            }
        }
        if is_terminating_paragraph && self.skip_spaces(pos_after_marker) >= self.em(start_line) {
            return false;
        }
        let marker_char = self.c(pos_after_marker - 1);
        if silent {
            return true;
        }
        let list_tok_idx = self.tokens.len();
        if is_ordered {
            let t = self.push("ordered_list_open", "ol", 1);
            if marker_value != 1 {
                self.tokens[t].attrs = vec![("start", marker_value.to_string())];
            }
        } else {
            self.push("bullet_list_open", "ul", 1);
        }
        let mut next_line = start_line;
        let mut prev_empty_end = false;
        let old_parent = self.parent;
        self.parent = Parent::List;
        while next_line < end_line {
            let mut pos = pos_after_marker;
            let max = self.em(next_line);
            let initial = self.sc(next_line) + pos_after_marker - (self.bm(start_line) + self.ts(start_line));
            let mut offset = initial;
            while pos < max {
                match self.c(pos) {
                    Some('\t') => offset += 4 - (offset + self.bsc(next_line)) % 4,
                    Some(' ') => offset += 1,
                    _ => break,
                }
                pos += 1;
            }
            let content_start = pos;
            let mut indent_after_marker = if content_start >= max { 1 } else { offset - initial };
            if indent_after_marker > 4 {
                indent_after_marker = 1;
            }
            let indent = initial + indent_after_marker;
            let t = self.push("list_item_open", "li", 1);
            self.tokens[t].markup = marker_char.map(String::from).unwrap_or_default();
            if is_ordered {
                self.tokens[t].info = self.slice(start, pos_after_marker - 1);
            }
            let old_tight = self.tight;
            let old_t_shift = self.ts(start_line);
            let old_s_count = self.sc(start_line);
            let old_list_indent = self.list_indent;
            self.list_indent = self.blk_indent;
            self.blk_indent = indent;
            self.tight = true;
            self.t_shift[start_line as usize] = content_start - self.bm(start_line);
            self.s_count[start_line as usize] = offset;
            if content_start >= max && self.is_empty(start_line + 1) {
                self.line = (self.line + 2).min(end_line);
            } else {
                self.tokenize(start_line, end_line);
            }
            if !self.tight || prev_empty_end {
                tight = false;
            }
            prev_empty_end = (self.line - start_line) > 1 && self.is_empty(self.line - 1);
            self.blk_indent = self.list_indent;
            self.list_indent = old_list_indent;
            self.t_shift[start_line as usize] = old_t_shift;
            self.s_count[start_line as usize] = old_s_count;
            self.tight = old_tight;
            let t = self.push("list_item_close", "li", -1);
            self.tokens[t].markup = marker_char.map(String::from).unwrap_or_default();
            start_line = self.line;
            next_line = start_line;
            if next_line >= end_line {
                break;
            }
            if self.sc(next_line) < self.blk_indent {
                break;
            }
            if self.is_code_block(start_line) {
                break;
            }
            if self.terminates(TERM_LIST, next_line, end_line) {
                break;
            }
            if is_ordered {
                pos_after_marker = self.skip_ordered_marker(next_line);
                if pos_after_marker < 0 {
                    break;
                }
                start = self.bm(next_line) + self.ts(next_line);
            } else {
                pos_after_marker = self.skip_bullet_marker(next_line);
                if pos_after_marker < 0 {
                    break;
                }
            }
            if marker_char != self.c(pos_after_marker - 1) {
                break;
            }
        }
        if is_ordered {
            self.push("ordered_list_close", "ol", -1);
        } else {
            self.push("bullet_list_close", "ul", -1);
        }
        self.line = next_line;
        self.parent = old_parent;
        if tight {
            self.mark_tight_paragraphs(list_tok_idx);
        }
        true
    }

    // --- reference ---
    fn get_next_line(&mut self, next_line: i64) -> Option<Vec<char>> {
        let end_line = self.line_max;
        if next_line >= end_line || self.is_empty(next_line) {
            return None;
        }
        let mut is_continuation = false;
        if self.is_code_block(next_line) {
            is_continuation = true;
        }
        if self.sc(next_line) < 0 {
            is_continuation = true;
        }
        if !is_continuation {
            let old_parent = self.parent;
            self.parent = Parent::Reference;
            let terminate = self.terminates(TERM_REFERENCE, next_line, end_line);
            self.parent = old_parent;
            if terminate {
                return None;
            }
        }
        let pos = self.bm(next_line) + self.ts(next_line);
        let max = self.em(next_line);
        Some(self.slice(pos, max + 1).chars().collect())
    }

    fn rule_reference(&mut self, start_line: i64, _end: i64, silent: bool) -> bool {
        let pos0 = self.bm(start_line) + self.ts(start_line);
        let max0 = self.em(start_line);
        let mut next_line = start_line + 1;
        if self.is_code_block(start_line) {
            return false;
        }
        if self.c(pos0) != Some('[') {
            return false;
        }
        let mut s: Vec<char> = self.slice(pos0, max0 + 1).chars().collect();
        let mut max = s.len() as i64;
        let at = |s: &Vec<char>, i: i64| -> Option<char> {
            if i < 0 {
                None
            } else {
                s.get(i as usize).copied()
            }
        };
        let mut label_end: i64 = -1;
        let mut pos: i64 = 1;
        while pos < max {
            let ch = at(&s, pos);
            if ch == Some('[') {
                return false;
            } else if ch == Some(']') {
                label_end = pos;
                break;
            } else if ch == Some('\n') {
                if let Some(lc) = self.get_next_line(next_line) {
                    s.extend(lc);
                    max = s.len() as i64;
                    next_line += 1;
                }
            } else if ch == Some('\\') {
                pos += 1;
                if pos < max && at(&s, pos) == Some('\n') {
                    if let Some(lc) = self.get_next_line(next_line) {
                        s.extend(lc);
                        max = s.len() as i64;
                        next_line += 1;
                    }
                }
            }
            pos += 1;
        }
        if label_end < 0 || at(&s, label_end + 1) != Some(':') {
            return false;
        }
        pos = label_end + 2;
        while pos < max {
            let ch = at(&s, pos);
            if ch == Some('\n') {
                if let Some(lc) = self.get_next_line(next_line) {
                    s.extend(lc);
                    max = s.len() as i64;
                    next_line += 1;
                }
            } else if is_sp(ch) {
            } else {
                break;
            }
            pos += 1;
        }
        let dest = parse_link_destination(&s, pos, max);
        if !dest.ok {
            return false;
        }
        let href = normalize_link(&dest.s);
        if !validate_link(&href) {
            return false;
        }
        pos = dest.pos;
        let dest_end_pos = pos;
        let dest_end_line = next_line;
        let start = pos;
        while pos < max {
            let ch = at(&s, pos);
            if ch == Some('\n') {
                if let Some(lc) = self.get_next_line(next_line) {
                    s.extend(lc);
                    max = s.len() as i64;
                    next_line += 1;
                }
            } else if is_sp(ch) {
            } else {
                break;
            }
            pos += 1;
        }
        let mut title_res = parse_link_title(&s, pos, max, None);
        while title_res.can_continue {
            let lc = match self.get_next_line(next_line) {
                Some(l) => l,
                None => break,
            };
            s.extend(lc);
            pos = max;
            max = s.len() as i64;
            next_line += 1;
            title_res = parse_link_title(&s, pos, max, Some(&title_res));
        }
        let mut title;
        if pos < max && start != pos && title_res.ok {
            title = title_res.s.clone();
            pos = title_res.pos;
        } else {
            title = String::new();
            pos = dest_end_pos;
            next_line = dest_end_line;
        }
        while pos < max {
            if !is_sp(at(&s, pos)) {
                break;
            }
            pos += 1;
        }
        if pos < max && at(&s, pos) != Some('\n') && !title.is_empty() {
            title = String::new();
            pos = dest_end_pos;
            next_line = dest_end_line;
            while pos < max {
                if !is_sp(at(&s, pos)) {
                    break;
                }
                pos += 1;
            }
        }
        if pos < max && at(&s, pos) != Some('\n') {
            return false;
        }
        let label = normalize_reference(&chars_to_string(&s[1..label_end as usize]));
        if label.is_empty() {
            return false;
        }
        if silent {
            return true;
        }
        let refs = self.env.references.get_or_insert_with(HashMap::new);
        self.line = next_line;
        refs.entry(label).or_insert((href, title));
        true
    }

    // --- heading ---
    fn rule_heading(&mut self, start_line: i64, _end: i64, silent: bool) -> bool {
        let mut pos = self.bm(start_line) + self.ts(start_line);
        let mut max = self.em(start_line);
        if self.is_code_block(start_line) {
            return false;
        }
        let mut ch = self.c(pos);
        if ch != Some('#') || pos >= max {
            return false;
        }
        let mut level = 1;
        pos += 1;
        ch = self.c(pos);
        while ch == Some('#') && pos < max && level <= 6 {
            level += 1;
            pos += 1;
            ch = self.c(pos);
        }
        if level > 6 || (pos < max && !is_sp(ch)) {
            return false;
        }
        if silent {
            return true;
        }
        max = self.skip_spaces_back(max, pos);
        let tmp = self.skip_chars_back(max, '#', pos);
        if tmp > pos && is_sp(self.c(tmp - 1)) {
            max = tmp;
        }
        self.line = start_line + 1;
        let tag = HEADING_TAGS[level - 1];
        let t = self.push("heading_open", tag, 1);
        self.tokens[t].markup = "#".repeat(level);
        let content = py_strip(&self.slice(pos, max)).to_string();
        let t = self.push("inline", "", 0);
        self.tokens[t].content = content;
        self.tokens[t].children = Some(Vec::new());
        let t = self.push("heading_close", tag, -1);
        self.tokens[t].markup = "#".repeat(level);
        true
    }

    // --- lheading ---
    fn rule_lheading(&mut self, start_line: i64, end_line: i64, _silent: bool) -> bool {
        let mut level = 0;
        let mut next_line = start_line + 1;
        let mut marker = ' ';
        if self.is_code_block(start_line) {
            return false;
        }
        let old_parent = self.parent;
        // NB: like markdown-it, parentType is not restored when no underline is found.
        self.parent = Parent::Paragraph;
        while next_line < end_line && !self.is_empty(next_line) {
            if self.sc(next_line) - self.blk_indent > 3 {
                next_line += 1;
                continue;
            }
            if self.sc(next_line) >= self.blk_indent {
                let mut pos = self.bm(next_line) + self.ts(next_line);
                let max = self.em(next_line);
                if pos < max {
                    if let Some(m @ ('-' | '=')) = self.c(pos) {
                        marker = m;
                        pos = self.skip_chars(pos, m);
                        pos = self.skip_spaces(pos);
                        if pos >= max {
                            level = if m == '=' { 1 } else { 2 };
                            break;
                        }
                    }
                }
            }
            if self.sc(next_line) < 0 {
                next_line += 1;
                continue;
            }
            if self.terminates(TERM_PARAGRAPH, next_line, end_line) {
                break;
            }
            next_line += 1;
        }
        if level == 0 {
            return false;
        }
        let content = py_strip(&self.get_lines(start_line, next_line, self.blk_indent, false)).to_string();
        self.line = next_line + 1;
        let tag = HEADING_TAGS[level - 1];
        let t = self.push("heading_open", tag, 1);
        self.tokens[t].markup = marker.to_string();
        let t = self.push("inline", "", 0);
        self.tokens[t].content = content;
        self.tokens[t].children = Some(Vec::new());
        let t = self.push("heading_close", tag, -1);
        self.tokens[t].markup = marker.to_string();
        self.parent = old_parent;
        true
    }

    // --- paragraph ---
    fn rule_paragraph(&mut self, start_line: i64, _end: i64, _silent: bool) -> bool {
        let mut next_line = start_line + 1;
        let end_line = self.line_max;
        let old_parent = self.parent;
        self.parent = Parent::Paragraph;
        while next_line < end_line {
            if self.is_empty(next_line) {
                break;
            }
            if self.sc(next_line) - self.blk_indent > 3 {
                next_line += 1;
                continue;
            }
            if self.sc(next_line) < 0 {
                next_line += 1;
                continue;
            }
            if self.terminates(TERM_PARAGRAPH, next_line, end_line) {
                break;
            }
            next_line += 1;
        }
        let content = py_strip(&self.get_lines(start_line, next_line, self.blk_indent, false)).to_string();
        self.line = next_line;
        self.push("paragraph_open", "p", 1);
        let t = self.push("inline", "", 0);
        self.tokens[t].content = content;
        self.tokens[t].children = Some(Vec::new());
        self.push("paragraph_close", "p", -1);
        self.parent = old_parent;
        true
    }
}

const HEADING_TAGS: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];

/// `^:?-+:?$`
fn header_line_ok(t: &str) -> bool {
    let t = t.strip_prefix(':').unwrap_or(t);
    let t = t.strip_suffix(':').unwrap_or(t);
    !t.is_empty() && t.chars().all(|c| c == '-')
}

fn escaped_split(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut result = Vec::new();
    let mut pos = 0usize;
    let max = chars.len();
    let mut is_escaped = false;
    let mut last_pos = 0usize;
    let mut current = String::new();
    while pos < max {
        let ch = chars[pos];
        if ch == '|' {
            if !is_escaped {
                current.push_str(&chars_to_string(&chars[last_pos..pos]));
                result.push(std::mem::take(&mut current));
                last_pos = pos + 1;
            } else {
                if pos >= 1 && last_pos < pos {
                    current.push_str(&chars_to_string(&chars[last_pos..pos - 1]));
                }
                last_pos = pos;
            }
        }
        is_escaped = ch == '\\';
        pos += 1;
    }
    current.push_str(&chars_to_string(&chars[last_pos.min(max)..]));
    result.push(current);
    result
}

/// markdown-it `normalizeReference`.
fn normalize_reference(s: &str) -> String {
    let collapsed = py_split_ws(py_strip(s)).join(" ");
    collapsed.to_lowercase().to_uppercase()
}

// ---------------------------------------------------------------------------
// Link helpers
// ---------------------------------------------------------------------------

struct DestResult {
    ok: bool,
    pos: i64,
    s: String,
}

fn at(s: &[char], i: i64) -> Option<char> {
    if i < 0 {
        None
    } else {
        s.get(i as usize).copied()
    }
}

fn parse_link_destination(s: &[char], mut pos: i64, max: i64) -> DestResult {
    let start = pos;
    let mut r = DestResult { ok: false, pos: 0, s: String::new() };
    if at(s, pos) == Some('<') {
        pos += 1;
        while pos < max {
            let code = at(s, pos);
            if code == Some('\n') || code == Some('<') {
                return r;
            }
            if code == Some('>') {
                r.pos = pos + 1;
                r.s = unescape_all(&chars_to_string(&s[(start + 1) as usize..pos as usize]));
                r.ok = true;
                return r;
            }
            if code == Some('\\') && pos + 1 < max {
                pos += 2;
                continue;
            }
            pos += 1;
        }
        return r;
    }
    let mut level = 0;
    while pos < max {
        let code = match at(s, pos) {
            None => break,
            Some(c) => c,
        };
        if code == ' ' {
            break;
        }
        if (code as u32) < 0x20 || code as u32 == 0x7F {
            break;
        }
        if code == '\\' && pos + 1 < max {
            if at(s, pos + 1) == Some(' ') {
                break;
            }
            pos += 2;
            continue;
        }
        if code == '(' {
            level += 1;
            if level > 32 {
                return r;
            }
        }
        if code == ')' {
            if level == 0 {
                break;
            }
            level -= 1;
        }
        pos += 1;
    }
    if start == pos {
        return r;
    }
    if level != 0 {
        return r;
    }
    let end = (pos as usize).min(s.len());
    r.s = unescape_all(&chars_to_string(&s[start as usize..end]));
    r.pos = pos;
    r.ok = true;
    r
}

#[derive(Clone)]
struct TitleState {
    ok: bool,
    can_continue: bool,
    pos: i64,
    s: String,
    marker: char,
}

fn parse_link_title(s: &[char], mut start: i64, max: i64, prev: Option<&TitleState>) -> TitleState {
    let mut pos = start;
    let mut st = TitleState { ok: false, can_continue: false, pos: 0, s: String::new(), marker: '\0' };
    if let Some(p) = prev {
        st.s = p.s.clone();
        st.marker = p.marker;
    } else {
        if pos >= max {
            return st;
        }
        let mut marker = match at(s, pos) {
            Some(c @ ('"' | '\'' | '(')) => c,
            _ => return st,
        };
        start += 1;
        pos += 1;
        if marker == '(' {
            marker = ')';
        }
        st.marker = marker;
    }
    let sl = |a: i64, b: i64| -> String {
        let a = (a.max(0) as usize).min(s.len());
        let b = (b.max(0) as usize).min(s.len());
        if a >= b {
            String::new()
        } else {
            chars_to_string(&s[a..b])
        }
    };
    while pos < max {
        let code = at(s, pos);
        if code == Some(st.marker) {
            st.pos = pos + 1;
            st.s.push_str(&unescape_all(&sl(start, pos)));
            st.ok = true;
            return st;
        } else if code == Some('(') && st.marker == ')' {
            return st;
        } else if code == Some('\\') && pos + 1 < max {
            pos += 1;
        }
        pos += 1;
    }
    st.can_continue = true;
    st.s.push_str(&unescape_all(&sl(start, pos)));
    st
}

fn is_valid_entity_code(c: u32) -> bool {
    if (0xD800..=0xDFFF).contains(&c) {
        return false;
    }
    if (0xFDD0..=0xFDEF).contains(&c) {
        return false;
    }
    if (c & 0xFFFF) == 0xFFFF || (c & 0xFFFF) == 0xFFFE {
        return false;
    }
    if c <= 0x08 {
        return false;
    }
    if c == 0x0B {
        return false;
    }
    if (0x0E..=0x1F).contains(&c) {
        return false;
    }
    if (0x7F..=0x9F).contains(&c) {
        return false;
    }
    c <= 0x10FFFF
}

/// markdown-it `unescapeAll`.
fn unescape_all(s: &str) -> String {
    if !s.contains('\\') && !s.contains('&') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\\' && i + 1 < chars.len() && chars[i + 1].is_ascii_punctuation() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if c == '&' && i + 1 < chars.len() && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '#') {
            // [a-z#][a-z0-9]{1,31};
            let mut j = i + 2;
            while j < chars.len() && j - (i + 2) < 31 && chars[j].is_ascii_alphanumeric() {
                j += 1;
            }
            let n = j - (i + 2);
            if n >= 1 && j < chars.len() && chars[j] == ';' {
                let name = chars_to_string(&chars[i + 1..j]);
                out.push_str(&replace_entity_pattern(&chars_to_string(&chars[i..=j]), &name));
                i = j + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn replace_entity_pattern(m: &str, name: &str) -> String {
    if let Some(v) = entities::lookup(name) {
        return v.to_string();
    }
    let mut code: Option<u32> = None;
    if let Some(rest) = name.strip_prefix('#') {
        if !rest.is_empty() && rest.len() <= 8 && rest.chars().all(|c| c.is_ascii_digit()) {
            code = rest.parse().ok();
        } else if let Some(hex) = rest.strip_prefix('x').or_else(|| rest.strip_prefix('X')) {
            if !hex.is_empty() && hex.len() <= 8 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                code = u32::from_str_radix(hex, 16).ok();
            }
        }
    }
    if let Some(c) = code {
        if is_valid_entity_code(c) {
            if let Some(ch) = char::from_u32(c) {
                return ch.to_string();
            }
        }
    }
    m.to_string()
}

// ---------------------------------------------------------------------------
// Inline level
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct Delimiter {
    marker: char,
    length: i64,
    token: i64,
    end: i64,
    open: bool,
    close: bool,
}

struct InlineState<'a> {
    src: Vec<char>,
    env: &'a Env,
    tokens: Vec<Token>,
    pos: i64,
    pos_max: i64,
    level: i64,
    pending: String,
    pending_level: i64,
    cache: HashMap<i64, i64>,
    delim_lists: Vec<Vec<Delimiter>>,
    cur: usize,
    prev: Vec<usize>,
    meta_lists: Vec<usize>,
    backticks: HashMap<i64, i64>,
    backticks_scanned: bool,
    link_level: i64,
}

fn parse_inline(src: Vec<char>, env: &Env) -> Vec<Token> {
    let n = src.len() as i64;
    let mut st = InlineState {
        src,
        env,
        tokens: Vec::new(),
        pos: 0,
        pos_max: n,
        level: 0,
        pending: String::new(),
        pending_level: 0,
        cache: HashMap::new(),
        delim_lists: vec![Vec::new()],
        cur: 0,
        prev: Vec::new(),
        meta_lists: Vec::new(),
        backticks: HashMap::new(),
        backticks_scanned: false,
        link_level: 0,
    };
    st.tokenize();
    // rules2
    st.balance_pairs_all();
    st.strikethrough_post_all();
    st.emphasis_post_all();
    st.fragments_join();
    st.tokens
}

const TERMINATORS: &[char] = &[
    '\n', '!', '#', '$', '%', '&', '*', '+', '-', ':', '<', '=', '>', '@', '[', '\\', ']', '^', '_', '`', '{', '}',
    '~',
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum IRule {
    Text,
    Newline,
    Escape,
    Backticks,
    Strikethrough,
    Emphasis,
    Link,
    Image,
    Autolink,
    Entity,
}

const IRULES: &[IRule] = &[
    IRule::Text,
    IRule::Newline,
    IRule::Escape,
    IRule::Backticks,
    IRule::Strikethrough,
    IRule::Emphasis,
    IRule::Link,
    IRule::Image,
    IRule::Autolink,
    IRule::Entity,
];

fn is_md_ascii_punct(c: char) -> bool {
    c.is_ascii_punctuation()
}

fn is_white_space(c: char) -> bool {
    let cp = c as u32;
    if (0x2000..=0x200A).contains(&cp) {
        return true;
    }
    matches!(cp, 0x09 | 0x0A | 0x0B | 0x0C | 0x0D | 0x20 | 0xA0 | 0x1680 | 0x202F | 0x205F | 0x3000)
}

impl<'a> InlineState<'a> {
    fn c(&self, i: i64) -> Option<char> {
        if i < 0 {
            None
        } else {
            self.src.get(i as usize).copied()
        }
    }

    fn slice(&self, a: i64, b: i64) -> String {
        let n = self.src.len() as i64;
        let a = a.clamp(0, n) as usize;
        let b = b.clamp(0, n) as usize;
        if a >= b {
            String::new()
        } else {
            chars_to_string(&self.src[a..b])
        }
    }

    fn push_pending(&mut self) {
        let mut t = Token::new("text", "", 0);
        t.content = std::mem::take(&mut self.pending);
        t.level = self.pending_level;
        self.tokens.push(t);
    }

    fn push(&mut self, ty: &'static str, tag: &'static str, nesting: i8) -> usize {
        if !self.pending.is_empty() {
            self.push_pending();
        }
        let mut t = Token::new(ty, tag, nesting);
        if nesting < 0 {
            self.level -= 1;
            self.cur = self.prev.pop().unwrap_or(0);
        }
        t.level = self.level;
        if nesting > 0 {
            self.level += 1;
            self.prev.push(self.cur);
            self.delim_lists.push(Vec::new());
            self.cur = self.delim_lists.len() - 1;
            self.meta_lists.push(self.cur);
        }
        self.pending_level = self.level;
        self.tokens.push(t);
        self.tokens.len() - 1
    }

    fn scan_delims(&self, start: i64, can_split_word: bool) -> (bool, bool, i64) {
        let mut pos = start;
        let max = self.pos_max;
        let marker = self.c(start);
        let last_char = if start > 0 { self.c(start - 1).unwrap_or(' ') } else { ' ' };
        while pos < max && self.c(pos) == marker {
            pos += 1;
        }
        let count = pos - start;
        let next_char = if pos < max { self.c(pos).unwrap_or(' ') } else { ' ' };
        let is_last_punct = is_md_ascii_punct(last_char) || unicode::is_punct_or_symbol(last_char);
        let is_next_punct = is_md_ascii_punct(next_char) || unicode::is_punct_or_symbol(next_char);
        let is_last_ws = is_white_space(last_char);
        let is_next_ws = is_white_space(next_char);
        let left_flanking = !(is_next_ws || (is_next_punct && !(is_last_ws || is_last_punct)));
        let right_flanking = !(is_last_ws || (is_last_punct && !(is_next_ws || is_next_punct)));
        let can_open = left_flanking && (can_split_word || !right_flanking || is_last_punct);
        let can_close = right_flanking && (can_split_word || !left_flanking || is_next_punct);
        (can_open, can_close, count)
    }

    fn run(&mut self, r: IRule, silent: bool) -> bool {
        match r {
            IRule::Text => self.r_text(silent),
            IRule::Newline => self.r_newline(silent),
            IRule::Escape => self.r_escape(silent),
            IRule::Backticks => self.r_backticks(silent),
            IRule::Strikethrough => self.r_strikethrough(silent),
            IRule::Emphasis => self.r_emphasis(silent),
            IRule::Link => self.r_link(silent),
            IRule::Image => self.r_image(silent),
            IRule::Autolink => self.r_autolink(silent),
            IRule::Entity => self.r_entity(silent),
        }
    }

    fn skip_token(&mut self) {
        let pos = self.pos;
        if let Some(&p) = self.cache.get(&pos) {
            self.pos = p;
            return;
        }
        let mut ok = false;
        if self.level < MAX_NESTING {
            for &r in IRULES {
                self.level += 1;
                ok = self.run(r, true);
                self.level -= 1;
                if ok {
                    break;
                }
            }
        } else {
            self.pos = self.pos_max;
        }
        if !ok {
            self.pos += 1;
        }
        self.cache.insert(pos, self.pos);
    }

    fn tokenize(&mut self) {
        let end = self.pos_max;
        while self.pos < end {
            let mut ok = false;
            if self.level < MAX_NESTING {
                for &r in IRULES {
                    ok = self.run(r, false);
                    if ok {
                        break;
                    }
                }
            }
            if ok {
                if self.pos >= end {
                    break;
                }
                continue;
            }
            if let Some(c) = self.c(self.pos) {
                self.pending.push(c);
            }
            self.pos += 1;
        }
        if !self.pending.is_empty() {
            self.push_pending();
        }
    }

    fn r_text(&mut self, silent: bool) -> bool {
        let mut pos = self.pos;
        let n = self.src.len() as i64;
        let mut found = None;
        while pos < n {
            if TERMINATORS.contains(&self.src[pos as usize]) {
                found = Some(pos);
                break;
            }
            pos += 1;
        }
        let pos = found.unwrap_or(self.pos_max);
        if pos == self.pos {
            return false;
        }
        if !silent {
            let s = self.slice(self.pos, pos);
            self.pending.push_str(&s);
        }
        self.pos = pos;
        true
    }

    fn r_newline(&mut self, silent: bool) -> bool {
        let mut pos = self.pos;
        if self.c(pos) != Some('\n') {
            return false;
        }
        let max = self.pos_max;
        if !silent {
            let trailing = self.pending.len() - self.pending.trim_end_matches(' ').len();
            if trailing >= 2 {
                let keep = self.pending.len() - trailing;
                self.pending.truncate(keep);
                self.push("hardbreak", "br", 0);
            } else if trailing == 1 {
                self.pending.pop();
                self.push("softbreak", "br", 0);
            } else {
                self.push("softbreak", "br", 0);
            }
        }
        pos += 1;
        while pos < max && is_sp(self.c(pos)) {
            pos += 1;
        }
        self.pos = pos;
        true
    }

    fn r_escape(&mut self, silent: bool) -> bool {
        let mut pos = self.pos;
        let max = self.pos_max;
        if self.c(pos) != Some('\\') {
            return false;
        }
        pos += 1;
        if pos >= max {
            return false;
        }
        let ch1 = match self.c(pos) {
            Some(c) => c,
            None => return false,
        };
        if ch1 == '\n' {
            if !silent {
                self.push("hardbreak", "br", 0);
            }
            pos += 1;
            while pos < max {
                if !is_sp(self.c(pos)) {
                    break;
                }
                pos += 1;
            }
            self.pos = pos;
            return true;
        }
        let orig = format!("\\{}", ch1);
        if !silent {
            let t = self.push("text_special", "", 0);
            self.tokens[t].content = if ch1.is_ascii_punctuation() { ch1.to_string() } else { orig.clone() };
            self.tokens[t].markup = orig;
            self.tokens[t].info = "escape".into();
        }
        self.pos = pos + 1;
        true
    }

    fn r_backticks(&mut self, silent: bool) -> bool {
        let mut pos = self.pos;
        if self.c(pos) != Some('`') {
            return false;
        }
        let start = pos;
        pos += 1;
        let max = self.pos_max;
        while pos < max && self.c(pos) == Some('`') {
            pos += 1;
        }
        let marker = self.slice(start, pos);
        let opener_len = pos - start;
        if self.backticks_scanned && *self.backticks.get(&opener_len).unwrap_or(&0) <= start {
            if !silent {
                self.pending.push_str(&marker);
            }
            self.pos += opener_len;
            return true;
        }
        let mut match_end = pos;
        loop {
            // str.index("`", matchEnd) over the whole source
            let n = self.src.len() as i64;
            let mut ms = match_end.max(0);
            while ms < n && self.src[ms as usize] != '`' {
                ms += 1;
            }
            if ms >= n {
                break;
            }
            let match_start = ms;
            match_end = match_start + 1;
            while match_end < max && self.c(match_end) == Some('`') {
                match_end += 1;
            }
            let closer_len = match_end - match_start;
            if closer_len == opener_len {
                if !silent {
                    let mut content = self.slice(pos, match_start).replace('\n', " ");
                    if content.starts_with(' ') && content.ends_with(' ') && !py_strip(&content).is_empty() {
                        content = content[1..content.len() - 1].to_string();
                    }
                    let t = self.push("code_inline", "code", 0);
                    self.tokens[t].markup = marker;
                    self.tokens[t].content = content;
                }
                self.pos = match_end;
                return true;
            }
            self.backticks.insert(closer_len, match_start);
        }
        self.backticks_scanned = true;
        if !silent {
            self.pending.push_str(&marker);
        }
        self.pos += opener_len;
        true
    }

    fn r_strikethrough(&mut self, silent: bool) -> bool {
        let start = self.pos;
        if silent {
            return false;
        }
        if self.c(start) != Some('~') {
            return false;
        }
        let (can_open, can_close, scanned_len) = self.scan_delims(self.pos, true);
        let mut length = scanned_len;
        if length < 2 {
            return false;
        }
        if length % 2 == 1 {
            let t = self.push("text", "", 0);
            self.tokens[t].content = "~".into();
            length -= 1;
        }
        let mut i = 0;
        while i < length {
            let t = self.push("text", "", 0);
            self.tokens[t].content = "~~".into();
            let tok = self.tokens.len() as i64 - 1;
            self.delim_lists[self.cur].push(Delimiter {
                marker: '~',
                length: 0,
                token: tok,
                end: -1,
                open: can_open,
                close: can_close,
            });
            i += 2;
        }
        self.pos += scanned_len;
        true
    }

    fn r_emphasis(&mut self, silent: bool) -> bool {
        let start = self.pos;
        let marker = match self.c(start) {
            Some(c) => c,
            None => return false,
        };
        if silent {
            return false;
        }
        if marker != '_' && marker != '*' {
            return false;
        }
        let (can_open, can_close, length) = self.scan_delims(self.pos, marker == '*');
        for _ in 0..length {
            let t = self.push("text", "", 0);
            self.tokens[t].content = marker.to_string();
            let tok = self.tokens.len() as i64 - 1;
            self.delim_lists[self.cur].push(Delimiter {
                marker,
                length,
                token: tok,
                end: -1,
                open: can_open,
                close: can_close,
            });
        }
        self.pos += length;
        true
    }

    fn parse_link_label(&mut self, start: i64, disable_nested: bool) -> i64 {
        let mut label_end = -1;
        let old_pos = self.pos;
        let mut found = false;
        self.pos = start + 1;
        let mut level = 1;
        while self.pos < self.pos_max {
            let marker = self.c(self.pos);
            if marker == Some(']') {
                level -= 1;
                if level == 0 {
                    found = true;
                    break;
                }
            }
            let prev_pos = self.pos;
            self.skip_token();
            if marker == Some('[') {
                if prev_pos == self.pos - 1 {
                    level += 1;
                } else if disable_nested {
                    self.pos = old_pos;
                    return -1;
                }
            }
        }
        if found {
            label_end = self.pos;
        }
        self.pos = old_pos;
        label_end
    }

    fn skip_ws_nl(&self, mut pos: i64, max: i64) -> i64 {
        while pos < max {
            let ch = self.c(pos);
            if !is_sp(ch) && ch != Some('\n') {
                break;
            }
            pos += 1;
        }
        pos
    }

    fn r_link(&mut self, silent: bool) -> bool {
        let mut href = String::new();
        let mut title = String::new();
        let mut label: Option<String> = None;
        let old_pos = self.pos;
        let max = self.pos_max;
        let mut parse_reference = true;
        if self.c(self.pos) != Some('[') {
            return false;
        }
        let label_start = self.pos + 1;
        let label_end = self.parse_link_label(self.pos, true);
        if label_end < 0 {
            return false;
        }
        let mut pos = label_end + 1;
        if pos < max && self.c(pos) == Some('(') {
            parse_reference = false;
            pos += 1;
            pos = self.skip_ws_nl(pos, max);
            if pos >= max {
                return false;
            }
            let res = parse_link_destination(&self.src, pos, self.pos_max);
            if res.ok {
                href = normalize_link(&res.s);
                if validate_link(&href) {
                    pos = res.pos;
                } else {
                    href = String::new();
                }
                let start = pos;
                pos = self.skip_ws_nl(pos, max);
                let res = parse_link_title(&self.src, pos, self.pos_max, None);
                if pos < max && start != pos && res.ok {
                    title = res.s;
                    pos = res.pos;
                    pos = self.skip_ws_nl(pos, max);
                }
            }
            if pos >= max || self.c(pos) != Some(')') {
                parse_reference = true;
            }
            pos += 1;
        }
        if parse_reference {
            let refs = match &self.env.references {
                None => return false,
                Some(r) => r,
            };
            if pos < max && self.c(pos) == Some('[') {
                let start = pos + 1;
                let p = self.parse_link_label(pos, false);
                if p >= 0 {
                    label = Some(self.slice(start, p));
                    pos = p + 1;
                } else {
                    pos = label_end + 1;
                }
            } else {
                pos = label_end + 1;
            }
            let l = match label {
                Some(ref l) if !l.is_empty() => l.clone(),
                _ => self.slice(label_start, label_end),
            };
            let key = normalize_reference(&l);
            match refs.get(&key) {
                None => {
                    self.pos = old_pos;
                    return false;
                }
                Some((h, t)) => {
                    href = h.clone();
                    title = t.clone();
                }
            }
        }
        if !silent {
            self.pos = label_start;
            self.pos_max = label_end;
            let t = self.push("link_open", "a", 1);
            self.tokens[t].attrs = vec![("href", href)];
            if !title.is_empty() {
                self.tokens[t].attr_set("title", title);
            }
            self.link_level += 1;
            self.tokenize();
            self.link_level -= 1;
            self.push("link_close", "a", -1);
        }
        self.pos = pos;
        self.pos_max = max;
        true
    }

    fn r_image(&mut self, silent: bool) -> bool {
        let mut label: Option<String> = None;
        let href;
        let title;
        let old_pos = self.pos;
        let max = self.pos_max;
        if self.c(self.pos) != Some('!') {
            return false;
        }
        if self.pos + 1 < self.pos_max && self.c(self.pos + 1) != Some('[') {
            return false;
        }
        let label_start = self.pos + 2;
        let label_end = self.parse_link_label(self.pos + 1, false);
        if label_end < 0 {
            return false;
        }
        let mut pos = label_end + 1;
        if pos < max && self.c(pos) == Some('(') {
            pos += 1;
            pos = self.skip_ws_nl(pos, max);
            if pos >= max {
                return false;
            }
            let res = parse_link_destination(&self.src, pos, self.pos_max);
            let mut h = String::new();
            if res.ok {
                h = normalize_link(&res.s);
                if validate_link(&h) {
                    pos = res.pos;
                } else {
                    h = String::new();
                }
            }
            href = h;
            let start = pos;
            pos = self.skip_ws_nl(pos, max);
            let res = parse_link_title(&self.src, pos, self.pos_max, None);
            if pos < max && start != pos && res.ok {
                title = res.s;
                pos = res.pos;
                pos = self.skip_ws_nl(pos, max);
            } else {
                title = String::new();
            }
            if pos >= max || self.c(pos) != Some(')') {
                self.pos = old_pos;
                return false;
            }
            pos += 1;
        } else {
            let refs = match &self.env.references {
                None => return false,
                Some(r) => r,
            };
            if pos < max && self.c(pos) == Some('[') {
                let start = pos + 1;
                let p = self.parse_link_label(pos, false);
                if p >= 0 {
                    label = Some(self.slice(start, p));
                    pos = p + 1;
                } else {
                    pos = label_end + 1;
                }
            } else {
                pos = label_end + 1;
            }
            let l = match label {
                Some(ref l) if !l.is_empty() => l.clone(),
                _ => self.slice(label_start, label_end),
            };
            let key = normalize_reference(&l);
            match refs.get(&key) {
                None => {
                    self.pos = old_pos;
                    return false;
                }
                Some((h, t)) => {
                    href = h.clone();
                    title = t.clone();
                }
            }
        }
        if !silent {
            let content = self.slice(label_start, label_end);
            let children = parse_inline(content.chars().collect(), self.env);
            let t = self.push("image", "img", 0);
            self.tokens[t].attrs = vec![("src", href), ("alt", String::new())];
            self.tokens[t].children = if children.is_empty() { None } else { Some(children) };
            self.tokens[t].content = content;
            if !title.is_empty() {
                self.tokens[t].attr_set("title", title);
            }
        }
        self.pos = pos;
        self.pos_max = max;
        true
    }

    fn r_autolink(&mut self, silent: bool) -> bool {
        let mut pos = self.pos;
        if self.c(pos) != Some('<') {
            return false;
        }
        let start = self.pos;
        let max = self.pos_max;
        loop {
            pos += 1;
            if pos >= max {
                return false;
            }
            let ch = self.c(pos);
            if ch == Some('<') {
                return false;
            }
            if ch == Some('>') {
                break;
            }
        }
        let url_chars: Vec<char> = self.src[(start + 1) as usize..pos as usize].to_vec();
        let url = chars_to_string(&url_chars);
        let full = if is_autolink_url(&url_chars) {
            normalize_link(&url)
        } else if is_email(&url_chars) {
            normalize_link(&format!("mailto:{}", url))
        } else {
            return false;
        };
        if !validate_link(&full) {
            return false;
        }
        if !silent {
            let t = self.push("link_open", "a", 1);
            self.tokens[t].attrs = vec![("href", full)];
            self.tokens[t].markup = "autolink".into();
            self.tokens[t].info = "auto".into();
            let t = self.push("text", "", 0);
            self.tokens[t].content = normalize_link_text(&url);
            let t = self.push("link_close", "a", -1);
            self.tokens[t].markup = "autolink".into();
            self.tokens[t].info = "auto".into();
        }
        self.pos += url_chars.len() as i64 + 2;
        true
    }

    fn r_entity(&mut self, silent: bool) -> bool {
        let pos = self.pos;
        let max = self.pos_max;
        if self.c(pos) != Some('&') {
            return false;
        }
        if pos + 1 >= max {
            return false;
        }
        let rest = &self.src[pos as usize..];
        if self.c(pos + 1) == Some('#') {
            // ^&#((?:x[a-f0-9]{1,6}|[0-9]{1,7}));  (ignorecase)
            let mut m: Option<(usize, u32)> = None;
            if rest.len() > 2 && (rest[2] == 'x' || rest[2] == 'X') {
                let mut j = 3;
                while j < rest.len() && j - 3 < 6 && rest[j].is_ascii_hexdigit() {
                    j += 1;
                }
                if j > 3 && j < rest.len() && rest[j] == ';' {
                    let v = u32::from_str_radix(&chars_to_string(&rest[3..j]), 16).unwrap_or(0);
                    m = Some((j + 1, v));
                }
            }
            if m.is_none() {
                let mut j = 2;
                while j < rest.len() && j - 2 < 7 && rest[j].is_ascii_digit() {
                    j += 1;
                }
                if j > 2 && j < rest.len() && rest[j] == ';' {
                    let v = chars_to_string(&rest[2..j]).parse::<u32>().unwrap_or(0);
                    m = Some((j + 1, v));
                }
            }
            if let Some((len, code)) = m {
                if !silent {
                    let ch = if is_valid_entity_code(code) {
                        char::from_u32(code).unwrap_or('\u{FFFD}')
                    } else {
                        '\u{FFFD}'
                    };
                    let markup = chars_to_string(&rest[..len]);
                    let t = self.push("text_special", "", 0);
                    self.tokens[t].content = ch.to_string();
                    self.tokens[t].markup = markup;
                    self.tokens[t].info = "entity".into();
                }
                self.pos += len as i64;
                return true;
            }
        } else {
            // ^&([a-z][a-z0-9]{1,31});
            if rest.len() > 1 && rest[1].is_ascii_alphabetic() {
                let mut j = 2;
                while j < rest.len() && j - 2 < 31 && rest[j].is_ascii_alphanumeric() {
                    j += 1;
                }
                if j > 2 && j < rest.len() && rest[j] == ';' {
                    let name = chars_to_string(&rest[1..j]);
                    if let Some(v) = entities::lookup(&name) {
                        if !silent {
                            let markup = chars_to_string(&rest[..=j]);
                            let t = self.push("text_special", "", 0);
                            self.tokens[t].content = v.to_string();
                            self.tokens[t].markup = markup;
                            self.tokens[t].info = "entity".into();
                        }
                        self.pos += j as i64 + 1;
                        return true;
                    }
                }
            }
        }
        false
    }

    // --- rules2 ---

    fn all_lists(&self) -> Vec<usize> {
        let mut v = vec![self.cur];
        v.extend(self.meta_lists.iter().copied());
        v
    }

    fn balance_pairs_all(&mut self) {
        for li in self.all_lists() {
            process_delimiters(&mut self.delim_lists[li]);
        }
    }

    fn strikethrough_post_all(&mut self) {
        for li in self.all_lists() {
            let delims = self.delim_lists[li].clone();
            strikethrough_post(&mut self.tokens, &delims);
        }
    }

    fn emphasis_post_all(&mut self) {
        for li in self.all_lists() {
            let delims = self.delim_lists[li].clone();
            emphasis_post(&mut self.tokens, &delims);
        }
    }

    fn fragments_join(&mut self) {
        let mut level = 0;
        let tokens = std::mem::take(&mut self.tokens);
        let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
        let mut prev_was_text_run = false;
        for mut t in tokens {
            if t.nesting < 0 {
                level -= 1;
            }
            t.level = level;
            if t.nesting > 0 {
                level += 1;
            }
            if t.ty == "text" && prev_was_text_run {
                if let Some(last) = out.last_mut() {
                    last.content.push_str(&t.content);
                    last.level = level;
                    continue;
                }
            }
            prev_was_text_run = t.ty == "text";
            out.push(t);
        }
        self.tokens = out;
    }
}

fn process_delimiters(delims: &mut [Delimiter]) {
    if delims.is_empty() {
        return;
    }
    let mut openers_bottom: HashMap<char, [i64; 6]> = HashMap::new();
    let max = delims.len();
    let mut header_idx = 0usize;
    let mut last_token_idx: i64 = -2;
    let mut jumps: Vec<i64> = Vec::with_capacity(max);
    let mut closer_idx = 0usize;
    while closer_idx < max {
        let closer = delims[closer_idx];
        jumps.push(0);
        if delims[header_idx].marker != closer.marker || last_token_idx != closer.token - 1 {
            header_idx = closer_idx;
        }
        last_token_idx = closer.token;
        if !closer.close {
            closer_idx += 1;
            continue;
        }
        let ob = openers_bottom.entry(closer.marker).or_insert([-1; 6]);
        let slot = (if closer.open { 3 } else { 0 }) + (closer.length % 3) as usize;
        let min_opener_idx = ob[slot];
        let mut opener_idx: i64 = header_idx as i64 - jumps[header_idx] - 1;
        let mut new_min_opener_idx = opener_idx;
        while opener_idx > min_opener_idx {
            let opener = delims[opener_idx as usize];
            if opener.marker != closer.marker {
                opener_idx -= jumps[opener_idx as usize] + 1;
                continue;
            }
            if opener.open && opener.end < 0 {
                let closer_now = delims[closer_idx];
                let mut is_odd_match = false;
                if (opener.close || closer_now.open)
                    && (opener.length + closer_now.length) % 3 == 0
                    && (opener.length % 3 != 0 || closer_now.length % 3 != 0)
                {
                    is_odd_match = true;
                }
                if !is_odd_match {
                    let last_jump = if opener_idx > 0 && !delims[opener_idx as usize - 1].open {
                        jumps[opener_idx as usize - 1] + 1
                    } else {
                        0
                    };
                    jumps[closer_idx] = closer_idx as i64 - opener_idx + last_jump;
                    jumps[opener_idx as usize] = last_jump;
                    delims[closer_idx].open = false;
                    delims[opener_idx as usize].end = closer_idx as i64;
                    delims[opener_idx as usize].close = false;
                    new_min_opener_idx = -1;
                    last_token_idx = -2;
                    break;
                }
            }
            opener_idx -= jumps[opener_idx as usize] + 1;
        }
        if new_min_opener_idx != -1 {
            let c = delims[closer_idx];
            let slot = (if c.open { 3 } else { 0 }) + (c.length % 3) as usize;
            openers_bottom.get_mut(&c.marker).unwrap()[slot] = new_min_opener_idx;
        }
        closer_idx += 1;
    }
}

fn strikethrough_post(tokens: &mut [Token], delims: &[Delimiter]) {
    let mut lone_markers: Vec<usize> = Vec::new();
    for start in delims.iter() {
        if start.marker != '~' || start.end == -1 {
            continue;
        }
        let end = delims[start.end as usize];
        let (si, ei) = (start.token as usize, end.token as usize);
        let markup = tokens[si].content.clone();
        tokens[si].ty = "s_open";
        tokens[si].tag = "s";
        tokens[si].nesting = 1;
        tokens[si].markup = markup.clone();
        tokens[si].content = String::new();
        tokens[ei].ty = "s_close";
        tokens[ei].tag = "s";
        tokens[ei].nesting = -1;
        tokens[ei].markup = markup;
        tokens[ei].content = String::new();
        if ei >= 1 && tokens[ei - 1].ty == "text" && tokens[ei - 1].content == "~" {
            lone_markers.push(ei - 1);
        }
    }
    while let Some(i) = lone_markers.pop() {
        let mut j = i + 1;
        while j < tokens.len() && tokens[j].ty == "s_close" {
            j += 1;
        }
        j -= 1;
        if i != j {
            tokens.swap(i, j);
        }
    }
}

fn emphasis_post(tokens: &mut [Token], delims: &[Delimiter]) {
    let mut i = delims.len() as i64 - 1;
    while i >= 0 {
        let start = delims[i as usize];
        if start.marker != '_' && start.marker != '*' {
            i -= 1;
            continue;
        }
        if start.end == -1 {
            i -= 1;
            continue;
        }
        let end = delims[start.end as usize];
        let is_strong = i > 0
            && delims[i as usize - 1].end == start.end + 1
            && delims[i as usize - 1].marker == start.marker
            && delims[i as usize - 1].token == start.token - 1
            && delims[(start.end + 1) as usize].token == end.token + 1;
        let ch = start.marker.to_string();
        let markup = if is_strong { format!("{}{}", ch, ch) } else { ch };
        let (ty_o, ty_c, tag) =
            if is_strong { ("strong_open", "strong_close", "strong") } else { ("em_open", "em_close", "em") };
        let t = &mut tokens[start.token as usize];
        t.ty = ty_o;
        t.tag = tag;
        t.nesting = 1;
        t.markup = markup.clone();
        t.content = String::new();
        let t = &mut tokens[end.token as usize];
        t.ty = ty_c;
        t.tag = tag;
        t.nesting = -1;
        t.markup = markup;
        t.content = String::new();
        if is_strong {
            let a = delims[i as usize - 1].token as usize;
            let b = delims[(start.end + 1) as usize].token as usize;
            tokens[a].content = String::new();
            tokens[b].content = String::new();
            i -= 1;
        }
        i -= 1;
    }
}

/// `^([a-zA-Z][a-zA-Z0-9+.\-]{1,31}):([^<>\x00-\x20]*)$`
fn is_autolink_url(u: &[char]) -> bool {
    let u = if u.last() == Some(&'\n') { &u[..u.len() - 1] } else { u };
    if u.is_empty() || !u[0].is_ascii_alphabetic() {
        return false;
    }
    let mut i = 1;
    while i < u.len() && (u[i].is_ascii_alphanumeric() || matches!(u[i], '+' | '.' | '-')) {
        i += 1;
    }
    // scheme length 2..=32; the char class excludes ':' so a longer run cannot backtrack to a ':'
    if !(2..=32).contains(&i) || i >= u.len() || u[i] != ':' {
        return false;
    }
    u[i + 1..].iter().all(|&c| !(c == '<' || c == '>' || (c as u32) <= 0x20))
}

/// The markdown-it EMAIL_RE.
fn is_email(u: &[char]) -> bool {
    let u = if u.last() == Some(&'\n') { &u[..u.len() - 1] } else { u };
    let at = match u.iter().position(|&c| c == '@') {
        Some(a) => a,
        None => return false,
    };
    let local = &u[..at];
    if local.is_empty()
        || !local.iter().all(|&c| c.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(c))
    {
        return false;
    }
    let domain = &u[at + 1..];
    if domain.is_empty() {
        return false;
    }
    domain.split(|&c| c == '.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.iter().all(|&c| c.is_ascii_alphanumeric() || c == '-')
            && label[0].is_ascii_alphanumeric()
            && label[label.len() - 1].is_ascii_alphanumeric()
    })
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

fn render_attrs(t: &Token) -> String {
    let mut s = String::new();
    for (k, v) in &t.attrs {
        s.push(' ');
        s.push_str(&escape_html(k));
        s.push_str("=\"");
        s.push_str(&escape_html(v));
        s.push('"');
    }
    s
}

fn render_token(tokens: &[Token], idx: usize) -> String {
    let t = &tokens[idx];
    if t.hidden {
        return String::new();
    }
    let mut r = String::new();
    if t.block && t.nesting != -1 && idx > 0 && tokens[idx - 1].hidden {
        r.push('\n');
    }
    r.push_str(if t.nesting == -1 { "</" } else { "<" });
    r.push_str(t.tag);
    r.push_str(&render_attrs(t));
    if t.nesting == 0 {
        r.push_str(" /");
    }
    let mut need_lf = false;
    if t.block {
        need_lf = true;
        if t.nesting == 1 && idx + 1 < tokens.len() {
            let n = &tokens[idx + 1];
            if n.ty == "inline" || n.hidden || (n.nesting == -1 && n.tag == t.tag) {
                need_lf = false;
            }
        }
    }
    r.push_str(if need_lf { ">\n" } else { ">" });
    r
}

fn render_inline_as_text(tokens: &[Token]) -> String {
    let mut r = String::new();
    for t in tokens {
        match t.ty {
            "text" => r.push_str(&t.content),
            "image" => {
                if let Some(c) = &t.children {
                    r.push_str(&render_inline_as_text(c));
                }
            }
            "softbreak" => r.push('\n'),
            _ => {}
        }
    }
    r
}

fn render_rule(tokens: &mut [Token], idx: usize) -> String {
    match tokens[idx].ty {
        "code_inline" => {
            let t = &tokens[idx];
            format!("<code{}>{}</code>", render_attrs(t), escape_html(&t.content))
        }
        "code_block" => {
            let t = &tokens[idx];
            format!("<pre{}><code>{}</code></pre>\n", render_attrs(t), escape_html(&t.content))
        }
        "fence" => {
            let t = &tokens[idx];
            let info = if t.info.is_empty() { String::new() } else { py_strip(&unescape_all(&t.info)).to_string() };
            let highlighted = escape_html(&t.content);
            if !info.is_empty() {
                let lang = py_split_ws(&info).first().map(|s| s.to_string()).unwrap_or_default();
                let mut tmp = Token::new("", "", 0);
                tmp.attrs = t.attrs.clone();
                tmp.attr_set("class", format!("language-{}", lang));
                format!("<pre><code{}>{}</code></pre>\n", render_attrs(&tmp), highlighted)
            } else {
                format!("<pre><code{}>{}</code></pre>\n", render_attrs(t), highlighted)
            }
        }
        "image" => {
            let alt = match &tokens[idx].children {
                Some(c) if !c.is_empty() => render_inline_as_text(c),
                _ => String::new(),
            };
            tokens[idx].attr_set("alt", alt);
            render_token(tokens, idx)
        }
        "hardbreak" => "<br />\n".to_string(),
        "softbreak" => "\n".to_string(),
        "text" => escape_html(&tokens[idx].content),
        _ => render_token(tokens, idx),
    }
}

fn render_inline(tokens: &mut [Token]) -> String {
    let mut r = String::new();
    for i in 0..tokens.len() {
        r.push_str(&render_rule(tokens, i));
    }
    r
}

fn render_tokens(tokens: &mut [Token]) -> String {
    let mut r = String::new();
    for i in 0..tokens.len() {
        if tokens[i].ty == "inline" {
            if let Some(mut c) = tokens[i].children.take() {
                if !c.is_empty() {
                    r.push_str(&render_inline(&mut c));
                }
                tokens[i].children = Some(c);
            }
        } else {
            r.push_str(&render_rule(tokens, i));
        }
    }
    r
}
