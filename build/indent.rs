//! Indent each built page by its nesting so "view source" reads like the
//! source it is (docs/adr/2026-10-28-indented-html.md).
//!
//! Only whitespace that follows a newline changes, and outside `<pre>`,
//! `<textarea>`, `<script>` and `<style>` the browser already collapses
//! that whitespace to the newline's single space, so pages render as before.
//! A tag with two or more attributes puts each on its own line, one level
//! deeper than the tag (docs/adr/2026-10-28-one-attribute-per-line.md):
//! whitespace inside a tag is never content.

/// Elements with no end tag.
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
];

/// Elements whose contents ship byte for byte: their whitespace is content.
const RAW: &[&str] = &["pre", "textarea", "script", "style"];

const STEP: &str = "  ";

/// Block elements, whose content goes on its own lines between the start and
/// end tags (docs/adr/2026-10-29-block-content-on-its-own-lines.md). A block
/// drops whitespace at the start and end of its lines, so this renders the
/// same; inline elements (`<a>`, `<span>`, `<time>`) keep their text against
/// the tag, since whitespace there would show.
pub const BLOCK: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "dd",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "header",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "section",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

/// `html` with blank lines dropped and every line indented two spaces per
/// open element. `<html>` adds no level, so `<head>` and `<body>` stay flush
/// left; a line that opens with an end tag lines up with its start tag.
pub fn indent(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + html.len() / 4);
    let mut depth = 0usize;
    let mut rest = html;
    let mut line_start = true;
    // Set right after an empty block's start tag, so its end tag stays put.
    let mut empty_block = false;
    while !rest.is_empty() {
        if line_start {
            line_start = false;
            let line = rest.trim_start_matches([' ', '\t']);
            if let Some(after) = line.strip_prefix('\n') {
                rest = after;
                line_start = true;
                continue;
            }
            rest = line;
            let level = if line.starts_with("</") {
                depth.saturating_sub(1)
            } else {
                depth
            };
            for _ in 0..level {
                out.push_str(STEP);
            }
        }
        let Some(i) = rest.find(['<', '\n']) else {
            out.push_str(rest);
            break;
        };
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if let Some(after) = rest.strip_prefix('\n') {
            out.push('\n');
            rest = after;
            line_start = true;
            continue;
        }
        // A comment or doctype: copied whole, no level.
        if rest.starts_with("<!") {
            let end = if rest.starts_with("<!--") {
                rest.find("-->").map_or(rest.len(), |j| j + 3)
            } else {
                rest.find('>').map_or(rest.len(), |j| j + 1)
            };
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        let closing = rest.starts_with("</");
        let name_start = if closing { 2 } else { 1 };
        let name_len = rest[name_start..]
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-')
            .unwrap_or(rest.len() - name_start);
        if name_len == 0 {
            // A bare `<` in text.
            out.push('<');
            rest = &rest[1..];
            continue;
        }
        let name = rest[name_start..name_start + name_len].to_ascii_lowercase();
        let end = tag_end(rest);
        let tag = &rest[..end];
        let block = BLOCK.contains(&name.as_str());
        let was_empty = std::mem::take(&mut empty_block);
        if closing && block && !was_empty && !current_line(&out).trim().is_empty() {
            // Content before a block's end tag: the end tag gets its own line.
            out.truncate(out.trim_end_matches([' ', '\t']).len());
            out.push('\n');
            for _ in 0..depth.saturating_sub(1) {
                out.push_str(STEP);
            }
        }
        push_tag(&mut out, tag);
        rest = &rest[end..];
        if name == "html" || VOID.contains(&name.as_str()) {
            continue;
        }
        if closing {
            depth = depth.saturating_sub(1);
            continue;
        }
        if tag.ends_with("/>") {
            continue;
        }
        depth += 1;
        if block {
            let next = rest.trim_start_matches([' ', '\t']);
            let empty = next.len() > name.len() + 2
                && next.starts_with("</")
                && next[2..2 + name.len()].eq_ignore_ascii_case(&name);
            empty_block = empty;
            if !next.starts_with('\n') && !next.is_empty() && !empty {
                // Content after a block's start tag: it starts a new line.
                out.push('\n');
                line_start = true;
            }
            continue;
        }
        if RAW.contains(&name.as_str()) {
            let close = format!("</{name}");
            let j = find_ascii_ci(rest, &close).unwrap_or(rest.len());
            out.push_str(&rest[..j]);
            rest = &rest[j..];
        }
    }
    out
}

/// `tag` as written if it has at most one attribute; otherwise the name,
/// then each attribute on its own line one level deeper than the line the
/// tag starts on, the `>` closing the last.
fn push_tag(out: &mut String, tag: &str) {
    if !tag.ends_with('>') {
        out.push_str(tag);
        return;
    }
    let words = tag_words(tag);
    let attrs = words.iter().filter(|w| **w != "/").count().saturating_sub(1);
    if attrs < 2 {
        out.push_str(tag);
        return;
    }
    let line = current_line(out);
    let indent = format!("{}{STEP}", &line[..line.len() - line.trim_start_matches(' ').len()]);
    let mut words = words.into_iter();
    out.push_str(words.next().unwrap_or_default());
    for w in words {
        if w == "/" {
            out.push_str(" /");
            continue;
        }
        out.push('\n');
        out.push_str(&indent);
        out.push_str(w);
    }
    out.push('>');
}

/// The words of a tag without its `<` `>`: the name, then each attribute
/// (quoted values kept whole) and a self-closing `/`, if any.
fn tag_words(tag: &str) -> Vec<&str> {
    let body = &tag[..tag.len() - 1];
    let mut words = Vec::new();
    let mut quote = None;
    let mut start = None;
    for (i, c) in body.char_indices() {
        match (quote, c) {
            (None, c) if c.is_ascii_whitespace() => {
                if let Some(s) = start.take() {
                    words.push(&body[s..i]);
                }
            }
            (None, '"' | '\'') => {
                quote = Some(c);
                start.get_or_insert(i);
            }
            (Some(q), _) if c == q => quote = None,
            _ => {
                start.get_or_insert(i);
            }
        }
    }
    if let Some(s) = start {
        words.push(&body[s..]);
    }
    words
}

/// The last, unfinished line of `out`.
fn current_line(out: &str) -> &str {
    &out[out.rfind('\n').map_or(0, |i| i + 1)..]
}

/// Length of the tag at the start of `s`, through its `>`, skipping any `>`
/// inside a quoted attribute value.
fn tag_end(s: &str) -> usize {
    let mut quote = None;
    for (i, c) in s.char_indices().skip(1) {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, '>') => return i + 1,
            _ => {}
        }
    }
    s.len()
}

fn find_ascii_ci(haystack: &str, needle: &str) -> Option<usize> {
    let (h, n) = (haystack.as_bytes(), needle.as_bytes());
    (0..=h.len().checked_sub(n.len())?).find(|&i| h[i..i + n.len()].eq_ignore_ascii_case(n))
}

#[cfg(test)]
mod tests {
    use super::indent;

    #[test]
    fn indents_by_nesting() {
        let html = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n</head>\n<body>\n<main>\n<ul>\n<li><a href=\"/\">x</a></li>\n</ul>\n</main>\n</body>\n</html>\n";
        let want = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n</head>\n<body>\n  <main>\n    <ul>\n      <li>\n        <a href=\"/\">x</a>\n      </li>\n    </ul>\n  </main>\n</body>\n</html>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn drops_blank_lines() {
        assert_eq!(
            indent("<ul>\n\n<li>a</li>\n  \n</ul>\n"),
            "<ul>\n  <li>\n    a\n  </li>\n</ul>\n"
        );
    }

    #[test]
    fn leaves_preformatted_text_alone() {
        let html = "<div>\n<pre><code>def f\n\n  x\nend\n</code></pre>\n<p>after</p>\n</div>\n";
        let want = "<div>\n  <pre><code>def f\n\n  x\nend\n</code></pre>\n  <p>\n    after\n  </p>\n</div>\n";
        assert_eq!(indent(html), want);
        let html = "<div>\n<script>\nif (a < b) {}\n</script>\n</div>\n";
        assert_eq!(indent(html), "<div>\n  <script>\nif (a < b) {}\n</script>\n</div>\n");
    }

    #[test]
    fn multi_line_paragraphs_indent_as_one_level() {
        let html = "<p>one\n<em>two</em>\nthree</p>\n<p>next</p>\n";
        assert_eq!(
            indent(html),
            "<p>\n  one\n  <em>two</em>\n  three\n</p>\n<p>\n  next\n</p>\n"
        );
    }

    #[test]
    fn quoted_angle_brackets_stay_in_the_tag() {
        let html = "<div title=\"a > b\">\n<p>x</p>\n</div>\n";
        assert_eq!(indent(html), "<div title=\"a > b\">\n  <p>\n    x\n  </p>\n</div>\n");
    }

    #[test]
    fn attributes_get_a_line_each() {
        let html = "<ul>\n<li class=\"h-entry\"><a class=\"u-url p-name\" href=\"/w/\">wonder</a></li>\n</ul>\n";
        let want =
            "<ul>\n  <li class=\"h-entry\">\n    <a\n      class=\"u-url p-name\"\n      href=\"/w/\">wonder</a>\n  </li>\n</ul>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn one_attribute_stays_on_the_line() {
        let html = "<nav aria-label=\"Site\">\n<a href=\"/\">Home</a>\n<span hidden>x</span>\n</nav>\n";
        let want = "<nav aria-label=\"Site\">\n  <a href=\"/\">Home</a>\n  <span hidden>x</span>\n</nav>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn split_tags_keep_quotes_booleans_and_slashes() {
        let html =
            "<div>\n<img src=\"/a b.png\" alt=\"x > y\" />\n<option value=\"auto\" selected>Auto</option>\n</div>\n";
        let want = "<div>\n  <img\n    src=\"/a b.png\"\n    alt=\"x > y\" />\n  <option\n    value=\"auto\"\n    selected>Auto</option>\n</div>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn a_tag_opening_on_an_attribute_line_nests_under_it() {
        let html = "<p>\n<a class=\"u\" href=\"/\"><data class=\"n\" value=\"J\"></data></a>\n</p>\n";
        let want = "<p>\n  <a\n    class=\"u\"\n    href=\"/\"><data\n      class=\"n\"\n      value=\"J\"></data></a>\n</p>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn block_content_gets_its_own_lines() {
        let html = "<ul>\n<li>one <a href=\"/\">two</a></li>\n<li></li>\n</ul>\n<h1>Title</h1>\n";
        let want = "<ul>\n  <li>\n    one <a href=\"/\">two</a>\n  </li>\n  <li></li>\n</ul>\n<h1>Title</h1>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn is_idempotent() {
        let html = "<html>\n<body>\n<div class=\"a\" id=\"b\">\n<p>a\nb <a class=\"u\" href=\"/\">c</a></p>\n</div>\n</body>\n</html>\n";
        assert_eq!(indent(&indent(html)), indent(html));
    }
}
