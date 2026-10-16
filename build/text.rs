//! Text transforms applied to note bodies before Markdown rendering:
//! slugs, `[[wikilinks]]`, inline `#tags` (both skipping code), relative
//! `.md` links (after rendering), and the
//! small regex-equivalents the build needs (HTML stripping, href scanning).

use std::collections::HashMap;

use crate::unicode;
use crate::util::{percent_decode, py_repr_str, py_strip};

/// `_slugify`: lower, strip, `[^a-z0-9]+` → `-`, strip `-`.
pub fn slugify(stem: &str) -> String {
    let lowered = stem.to_lowercase();
    let s = py_strip(&lowered);
    let mut out = String::with_capacity(s.len());
    let mut in_run = false;
    for c in s.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            in_run = false;
        } else if !in_run {
            out.push('-');
            in_run = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Length (in chars) of a code region starting at `i`, if any:
/// "```" … "```" (lazy, may span lines) or "`" [^`\n]+ "`".
fn code_at(c: &[char], i: usize) -> Option<usize> {
    let tick = |k: usize| c.get(k) == Some(&'`');
    if tick(i) && tick(i + 1) && tick(i + 2) {
        let mut j = i + 3;
        while j + 2 < c.len() {
            if tick(j) && tick(j + 1) && tick(j + 2) {
                return Some(j + 3 - i);
            }
            j += 1;
        }
    }
    if tick(i) {
        let mut j = i + 1;
        while j < c.len() && c[j] != '`' && c[j] != '\n' {
            j += 1;
        }
        if j > i + 1 && tick(j) {
            return Some(j + 1 - i);
        }
    }
    None
}

/// `\[\[(?P<t>[^\]\|]+?)(?:\|(?P<a>[^\]]+))?\]\]` at `i`: returns (len, target, alt).
fn wikilink_at(c: &[char], i: usize) -> Option<(usize, String, Option<String>)> {
    if c.get(i) != Some(&'[') || c.get(i + 1) != Some(&'[') {
        return None;
    }
    let p = i + 2;
    let mut q = p;
    loop {
        // target must be non-empty and free of ']' and '|'
        match c.get(q) {
            Some(&ch) if ch != ']' && ch != '|' => q += 1,
            _ => return None,
        }
        // try with an alt part
        if c.get(q) == Some(&'|') {
            let mut r = q + 1;
            while r < c.len() && c[r] != ']' {
                r += 1;
            }
            if r > q + 1 && c.get(r) == Some(&']') && c.get(r + 1) == Some(&']') {
                return Some((r + 2 - i, c[p..q].iter().collect(), Some(c[q + 1..r].iter().collect())));
            }
        }
        if c.get(q) == Some(&']') && c.get(q + 1) == Some(&']') {
            return Some((q + 2 - i, c[p..q].iter().collect(), None));
        }
    }
}

/// Replace `[[target#anchor|alt]]` with Markdown links, leaving code alone.
pub fn resolve_wikilinks(text: &str, slug_to_url: &HashMap<String, String>, context: &str) -> Result<String, String> {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < c.len() {
        if let Some(n) = code_at(&c, i) {
            out.extend(&c[i..i + n]);
            i += n;
            continue;
        }
        if let Some((n, target, alt)) = wikilink_at(&c, i) {
            let target = py_strip(&target).to_string();
            let alt = alt.map(|a| py_strip(&a).to_string()).unwrap_or_default();
            let no_anchor = target.split('#').next().unwrap_or("");
            let last = no_anchor.rsplit('/').next().unwrap_or("");
            let target_slug = slugify(last);
            let url = match slug_to_url.get(&target_slug) {
                Some(u) if !u.is_empty() => u,
                _ => {
                    return Err(format!(
                        "{}: unresolved wikilink [[{}]] (slug {})",
                        context,
                        target,
                        py_repr_str(&target_slug)
                    ))
                }
            };
            let anchor = match target.split_once('#') {
                Some((_, a)) => format!("#{}", slugify(a)),
                None => String::new(),
            };
            let label = if alt.is_empty() { last.to_string() } else { alt };
            out.push_str(&format!("[{}]({}{})", label, url, anchor));
            i += n;
            continue;
        }
        out.push(c[i]);
        i += 1;
    }
    Ok(out)
}

/// A relative link to a Markdown file (`other.md`, `../about.md#Sec`):
/// no scheme, not root-absolute, not a bare `#fragment`.
fn is_md_link(href: &str) -> bool {
    let path = &href[..href.find(['#', '?']).unwrap_or(href.len())];
    let scheme = path.find(':').is_some_and(|c| !path[..c].contains('/'));
    !scheme && !path.starts_with('/') && path.to_ascii_lowercase().ends_with(".md")
}

/// Rewrite relative `.md` hrefs in rendered `html` to the linked note's URL,
/// resolving each path from the directory of `rel` (as GitHub and Obsidian
/// do; ADR 2026-10-16-relative-markdown-links). A link to a file that isn't a
/// published note is a build error, like an unresolved wikilink.
pub fn resolve_md_links(html: &str, rel: &str, rel_to_url: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(p) = rest.find("href=\"") {
        let start = p + 6;
        let Some(len) = rest[start..].find('"') else { break };
        let href = &rest[start..start + len];
        out.push_str(&rest[..start]);
        rest = &rest[start + len..];
        if !is_md_link(href) {
            out.push_str(href);
            continue;
        }
        let (path, anchor) = href.split_once('#').unwrap_or((href, ""));
        let mut parts: Vec<String> = rel.split('/').map(str::to_string).collect();
        parts.pop();
        for seg in percent_decode(path).replace("&amp;", "&").split('/') {
            match seg {
                "" | "." => {}
                ".." => {
                    if parts.pop().is_none() {
                        return Err(format!("{}: link {} points outside content/", rel, py_repr_str(href)));
                    }
                }
                s => parts.push(s.to_string()),
            }
        }
        let target = parts.join("/");
        let url = rel_to_url.get(&target).ok_or_else(|| {
            format!(
                "{}: broken link {} (no published note at content/{})",
                rel,
                py_repr_str(href),
                target
            )
        })?;
        out.push_str(url);
        let anchor = slugify(&percent_decode(anchor));
        if !anchor.is_empty() {
            out.push('#');
            out.push_str(&anchor);
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// `(?:^|(?<=\s))#(?P<tag>[a-z0-9][a-z0-9-]*)\b` at `i`: returns (len, tag).
fn tag_at(c: &[char], i: usize) -> Option<(usize, String)> {
    if c.get(i) != Some(&'#') {
        return None;
    }
    if !(i == 0 || unicode::is_space(c[i - 1])) {
        return None;
    }
    let tag_char = |ch: char| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
    let start = i + 1;
    match c.get(start) {
        Some(&ch) if ch.is_ascii_lowercase() || ch.is_ascii_digit() => {}
        _ => return None,
    }
    let mut end = start + 1;
    while end < c.len() && tag_char(c[end]) {
        end += 1;
    }
    // backtrack until a word boundary follows
    while end > start {
        let before = unicode::is_word(c[end - 1]);
        let after = c.get(end).is_some_and(|&ch| unicode::is_word(ch));
        if before != after {
            return Some((end - i, c[start..end].iter().collect()));
        }
        end -= 1;
    }
    None
}

/// Turn inline `#tag`s into links and collect them (code is skipped).
pub fn extract_inline_tags(text: &str) -> (String, Vec<String>) {
    let c: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut found = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if let Some(n) = code_at(&c, i) {
            out.extend(&c[i..i + n]);
            i += n;
            continue;
        }
        if let Some((n, tag)) = tag_at(&c, i) {
            out.push_str(&format!("[#{}](/tags/{}/)", tag, tag));
            found.push(tag);
            i += n;
            continue;
        }
        out.push(c[i]);
        i += 1;
    }
    (out, found)
}

/// `re.sub(r"<[^>]+>", "", html)`
pub fn strip_html(html: &str) -> String {
    let c: Vec<char> = html.chars().collect();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while i < c.len() {
        if c[i] == '<' && i + 1 < c.len() && c[i + 1] != '>' {
            if let Some(off) = c[i + 1..].iter().position(|&x| x == '>') {
                i += off + 2;
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// Plain-text excerpt of rendered HTML for `<meta name="description">`:
/// tags stripped, the markdown renderer's entities decoded, whitespace
/// collapsed, and cut at a word boundary to at most `max` characters
/// (an ellipsis marks a cut).
pub fn excerpt(html: &str, max: usize) -> String {
    let text = strip_html(html)
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&");
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = String::new();
    for w in &words {
        let len = out.chars().count() + usize::from(!out.is_empty()) + w.chars().count();
        if len > max.saturating_sub(1) {
            out.push('\u{2026}');
            return out;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(w);
    }
    out
}

/// `re.findall(r'href="([^"#]+)(?:#[^"]*)?"', html)`
pub fn hrefs(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(p) = rest.find("href=\"") {
        let after = &rest[p + 6..];
        let run = after.find(['"', '#']).unwrap_or(after.len());
        let mut consumed = 6;
        if run > 0 && run < after.len() {
            let tail = &after[run..];
            let ok_end = if tail.starts_with('"') {
                Some(run + 1)
            } else {
                tail.find('"').map(|q| run + q + 1)
            };
            if let Some(e) = ok_end {
                out.push(after[..run].to_string());
                consumed = 6 + e;
            }
        }
        rest = &rest[p + consumed..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpts() {
        assert_eq!(
            excerpt("<p>Fish &amp; chips</p>\n<p>are  &quot;good&quot;</p>", 160),
            "Fish & chips are \"good\""
        );
        assert_eq!(excerpt("<p>one two three four</p>", 12), "one two\u{2026}");
        assert_eq!(excerpt("", 160), "");
    }

    #[test]
    fn slugs() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  --a__b-- "), "a-b");
        assert_eq!(slugify("Économie"), "conomie");
    }

    #[test]
    fn md_links() {
        let mut m = HashMap::new();
        m.insert("notes/foo.md".to_string(), "/notes/foo/".to_string());
        m.insert("about.md".to_string(), "/about/".to_string());
        let html = r#"<a href="foo.md#Sec%20One">x</a> <a href="./foo.md">y</a> <a href="../about.md">z</a> <a href="https://x.org/a.md">w</a> <a href="/images/a.png">v</a>"#;
        assert_eq!(
            resolve_md_links(html, "notes/bar.md", &m).unwrap(),
            r#"<a href="/notes/foo/#sec-one">x</a> <a href="/notes/foo/">y</a> <a href="/about/">z</a> <a href="https://x.org/a.md">w</a> <a href="/images/a.png">v</a>"#
        );
        assert_eq!(
            resolve_md_links(r#"<a href="notes/foo.md">a</a>"#, "about.md", &m).unwrap(),
            r#"<a href="/notes/foo/">a</a>"#
        );
        assert!(resolve_md_links(r#"<a href="nope.md">a</a>"#, "notes/bar.md", &m)
            .unwrap_err()
            .contains("broken link 'nope.md' (no published note at content/notes/nope.md)"));
        assert!(resolve_md_links(r#"<a href="../../x.md">a</a>"#, "about.md", &m)
            .unwrap_err()
            .contains("outside content/"));
    }

    #[test]
    fn wikilinks() {
        let mut m = HashMap::new();
        m.insert("foo".to_string(), "/notes/foo/".to_string());
        let r = resolve_wikilinks("a [[Foo#Sec One|x]] `[[nope]]` [[notes/foo]]", &m, "c").unwrap();
        assert_eq!(r, "a [x](/notes/foo/#sec-one) `[[nope]]` [foo](/notes/foo/)");
        assert!(resolve_wikilinks("[[bar]]", &m, "c.md")
            .unwrap_err()
            .contains("unresolved wikilink [[bar]] (slug 'bar')"));
    }

    #[test]
    fn tags() {
        let (t, f) = extract_inline_tags("#a x #b-c\n`#no` d#e #f- #G");
        assert_eq!(t, "[#a](/tags/a/) x [#b-c](/tags/b-c/)\n`#no` d#e [#f](/tags/f/)- #G");
        assert_eq!(f, vec!["a", "b-c", "f"]);
    }

    #[test]
    fn strip_and_hrefs() {
        assert_eq!(strip_html("<p>a <b>x</b> < c <> d</p>"), "a x  d");
        assert_eq!(strip_html("a < b <"), "a < b <");
        assert_eq!(
            hrefs(r##"<a href="/x/#y">a</a><a href="#z"></a><a href="/q/">"##),
            vec!["/x/", "/q/"]
        );
    }
}
