//! Every internal href resolves to an emitted file (spec/output-contract.md),
//! and links between notes are relative `.md` paths that work on GitHub, in
//! Obsidian and on the site (ADR 2026-10-16-relative-markdown-links).

use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::support::{
    build, content_dir, files_with_ext, hrefs, html_files, read, rel, site, strip_fragment, temp_dir, vault_copy,
};

/// The file a static host would serve for URL path `href`.
fn resolves(site: &Path, href: &str) -> bool {
    let path = href.trim_start_matches('/');
    if path.is_empty() {
        return site.join("index.html").is_file();
    }
    site.join(path).is_file()
        || site.join(path).join("index.html").is_file()
        || (!path.ends_with('/') && site.join(format!("{path}.html")).is_file())
}

/// `href` has a scheme (`https:`, `mailto:`) or is protocol-relative.
fn external(href: &str) -> bool {
    href.starts_with("//") || href.find(':').is_some_and(|c| !href[..c].contains('/'))
}

/// A relative `href` on page `page` as a root-absolute URL path, or `None`
/// when it climbs above the site root.
fn absolute(root: &Path, page: &Path, href: &str) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    for c in page.parent()?.strip_prefix(root).ok()?.join(href).components() {
        match c {
            Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts.pop()?;
            }
            _ => {}
        }
    }
    let trailing = if href.ends_with('/') { "/" } else { "" };
    Some(format!("/{}{trailing}", parts.join("/")))
}

#[test]
fn no_broken_internal_links() {
    let root = site();
    let mut failures = Vec::new();
    for (p, text) in html_files() {
        for href in hrefs(text).into_iter().map(strip_fragment) {
            if href.is_empty() || external(href) {
                continue;
            }
            let ok = if href.starts_with('/') {
                resolves(root, href)
            } else {
                absolute(root, p, href).is_some_and(|a| resolves(root, &a))
            };
            if !ok {
                failures.push(format!("{} -> {href}", rel(p, root)));
            }
        }
    }
    assert!(failures.is_empty(), "broken internal links:\n{}", failures.join("\n"));
}

/// `md` with fenced and inline code removed.
fn prose(md: &str) -> String {
    let mut out = String::new();
    for block in md.split("```").step_by(2) {
        for span in block.split('`').step_by(2) {
            out.push_str(span);
        }
    }
    out
}

/// Raw HTML tags in `text`, like `<iframe` or `</div`. Autolinks
/// (`<https://...>`) aren't tags.
fn html_tags(text: &str) -> Vec<&str> {
    text.match_indices('<')
        .filter_map(|(i, _)| {
            let rest = text[i + 1..].strip_prefix('/').unwrap_or(&text[i + 1..]);
            let end = rest.find(|c: char| !c.is_ascii_alphanumeric()).unwrap_or(rest.len());
            let name = &rest[..end];
            let next = rest[end..].chars().next();
            (name.starts_with(|c: char| c.is_ascii_alphabetic()) && matches!(next, Some(' ' | '>' | '/' | '\n')))
                .then_some(name)
        })
        .collect()
}

/// Inline Markdown link targets in `text`: the `x` of `](x)`.
fn link_targets(text: &str) -> Vec<&str> {
    text.match_indices("](")
        .filter_map(|(k, _)| {
            let rest = &text[k + 2..];
            rest.find(')')
                .map(|end| rest[..end].split_whitespace().next().unwrap_or(""))
        })
        .collect()
}

#[test]
fn content_links_are_relative_markdown_paths() {
    let content = content_dir();
    let mut failures = Vec::new();
    for p in files_with_ext(&content, "md") {
        let md = prose(&read(&p));
        let name = rel(&p, &content);
        if md.contains("[[") {
            failures.push(format!("{name}: uses [[wikilinks]]; write [text](note.md)"));
        }
        // The build escapes raw HTML, so a tag shows up as literal text on
        // the page (ADR 2026-10-28-videos-as-links).
        for tag in html_tags(&md) {
            failures.push(format!("{name}: raw HTML <{tag}> shows as text; write Markdown"));
        }
        for t in link_targets(&md) {
            // Images stay at /images/ (served from content/images/).
            if t.starts_with('/') && !t.starts_with("//") && !t.starts_with("/images/") {
                failures.push(format!(
                    "{name}: {t} is a site URL; link the file, e.g. [text](note.md)"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "links that don't work on GitHub or in Obsidian:\n{}",
        failures.join("\n")
    );
}

fn vault_with(label: &str, body: &str) -> (PathBuf, PathBuf) {
    let vault = vault_copy(label);
    fs::write(
        vault.join("notes/link-test.md"),
        format!("---\ntitle: Link test\ndate: 2026-10-16\n---\n\n{body}\n"),
    )
    .unwrap();
    (vault, temp_dir(&format!("{label}-out")))
}

#[test]
fn markdown_links_resolve_to_note_urls() {
    let (vault, out) = vault_with(
        "md-links",
        "[a](bliki.md), [b](../about.md#Who%20I%20am), [c](./bliki.md)",
    );
    build(&vault, &out, &[]).expect("build with .md links");
    let html = read(&out.join("notes/link-test/index.html"));
    assert!(html.contains(r#"href="/notes/bliki/""#), "bliki.md not rewritten");
    assert!(
        html.contains(r#"href="/about/#who-i-am""#),
        "../about.md#… not rewritten"
    );
    assert!(!html.contains(".md\""), "an .md href reached the output");
    let bliki = read(&out.join("notes/bliki/index.html"));
    assert!(
        bliki.contains("/notes/link-test/"),
        "an .md link doesn't count as a backlink"
    );
}

#[test]
fn broken_markdown_link_fails_the_build() {
    let (vault, out) = vault_with("md-broken", "[gone](no-such-note.md)");
    let err = build(&vault, &out, &[]).expect_err("build should fail");
    assert!(err.contains("broken link 'no-such-note.md'"), "unexpected error: {err}");
}
