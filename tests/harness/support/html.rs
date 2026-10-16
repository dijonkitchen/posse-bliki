//! A strict HTML5 checker and tree builder, sized to the HTML the build emits.
//!
//! It replaces html5lib's strict mode (ADR 2026-10-11-rust-only-toolchain). It parses the document into a
//! tree and rejects the parse errors our output could plausibly produce:
//! a missing doctype, unclosed or mismatched elements, end tags with no open
//! element, block elements inside `<p>`, nested `<a>`, duplicate or malformed
//! attributes, unknown or unterminated character references, stray `<`, and
//! control characters. Unlike a browser it never repairs: every non-void
//! element must be closed explicitly, which the build always does.

use posse_bliki_build::entities;

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

impl Element {
    fn new(name: &str) -> Self {
        Element {
            name: name.to_string(),
            attrs: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    pub fn classes(&self) -> Vec<&str> {
        self.attr("class")
            .map(|c| c.split_ascii_whitespace().collect())
            .unwrap_or_default()
    }

    /// Decoded text of this element and its descendants.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for c in &self.children {
            match c {
                Node::Text(t) => out.push_str(t),
                Node::Element(e) => out.push_str(&e.text()),
            }
        }
        out
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|c| match c {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    /// This element and every descendant element, document order.
    pub fn descendants(&self) -> Vec<&Element> {
        let mut out = vec![self];
        for e in self.elements() {
            out.extend(e.descendants());
        }
        out
    }

    pub fn child(&self, name: &str) -> Option<&Element> {
        self.elements().find(|e| e.name == name)
    }
}

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
];

/// Start tags that close an open `<p>` in a browser, which strict mode reports.
const CLOSES_P: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "details",
    "dialog",
    "div",
    "dl",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hgroup",
    "hr",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "ul",
];

/// Elements whose content is text up to their end tag.
const RAW_TEXT: &[&str] = &["script", "style", "title", "textarea"];

/// Decode character references in `s`, rejecting unknown or malformed ones.
pub fn decode(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        rest = &rest[i + 1..];
        let name_len = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '#')
            .unwrap_or(rest.len());
        let name = &rest[..name_len];
        if name.is_empty() {
            out.push('&'); // a bare ampersand is fine
            continue;
        }
        if !rest[name_len..].starts_with(';') {
            return Err(format!("character reference &{name} has no ';'"));
        }
        let decoded = if let Some(num) = name.strip_prefix('#') {
            let cp = match num.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16),
                None => num.parse::<u32>(),
            }
            .map_err(|_| format!("bad numeric reference &{name};"))?;
            char::from_u32(cp)
                .filter(|c| !is_control(*c))
                .ok_or(format!("invalid code point &{name};"))?
                .to_string()
        } else {
            entities::lookup(name)
                .ok_or(format!("unknown entity &{name};"))?
                .to_string()
        };
        out.push_str(&decoded);
        rest = &rest[name_len + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn is_control(c: char) -> bool {
    matches!(c as u32, 0..=0x08 | 0x0b | 0x0e..=0x1f | 0x7f..=0x9f)
}

/// Parse `html` strictly into a tree rooted at a synthetic `#document` element.
pub fn parse(html: &str) -> Result<Element, String> {
    if let Some((i, c)) = html.char_indices().find(|(_, c)| is_control(*c)) {
        return Err(format!("control character U+{:04X} at byte {i}", c as u32));
    }
    let rest = html.strip_prefix('\u{feff}').unwrap_or(html);
    if !rest
        .get(..15)
        .is_some_and(|d| d.eq_ignore_ascii_case("<!DOCTYPE html>"))
    {
        return Err("document does not start with <!DOCTYPE html>".into());
    }
    let mut p = Parser {
        src: rest,
        pos: 15,
        stack: vec![Element::new("#document")],
    };
    p.run()?;
    if p.stack.len() > 1 {
        let open: Vec<&str> = p.stack[1..].iter().map(|e| e.name.as_str()).collect();
        return Err(format!("unclosed elements at end of document: {}", open.join(" > ")));
    }
    Ok(p.stack.pop().unwrap())
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    stack: Vec<Element>,
}

impl<'a> Parser<'a> {
    fn err<T>(&self, msg: impl std::fmt::Display) -> Result<T, String> {
        let line = self.src[..self.pos].matches('\n').count() + 1;
        Err(format!("line {line}: {msg}"))
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn push_text(&mut self, raw: &str) -> Result<(), String> {
        if raw.is_empty() {
            return Ok(());
        }
        let text = decode(raw).or_else(|e| self.err(e))?;
        self.stack.last_mut().unwrap().children.push(Node::Text(text));
        Ok(())
    }

    fn open(&self, name: &str) -> bool {
        self.stack.iter().any(|e| e.name == name)
    }

    fn run(&mut self) -> Result<(), String> {
        while self.pos < self.src.len() {
            let rest = self.rest();
            let lt = rest.find('<').unwrap_or(rest.len());
            let text = &self.src[self.pos..self.pos + lt];
            self.push_text(text)?;
            self.pos += lt;
            if self.pos >= self.src.len() {
                break;
            }
            let rest = self.rest();
            if let Some(after) = rest.strip_prefix("<!--") {
                let end = after.find("-->").map_or_else(|| self.err("unterminated comment"), Ok)?;
                self.pos += 4 + end + 3;
            } else if rest.starts_with("</") {
                self.end_tag()?;
            } else if rest[1..].starts_with(|c: char| c.is_ascii_alphabetic()) {
                self.start_tag()?;
            } else {
                return self.err("'<' that does not start a tag (escape it as &lt;)");
            }
        }
        Ok(())
    }

    fn tag_name(&mut self) -> String {
        let rest = self.rest();
        let len = rest
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
            .unwrap_or(rest.len());
        let name = rest[..len].to_ascii_lowercase();
        self.pos += len;
        name
    }

    fn skip_ws(&mut self) -> bool {
        let rest = self.rest();
        let trimmed = rest.trim_start_matches([' ', '\t', '\n', '\r', '\u{c}']);
        let skipped = rest.len() - trimmed.len();
        self.pos += skipped;
        skipped > 0
    }

    fn start_tag(&mut self) -> Result<(), String> {
        self.pos += 1;
        let mut el = Element::new(&self.tag_name());
        let mut self_closing = false;
        loop {
            let had_ws = self.skip_ws();
            let rest = self.rest();
            if rest.starts_with('>') {
                self.pos += 1;
                break;
            }
            if rest.starts_with("/>") {
                self.pos += 2;
                self_closing = true;
                break;
            }
            if rest.is_empty() {
                return self.err(format!("unterminated <{}> tag", el.name));
            }
            if !had_ws {
                return self.err(format!("<{}>: attributes must be separated by whitespace", el.name));
            }
            let len = rest
                .find(|c: char| c.is_whitespace() || "=>/\"'<".contains(c))
                .unwrap_or(rest.len());
            if len == 0 {
                return self.err(format!("<{}>: malformed attribute", el.name));
            }
            let name = rest[..len].to_ascii_lowercase();
            self.pos += len;
            let mut value = String::new();
            if self.rest().starts_with('=') {
                self.pos += 1;
                let rest = self.rest();
                let raw = if let Some(q) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') {
                    let end = rest[1..]
                        .find(q)
                        .map_or_else(|| self.err("unterminated attribute value"), Ok)?;
                    self.pos += end + 2;
                    &rest[1..end + 1]
                } else {
                    let end = rest.find(|c: char| c.is_whitespace() || c == '>').unwrap_or(rest.len());
                    if end == 0 || rest[..end].contains(['"', '\'', '<', '=', '`']) {
                        return self.err(format!("<{}>: bad unquoted value for {name}", el.name));
                    }
                    self.pos += end;
                    &rest[..end]
                };
                value = decode(raw).or_else(|e| self.err(e))?;
            }
            if el.attr(&name).is_some() {
                return self.err(format!("<{}>: duplicate attribute {name}", el.name));
            }
            el.attrs.push((name, value));
        }
        let void = VOID.contains(&el.name.as_str());
        if self_closing && !void {
            return self.err(format!("<{} /> is not a void element", el.name));
        }
        if CLOSES_P.contains(&el.name.as_str()) && self.open("p") {
            return self.err(format!("<{}> inside an open <p>", el.name));
        }
        if el.name == "a" && self.open("a") {
            return self.err("<a> inside another <a>");
        }
        if void {
            self.stack.last_mut().unwrap().children.push(Node::Element(el));
        } else if RAW_TEXT.contains(&el.name.as_str()) {
            let close = format!("</{}>", el.name);
            let rest = self.rest();
            let end = rest
                .find(&close)
                .map_or_else(|| self.err(format!("unclosed <{}>", el.name)), Ok)?;
            let text = rest[..end].to_string();
            self.pos += end + close.len();
            let decoded = if el.name == "title" || el.name == "textarea" {
                decode(&text).or_else(|e| self.err(e))?
            } else {
                text
            };
            el.children.push(Node::Text(decoded));
            self.stack.last_mut().unwrap().children.push(Node::Element(el));
        } else {
            self.stack.push(el);
        }
        Ok(())
    }

    fn end_tag(&mut self) -> Result<(), String> {
        self.pos += 2;
        let name = self.tag_name();
        self.skip_ws();
        if !self.rest().starts_with('>') {
            return self.err(format!("malformed </{name}> tag"));
        }
        self.pos += 1;
        if VOID.contains(&name.as_str()) {
            return self.err(format!("end tag for void element </{name}>"));
        }
        let top = &self.stack.last().unwrap().name;
        if *top != name {
            return self.err(format!("</{name}> does not close the open <{top}>"));
        }
        let el = self.stack.pop().unwrap();
        self.stack.last_mut().unwrap().children.push(Node::Element(el));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "<!DOCTYPE html>\n<html lang=\"en\"><head><title>t &amp; u</title></head><body>";

    fn doc(body: &str) -> Result<Element, String> {
        parse(&format!("{HEAD}{body}</body></html>"))
    }

    #[test]
    fn accepts_well_formed_output() {
        let d = doc("<p>a <a href=\"/x?a=1&amp;b=2\">b</a><br>c &#8212; &mdash;</p><ul><li>x</li></ul>").unwrap();
        assert!(d
            .descendants()
            .iter()
            .any(|e| e.name == "a" && e.attr("href") == Some("/x?a=1&b=2")));
    }

    #[test]
    fn rejects_parse_errors() {
        for bad in [
            "<p>unclosed",
            "<p>a</div>",
            "</p>",
            "<p><ul><li>x</li></ul></p>",
            "<a href=\"/\"><a href=\"/\">x</a></a>",
            "<p class=\"a\" class=\"b\">x</p>",
            "<p>&nosuch;</p>",
            "<p>&amp</p>",
            "<p>1 < 2</p>",
            "<div/>",
            "<br></br>",
            "<p id=\"x\"title=\"y\">x</p>",
        ] {
            assert!(doc(bad).is_err(), "accepted {bad:?}");
        }
        assert!(parse("<html></html>").is_err(), "accepted a missing doctype");
    }
}
