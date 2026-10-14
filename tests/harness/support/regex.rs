//! A small backtracking regex engine for the `pattern` keywords in
//! spec/content-schema.json, so the schema file stays the single source of
//! those rules (ADR 0010). Supports `^ $ . \d \w \s` and escaped literals,
//! classes `[a-z0-9-]` and `[^...]`, groups with `|`, and the quantifiers
//! `? * + {n} {n,} {n,m}`. Unsupported syntax panics rather than guessing.

#[derive(Debug, Clone)]
enum Node {
    Start,
    End,
    Any,
    Char(char),
    Class(Vec<(char, char)>, bool),
    Group(Vec<Vec<Node>>),
    Repeat(Box<Node>, usize, usize),
}

pub struct Regex {
    alts: Vec<Vec<Node>>,
}

impl Regex {
    pub fn new(pattern: &str) -> Regex {
        let chars: Vec<char> = pattern.chars().collect();
        let mut pos = 0;
        let alts = parse_alts(&chars, &mut pos);
        assert!(pos == chars.len(), "unbalanced ')' in {pattern:?}");
        Regex { alts }
    }

    /// JSON Schema semantics: the pattern may match anywhere in `s`.
    pub fn is_match(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        (0..=chars.len()).any(|start| self.alts.iter().any(|seq| match_seq(seq, &chars, start, &mut |_| true)))
    }
}

fn parse_alts(p: &[char], pos: &mut usize) -> Vec<Vec<Node>> {
    let mut alts = vec![Vec::new()];
    while *pos < p.len() {
        let c = p[*pos];
        *pos += 1;
        let atom = match c {
            '|' => {
                alts.push(Vec::new());
                continue;
            }
            ')' => {
                *pos -= 1;
                break;
            }
            '(' => {
                let inner = parse_alts(p, pos);
                assert!(p.get(*pos) == Some(&')'), "unclosed '('");
                *pos += 1;
                Node::Group(inner)
            }
            '^' => Node::Start,
            '$' => Node::End,
            '.' => Node::Any,
            '[' => parse_class(p, pos),
            '\\' => {
                let e = p[*pos];
                *pos += 1;
                escape(e)
            }
            '?' | '*' | '+' | '{' => panic!("quantifier {c:?} with nothing to repeat"),
            c => Node::Char(c),
        };
        let atom = parse_quantifier(p, pos, atom);
        alts.last_mut().unwrap().push(atom);
    }
    alts
}

fn escape(e: char) -> Node {
    match e {
        'd' => Node::Class(vec![('0', '9')], false),
        'w' => Node::Class(vec![('a', 'z'), ('A', 'Z'), ('0', '9'), ('_', '_')], false),
        's' => Node::Class(vec![(' ', ' '), ('\t', '\r')], false),
        c if c.is_ascii_punctuation() => Node::Char(c),
        c => panic!("unsupported escape \\{c}"),
    }
}

fn parse_class(p: &[char], pos: &mut usize) -> Node {
    let negated = p.get(*pos) == Some(&'^');
    if negated {
        *pos += 1;
    }
    let mut ranges = Vec::new();
    loop {
        let c = *p.get(*pos).expect("unclosed '['");
        *pos += 1;
        if c == ']' && !ranges.is_empty() {
            break;
        }
        let lo = if c == '\\' {
            let e = p[*pos];
            *pos += 1;
            match escape(e) {
                Node::Char(c) => c,
                Node::Class(rs, _) => {
                    ranges.extend(rs);
                    continue;
                }
                _ => unreachable!(),
            }
        } else {
            c
        };
        if p.get(*pos) == Some(&'-') && p.get(*pos + 1).is_some_and(|c| *c != ']') {
            let hi = p[*pos + 1];
            *pos += 2;
            ranges.push((lo, hi));
        } else {
            ranges.push((lo, lo));
        }
    }
    Node::Class(ranges, negated)
}

fn parse_quantifier(p: &[char], pos: &mut usize, atom: Node) -> Node {
    let (min, max) = match p.get(*pos) {
        Some('?') => (0, 1),
        Some('*') => (0, usize::MAX),
        Some('+') => (1, usize::MAX),
        Some('{') => {
            let close = p[*pos..].iter().position(|c| *c == '}').expect("unclosed '{'") + *pos;
            let body: String = p[*pos + 1..close].iter().collect();
            let (lo, hi) = match body.split_once(',') {
                None => (body.parse().unwrap(), body.parse().unwrap()),
                Some((lo, "")) => (lo.parse().unwrap(), usize::MAX),
                Some((lo, hi)) => (lo.parse().unwrap(), hi.parse().unwrap()),
            };
            *pos = close;
            (lo, hi)
        }
        _ => return atom,
    };
    *pos += 1;
    assert!(
        !matches!(p.get(*pos), Some('?' | '+')),
        "lazy/possessive quantifiers are unsupported"
    );
    Node::Repeat(Box::new(atom), min, max)
}

/// Match `seq` at `i`, then call `k` with the end position (continuation style).
fn match_seq(seq: &[Node], s: &[char], i: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
    let Some((first, rest)) = seq.split_first() else {
        return k(i);
    };
    match_node(first, s, i, &mut |j| match_seq(rest, s, j, k))
}

fn match_node(n: &Node, s: &[char], i: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
    match n {
        Node::Start => i == 0 && k(i),
        Node::End => i == s.len() && k(i),
        Node::Any => i < s.len() && s[i] != '\n' && k(i + 1),
        Node::Char(c) => s.get(i) == Some(c) && k(i + 1),
        Node::Class(ranges, neg) => {
            s.get(i)
                .is_some_and(|c| ranges.iter().any(|(lo, hi)| lo <= c && c <= hi) != *neg)
                && k(i + 1)
        }
        Node::Group(alts) => alts.iter().any(|seq| match_seq(seq, s, i, k)),
        Node::Repeat(atom, min, max) => repeat(atom, *min, *max, s, i, k),
    }
}

/// Greedy repetition with backtracking.
fn repeat(atom: &Node, min: usize, max: usize, s: &[char], i: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
    if max > 0
        && match_node(atom, s, i, &mut |j| {
            j != i && repeat(atom, min.saturating_sub(1), max - 1, s, j, k)
        })
    {
        return true;
    }
    min == 0 && k(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_schema_patterns() {
        let date = Regex::new(r"^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}(:\d{2})?(Z|[+-]\d{2}:?\d{2})?)?$");
        for ok in [
            "2026-05-08",
            "2026-05-08T10:30",
            "2026-05-08T10:30:00Z",
            "2026-05-08T10:30+0100",
        ] {
            assert!(date.is_match(ok), "{ok}");
        }
        for bad in ["2026-5-08", "2026-05-08T", "2026-05-08 10:30", "x2026-05-08"] {
            assert!(!date.is_match(bad), "{bad}");
        }
        let tag = Regex::new("^[a-z0-9][a-z0-9-]*$");
        assert!(tag.is_match("web-dev") && !tag.is_match("-x") && !tag.is_match("Web"));
        let alias = Regex::new("^/.+");
        assert!(alias.is_match("/old/") && !alias.is_match("/") && !alias.is_match("old"));
        assert!(Regex::new("b+").is_match("abbc"), "unanchored patterns search");
    }
}
