//! Indent each built page by its nesting so "view source" reads like the
//! source it is (docs/adr/2026-10-28-indented-html.md).
//!
//! Only whitespace that follows a newline changes, and outside `<pre>`,
//! `<textarea>`, `<script>` and `<style>` the browser already collapses
//! that whitespace to the newline's single space, so pages render as before.

/// Elements with no end tag.
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
];

/// Elements whose contents ship byte for byte: their whitespace is content.
const RAW: &[&str] = &["pre", "textarea", "script", "style"];

const STEP: &str = "  ";

/// `html` with blank lines dropped and every line indented two spaces per
/// open element. `<html>` adds no level, so `<head>` and `<body>` stay flush
/// left; a line that opens with an end tag lines up with its start tag.
pub fn indent(html: &str) -> String {
    let mut out = String::with_capacity(html.len() + html.len() / 4);
    let mut depth = 0usize;
    let mut rest = html;
    let mut line_start = true;
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
        out.push_str(tag);
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
        if RAW.contains(&name.as_str()) {
            let close = format!("</{name}");
            let j = find_ascii_ci(rest, &close).unwrap_or(rest.len());
            out.push_str(&rest[..j]);
            rest = &rest[j..];
        }
    }
    out
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
        let want = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n</head>\n<body>\n  <main>\n    <ul>\n      <li><a href=\"/\">x</a></li>\n    </ul>\n  </main>\n</body>\n</html>\n";
        assert_eq!(indent(html), want);
    }

    #[test]
    fn drops_blank_lines() {
        assert_eq!(indent("<ul>\n\n<li>a</li>\n  \n</ul>\n"), "<ul>\n  <li>a</li>\n</ul>\n");
    }

    #[test]
    fn leaves_preformatted_text_alone() {
        let html = "<div>\n<pre><code>def f\n\n  x\nend\n</code></pre>\n<p>after</p>\n</div>\n";
        let want = "<div>\n  <pre><code>def f\n\n  x\nend\n</code></pre>\n  <p>after</p>\n</div>\n";
        assert_eq!(indent(html), want);
        let html = "<div>\n<script>\nif (a < b) {}\n</script>\n</div>\n";
        assert_eq!(indent(html), "<div>\n  <script>\nif (a < b) {}\n</script>\n</div>\n");
    }

    #[test]
    fn multi_line_paragraphs_indent_as_one_level() {
        let html = "<p>one\n<em>two</em>\nthree</p>\n<p>next</p>\n";
        assert_eq!(indent(html), "<p>one\n  <em>two</em>\n  three</p>\n<p>next</p>\n");
    }

    #[test]
    fn quoted_angle_brackets_stay_in_the_tag() {
        let html = "<div title=\"a > b\">\n<p>x</p>\n</div>\n";
        assert_eq!(indent(html), "<div title=\"a > b\">\n  <p>x</p>\n</div>\n");
    }

    #[test]
    fn is_idempotent() {
        let html = "<html>\n<body>\n<div>\n<p>a\nb</p>\n</div>\n</body>\n</html>\n";
        assert_eq!(indent(&indent(html)), indent(html));
    }
}
