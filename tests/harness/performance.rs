//! Pages follow spec/performance.md: mobile first, readable (never
//! minified) output, and the basics Lighthouse checks.

use std::fs;

use posse_bliki_build::image;
use posse_bliki_build::indent::BLOCK;

use posse_bliki_build::util::escape_html;

use crate::support::{build, content_dir, files_with_ext, html_files, read_as_built, repo, site, temp_dir};

/// Image budget (spec/performance.md): the build ships images as they are,
/// so a heavy one has to be caught before it is committed.
const IMAGE_MAX_BYTES: u64 = 200 * 1024;
const IMAGE_MAX_WIDTH: u32 = 1600;

fn css() -> String {
    fs::read_to_string(site().join("style.css")).expect("read style.css")
}

/// `css` with every `/* ... */` comment removed.
fn without_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    out.push_str(rest);
    out
}

/// Every tag starting `<name` in `html`, up to its `>`.
fn tags<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("<{name} ");
    html.match_indices(&open)
        .map(|(i, _)| &html[i..i + html[i..].find('>').expect("unclosed tag") + 1])
        .collect()
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn description(html: &str) -> Option<&str> {
    let key = "<meta name=\"description\" content=\"";
    let start = html.find(key)? + key.len();
    Some(&html[start..start + html[start..].find('"')?])
}

/// Pages a search engine may index: alias redirects carry `noindex`.
fn indexable() -> impl Iterator<Item = &'static (std::path::PathBuf, String)> {
    html_files()
        .iter()
        .filter(|(_, html)| !html.contains("name=\"robots\" content=\"noindex\""))
}

// --- mobile first ---

#[test]
fn stylesheet_is_mobile_first() {
    // Base rules are the phone layout; width queries only ever add (min-width).
    let css = without_comments(&css());
    let queries: Vec<&str> = css
        .match_indices("@media")
        .map(|(i, _)| &css[i..i + css[i..].find('{').expect("unclosed @media")])
        .filter(|q| q.contains("width"))
        .collect();
    assert!(!queries.is_empty(), "expected min-width queries for wider screens");
    for q in queries {
        assert!(!q.contains("max-width"), "desktop-first query: {q}");
        assert!(q.contains("min-width"), "unexpected width query: {q}");
    }
}

// --- readable, unminified output ---

#[test]
fn stylesheet_ships_as_written() {
    // Copied byte for byte: no minifier, comments kept.
    let source = fs::read_to_string(repo().join("build/static/style.css")).expect("read source");
    let css = css();
    assert_eq!(css, source, "style.css differs from build/static/style.css");
    assert!(css.contains("/*"), "comments were stripped");
}

#[test]
fn stylesheet_is_readable() {
    let css = css();
    let lines: Vec<&str> = css.lines().collect();
    assert!(lines.len() > 100, "stylesheet looks minified");
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    assert!(longest <= 200, "stylesheet line of {longest} characters");
}

#[test]
fn html_is_not_minified() {
    // Each <head> element sits on its own line, as the templates write it.
    for (p, html) in html_files() {
        assert!(
            html.contains("<head>\n  <meta charset=\"utf-8\">\n"),
            "{}: head is not one element per line",
            p.display()
        );
        assert!(html.matches('\n').count() >= 20, "{}: looks minified", p.display());
    }
}

/// Lines outside `<pre>` blocks, which keep their own whitespace.
fn lines_outside_pre(html: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut in_pre = false;
    for line in html.lines() {
        if !in_pre {
            lines.push(line);
        }
        if line.contains("<pre") {
            in_pre = true;
        }
        if line.contains("</pre>") {
            in_pre = false;
        }
    }
    lines
}

#[test]
fn html_is_indented_by_nesting() {
    // Two spaces per open element, <html> adding none (ADR 2026-10-28-indented-html).
    // Every page ending with <main> and </main> at one level proves that
    // whatever the note's text opens, it closes.
    let flush = ["<head>", "</head>", "<body>", "</body>", "</html>"];
    for p in files_with_ext(site(), "html") {
        let html = &read_as_built(&p);
        let lines = lines_outside_pre(html);
        for tag in flush {
            assert!(lines.contains(&tag), "{}: {tag} not flush left", p.display());
        }
        if !html.contains("http-equiv=\"refresh\"") {
            assert!(
                lines.contains(&"  <main id=\"main\">"),
                "{}: <main> not one level in",
                p.display()
            );
            assert!(
                lines.contains(&"  </main>"),
                "{}: </main> not one level in",
                p.display()
            );
        }
        for line in &lines {
            assert!(!line.trim().is_empty(), "{}: blank line", p.display());
            let spaces = line.len() - line.trim_start_matches(' ').len();
            assert!(spaces % 2 == 0, "{}: odd indent on {line:?}", p.display());
        }
    }
}

/// A tag with two or more attributes opens alone on its line, each attribute
/// below it one level deeper (ADR 2026-10-28-one-attribute-per-line).
#[test]
fn each_attribute_on_its_own_line() {
    let mut split = 0;
    for p in files_with_ext(site(), "html") {
        let html = read_as_built(&p);
        let lines = lines_outside_pre(&html);
        for (n, line) in lines.iter().enumerate() {
            for (i, _) in line.match_indices('<') {
                let tag = &line[i + 1..];
                if !tag.starts_with(|c: char| c.is_ascii_alphabetic()) {
                    continue;
                }
                let (words, closed) = tag_words_on_line(tag);
                assert!(words <= 2, "{}:{}: attributes share a line: {line}", p.display(), n + 1);
                if words == 1 && !closed {
                    let indent = |l: &str| l.len() - l.trim_start_matches(' ').len();
                    let next = lines[n + 1];
                    assert_eq!(
                        indent(next),
                        indent(line) + 2,
                        "{}:{}: attribute not one level in: {next}",
                        p.display(),
                        n + 2
                    );
                    split += 1;
                }
            }
        }
    }
    assert!(split > 100, "expected many split tags, found {split}");
}

/// Words (name, then attributes) of the tag starting at `tag` up to its `>`
/// or the end of the line, and whether the `>` is on this line.
fn tag_words_on_line(tag: &str) -> (usize, bool) {
    let mut words = 0;
    let mut in_word = false;
    let mut quote = None;
    for c in tag.chars() {
        match (quote, c) {
            (None, '>') => return (words, true),
            (None, c) if c.is_ascii_whitespace() => in_word = false,
            (None, '/') if !in_word => {}
            (None, _) | (Some(_), _) => {
                if quote.is_none() && matches!(c, '"' | '\'') {
                    quote = Some(c);
                } else if quote == Some(c) {
                    quote = None;
                }
                if !in_word {
                    words += 1;
                    in_word = true;
                }
            }
        }
    }
    (words, false)
}

/// A block's content starts on the line after its start tag and its end tag
/// gets a line of its own (ADR 2026-10-29-block-content-on-its-own-lines).
#[test]
fn block_content_on_its_own_lines() {
    let mut checked = 0;
    for p in files_with_ext(site(), "html") {
        let html = read_as_built(&p);
        for (n, line) in lines_outside_pre(&html).iter().enumerate() {
            let line = line.trim();
            for name in BLOCK {
                let open = format!("<{name}");
                let close = format!("</{name}>");
                if let Some(rest) = line.strip_prefix(open.as_str()) {
                    if rest.starts_with([' ', '>']) && rest.ends_with('>') && !rest.ends_with(close.as_str()) {
                        checked += 1;
                    }
                }
                let empty = format!("<{name}></{name}>");
                if line.ends_with(close.as_str()) && line != close && !line.ends_with(empty.as_str()) {
                    panic!("{}:{}: content before {close} on one line: {line}", p.display(), n + 1);
                }
                if let Some(at) = line.find(open.as_str()) {
                    let after = &line[at + open.len()..];
                    let tag_closed = after.starts_with([' ', '>']) && after.contains('>');
                    let tail = after.split_once('>').map_or("", |(_, t)| t);
                    assert!(
                        !tag_closed || tail.is_empty() || tail.starts_with(close.as_str()),
                        "{}:{}: content after <{name}> on one line: {line}",
                        p.display(),
                        n + 1
                    );
                }
            }
        }
    }
    assert!(checked > 100, "expected many block start tags, found {checked}");
}

/// Indenting never touches a code block: each fenced block in a note ships
/// exactly as written, every line where the author put it.
#[test]
fn code_blocks_keep_their_whitespace() {
    let mut checked = 0;
    for p in files_with_ext(&content_dir(), "md") {
        let md = fs::read_to_string(&p).expect("read note");
        let mut blocks = Vec::new();
        let mut open: Option<String> = None;
        for line in md.lines() {
            match open.as_mut() {
                None if line.starts_with("```") => open = Some(String::new()),
                Some(code) if line.starts_with("```") => {
                    blocks.push(std::mem::take(code));
                    open = None;
                }
                Some(code) => {
                    code.push_str(line);
                    code.push('\n');
                }
                None => {}
            }
        }
        if blocks.is_empty() {
            continue;
        }
        let rel = p.strip_prefix(content_dir()).unwrap().with_extension("");
        let page = html_files()
            .iter()
            .find(|(out, _)| out.parent().is_some_and(|d| d.ends_with(&rel)))
            .unwrap_or_else(|| panic!("{}: no page", p.display()));
        for code in blocks {
            let escaped = escape_html(&code);
            assert!(
                page.1.contains(&format!(">{escaped}</code></pre>")),
                "{}: code block changed:\n{code}",
                p.display()
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "no fenced code blocks in the vault to check");
}

// --- Lighthouse basics ---

#[test]
fn every_indexable_page_has_a_plain_description() {
    for (p, html) in indexable() {
        let d = description(html).unwrap_or_else(|| panic!("{}: no meta description", p.display()));
        assert!(!d.trim().is_empty(), "{}: empty description", p.display());
        assert!(
            !d.contains('<') && !d.contains("&amp;amp;"),
            "{}: markup in description: {d}",
            p.display()
        );
    }
}

#[test]
fn every_page_has_an_icon() {
    // A declared icon stops browsers requesting /favicon.ico and logging a 404.
    for icon in [
        "favicon.ico",
        "icon.svg",
        "apple-touch-icon.png",
        "icon-192.png",
        "icon-512.png",
        "manifest.webmanifest",
    ] {
        assert!(site().join(icon).is_file(), "{icon} not emitted");
    }
    for (p, html) in html_files() {
        assert!(
            html.contains("<link rel=\"icon\" href=\"/favicon.ico\" sizes=\"32x32\">")
                && html.contains("<link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\">")
                && html.contains("<link rel=\"apple-touch-icon\" href=\"/apple-touch-icon.png\">")
                && html.contains("<link rel=\"manifest\" href=\"/manifest.webmanifest\">"),
            "{}",
            p.display()
        );
    }
}

#[test]
fn images_after_the_first_load_lazily() {
    for (p, html) in html_files() {
        for (i, tag) in tags(html, "img").into_iter().enumerate() {
            assert!(tag.contains("decoding=\"async\""), "{}: {tag}", p.display());
            if i == 0 {
                assert!(
                    !tag.contains("loading=\"lazy\""),
                    "{}: keep the first image eager: {tag}",
                    p.display()
                );
            } else {
                assert!(tag.contains("loading=\"lazy\""), "{}: {tag}", p.display());
            }
        }
    }
}

#[test]
fn images_are_self_hosted_with_a_size() {
    // From /images/ with width and height, so nothing shifts as they load.
    for (p, html) in html_files() {
        for tag in tags(html, "img") {
            let src = attr(tag, "src").unwrap_or("");
            assert!(src.starts_with("/images/"), "{}: third-party image {tag}", p.display());
            assert!(site().join(&src[1..]).is_file(), "{}: {src} not emitted", p.display());
            for dim in ["width", "height"] {
                let v = attr(tag, dim).unwrap_or("");
                assert!(
                    !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()),
                    "{}: no {dim} on {tag}",
                    p.display()
                );
            }
        }
    }
}

#[test]
fn embeds_load_lazily() {
    for (p, html) in html_files() {
        for tag in tags(html, "iframe") {
            assert!(tag.contains("loading=\"lazy\""), "{}: {tag}", p.display());
        }
    }
}

#[test]
fn missing_self_hosted_image_is_a_build_error() {
    let content = temp_dir("missing-image");
    fs::create_dir_all(content.join("notes")).unwrap();
    fs::write(content.join("index.md"), "---\ntitle: Home\n---\nHi\n").unwrap();
    fs::write(
        content.join("notes/a.md"),
        "---\ntitle: A\ndate: 2026-01-01\n---\n![x](/images/nope.png)\n",
    )
    .unwrap();
    let err = build(&content, &temp_dir("missing-image-out"), &[]).expect_err("build should fail");
    assert!(err.contains("nope.png"), "unexpected error: {err}");
}

#[test]
fn images_fit_the_budget() {
    let dir = content_dir().join("images");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .expect("read content/images")
        .flatten()
        .map(|e| e.path())
        .collect();
    files.sort();
    assert!(!files.is_empty(), "expected images in content/images/");
    for p in files {
        let bytes = fs::read(&p).expect("read image");
        let name = p.file_name().unwrap().to_string_lossy();
        assert!(
            bytes.len() as u64 <= IMAGE_MAX_BYTES,
            "{name} is {} KB; compress it to {} KB or less",
            bytes.len() / 1024,
            IMAGE_MAX_BYTES / 1024
        );
        let (w, _) = image::size(&bytes).unwrap_or_else(|| panic!("{name}: not a PNG, GIF, JPEG or WebP"));
        assert!(
            w <= IMAGE_MAX_WIDTH,
            "{name} is {w}px wide; resize it to {IMAGE_MAX_WIDTH}px or less"
        );
    }
}
