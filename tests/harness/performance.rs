//! Pages follow spec/performance.md: mobile first, readable (never
//! minified) output, and the basics Lighthouse checks.

use std::fs;

use posse_bliki_build::image;

use crate::support::{build, content_dir, html_files, repo, site, temp_dir};

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
            html.contains("<head>\n<meta charset=\"utf-8\">\n"),
            "{}: head is not one element per line",
            p.display()
        );
        assert!(html.matches('\n').count() >= 20, "{}: looks minified", p.display());
    }
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
    for icon in ["favicon.ico", "favicon-48x48.png", "apple-touch-icon.png"] {
        assert!(site().join(icon).is_file(), "{icon} not emitted");
    }
    for (p, html) in html_files() {
        assert!(
            html.contains("<link rel=\"icon\" href=\"/favicon-48x48.png\" type=\"image/png\" sizes=\"48x48\">")
                && html.contains("<link rel=\"apple-touch-icon\" href=\"/apple-touch-icon.png\">"),
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
