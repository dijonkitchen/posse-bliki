//! Pages follow spec/design.md: one self-hosted stylesheet built on the a11y
//! design system. Runs under `cargo test`; std only (ADR 0007).
//!
//! Builds the real vault once with the `bliki` binary into a temp dir, then
//! checks the emitted HTML and `style.css`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const SEMANTIC_TOKENS: &[&str] = &[
    "--text-1",
    "--text-2",
    "--text-link",
    "--surface-page",
    "--surface-sunken",
    "--border-strong",
    "--border-subtle",
    "--focus-halo",
    "--focus-ring",
];

const DARK_QUERY: &str = "@media (prefers-color-scheme: dark) {";

/// The built site, shared by every test in this file.
fn site() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let out = std::env::temp_dir().join(format!("bliki-design-{}", std::process::id()));
        let _ = fs::remove_dir_all(&out);
        let status = Command::new(env!("CARGO_BIN_EXE_bliki"))
            .current_dir(root)
            .arg("--content")
            .arg(root.join("content"))
            .arg("--out")
            .arg(&out)
            .output()
            .expect("run bliki");
        assert!(status.status.success(), "build failed: {}", String::from_utf8_lossy(&status.stderr));
        out
    })
}

fn html_files() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let p = entry.expect("dir entry").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|e| e == "html") {
                out.push(p);
            }
        }
    }
    let mut paths = Vec::new();
    walk(site(), &mut paths);
    paths.sort();
    assert!(paths.len() >= 4, "expected several HTML files, got {}", paths.len());
    paths
        .into_iter()
        .map(|p| {
            let text = fs::read_to_string(&p).expect("read html");
            (p, text)
        })
        .collect()
}

fn css() -> String {
    fs::read_to_string(site().join("style.css")).expect("read style.css")
}

/// The argument of every `url(...)` in `css`, quotes stripped.
fn css_urls(css: &str) -> Vec<String> {
    css.match_indices("url(")
        .map(|(i, _)| {
            let rest = &css[i + 4..];
            let inner = &rest[..rest.find(')').expect("unclosed url(")];
            inner.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
        })
        .collect()
}

/// Every `<link ...>` tag in `html`.
fn link_tags(html: &str) -> Vec<&str> {
    html.match_indices("<link ")
        .map(|(i, _)| &html[i..i + html[i..].find('>').expect("unclosed <link") + 1])
        .collect()
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

/// The rule body following `selector {` in `css`, if any.
fn rule_after<'a>(css: &'a str, from: usize, selector: &str) -> Option<&'a str> {
    let start = from + css[from..].find(&format!("{selector} {{"))? + selector.len() + 2;
    Some(&css[start..start + css[start..].find('}')?])
}

/// Split the stylesheet at the dark-mode block: (everything before, the block's body).
fn light_and_dark(css: &str) -> (&str, &str) {
    let start = css.find(DARK_QUERY).expect("no prefers-color-scheme: dark block");
    let body = &css[start + DARK_QUERY.len()..];
    let end = body.find("\n}").expect("unclosed dark block");
    (&css[..start], &body[..end])
}

#[test]
fn every_page_links_the_stylesheet() {
    for (p, html) in html_files() {
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/style.css\">"), "{}", p.display());
    }
}

#[test]
fn no_inline_styles() {
    for (p, html) in html_files() {
        let lower = html.to_lowercase();
        assert!(!lower.contains("<style"), "{}: <style> block", p.display());
        assert!(!lower.contains(" style="), "{}: inline style attribute", p.display());
    }
}

#[test]
fn no_third_party_requests_from_pages() {
    // rel=me, alternate and webmention links point elsewhere by design; only
    // links that make the browser fetch something must stay on this origin.
    for (p, html) in html_files() {
        for tag in link_tags(&html) {
            let rel = attr(tag, "rel").unwrap_or("");
            if !matches!(rel, "stylesheet" | "preload" | "icon" | "modulepreload") {
                continue;
            }
            let href = attr(tag, "href").unwrap_or("");
            assert!(href.starts_with('/') && !href.starts_with("//"), "{}: {}", p.display(), tag);
        }
    }
}

#[test]
fn stylesheet_is_self_contained() {
    let css = css();
    assert!(!css.contains("@import"), "style.css uses @import");
    for url in css_urls(&css) {
        assert!(
            !url.contains("//") && !url.starts_with('/') && !url.starts_with("data:"),
            "non-relative url({url})"
        );
    }
}

#[test]
fn stylesheet_urls_resolve() {
    let urls = css_urls(&css());
    assert!(!urls.is_empty(), "expected self-hosted font urls");
    for url in urls {
        assert!(site().join(&url).is_file(), "url({url}) does not resolve next to style.css");
    }
}

#[test]
fn fonts_ship_with_licences() {
    let fonts = site().join("fonts");
    let mut woff2 = 0;
    for entry in fs::read_dir(&fonts).expect("no fonts/ emitted") {
        let p = entry.expect("dir entry").path();
        if p.extension().is_some_and(|e| e == "woff2") {
            woff2 += 1;
            let bytes = fs::read(&p).expect("read font");
            assert!(bytes.starts_with(b"wOF2"), "{} is not WOFF2", p.display());
        }
    }
    assert!(woff2 > 0, "no fonts emitted");
    assert!(fonts.join("OFL-atkinson-hyperlegible-next.txt").is_file());
    assert!(fonts.join("OFL-jetbrains-mono.txt").is_file());
}

#[test]
fn type_uses_the_design_system_fonts() {
    let css = css();
    assert!(css.contains("font-family: \"Atkinson Hyperlegible Next\""));
    assert!(css.contains("font-family: \"JetBrains Mono\""));
    assert!(css.contains("--font-sans:") && css.contains("--font-code:"));
}

#[test]
fn semantic_tokens_light_and_dark() {
    let css = css();
    let (light, dark) = light_and_dark(&css);
    for token in SEMANTIC_TOKENS {
        assert!(light.contains(&format!("{token}:")), "{token} missing in light");
        assert!(dark.contains(&format!("{token}:")), "{token} missing in dark");
    }
}

#[test]
fn user_preferences_honoured() {
    let css = css();
    for query in [
        "prefers-color-scheme: dark",
        "prefers-reduced-motion: reduce",
        "prefers-contrast: more",
        "forced-colors: active",
    ] {
        assert!(css.contains(&format!("@media ({query})")), "no @media ({query})");
    }
}

#[test]
fn focus_is_always_visible() {
    let css = css();
    let focus = rule_after(&css, 0, ":focus-visible").expect("no :focus-visible rule");
    assert!(focus.contains("box-shadow: var(--focus-outline)"), "focus rule: {focus}");
    let forced = css.find("@media (forced-colors: active)").expect("no forced-colors block");
    let forced_focus = rule_after(&css, forced, ":focus-visible").expect("no forced-colors focus rule");
    assert!(forced_focus.contains("outline: 3px"), "forced-colors focus rule: {forced_focus}");
}

#[test]
fn no_text_below_16px() {
    let css = css();
    for (i, _) in css.match_indices("font-size:") {
        let value = css[i + "font-size:".len()..].trim_start();
        let digits: String = value.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if digits.is_empty() {
            continue; // var(--size-*) tokens are checked where they are defined
        }
        let unit = &value[digits.len()..];
        let n: f64 = digits.parse().expect("number");
        let px = if unit.starts_with("px") {
            n
        } else if unit.starts_with("rem") || unit.starts_with("em") {
            n * 16.0
        } else {
            continue;
        };
        assert!(px >= 16.0, "font-size {digits}{} is below 16px", &unit[..unit.find(';').unwrap_or(unit.len())]);
    }
    // The size tokens themselves.
    for (i, _) in css.match_indices("--size-") {
        let decl = &css[i..i + css[i..].find(';').unwrap_or(0)];
        if let Some(v) = decl.split(':').nth(1).and_then(|v| v.trim().strip_suffix("rem")) {
            let n: f64 = v.parse().expect("rem number");
            assert!(n >= 1.0, "{decl} is below 16px");
        }
    }
}

#[test]
fn skip_link_first_in_body() {
    for (p, html) in html_files() {
        let Some(i) = html.find("<body>") else {
            continue; // alias redirect stubs carry no body content
        };
        let first = html[i + "<body>".len()..].trim_start();
        assert!(
            first.starts_with("<a class=\"skip-link\" href=\"#main\">"),
            "{}: body starts with {:?}",
            p.display(),
            &first[..first.len().min(60)]
        );
        assert!(html.contains("<main id=\"main\">"), "{}: no <main id=\"main\">", p.display());
    }
}
