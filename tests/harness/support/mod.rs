//! Shared fixtures for the harness.
//!
//! The harness treats the build as a black box: it runs the `bliki` binary
//! (`CARGO_BIN_EXE_bliki`) into temp directories and checks the emitted
//! files against `spec/`. The real vault is built once per test run and
//! shared; tests that need a modified vault copy it.

pub mod html;
pub mod iso639;
pub mod mf2;
pub mod regex;
pub mod schema;
pub mod xml;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use posse_bliki_build::json::{self, Json};

pub fn repo() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

pub fn content_dir() -> PathBuf {
    repo().join("content")
}

/// A fresh, empty directory under the system temp dir.
pub fn temp_dir(label: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let n = N.fetch_add(1, Ordering::SeqCst);
    let p = std::env::temp_dir().join(format!("bliki-harness-{}-{label}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create temp dir");
    p
}

/// Run `bliki` with `args` from the repo root.
pub fn bliki(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bliki"))
        .current_dir(repo())
        .args(args)
        .output()
        .expect("run bliki")
}

/// Build `content` into `out` (plus `extra` flags). `Err` holds stderr when the build fails.
pub fn build(content: &Path, out: &Path, extra: &[&str]) -> Result<Output, String> {
    let mut args = vec!["--content", content.to_str().unwrap(), "--out", out.to_str().unwrap()];
    args.extend_from_slice(extra);
    let proc = bliki(&args);
    if proc.status.success() {
        Ok(proc)
    } else {
        Err(String::from_utf8_lossy(&proc.stderr).into_owned())
    }
}

/// The real vault, built once.
pub fn site() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let out = temp_dir("site");
        build(&content_dir(), &out, &[]).expect("build the vault");
        out
    })
}

/// `bliki --print-config`, parsed.
pub fn config() -> &'static Json {
    static CONFIG: OnceLock<Json> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let proc = bliki(&["--print-config"]);
        assert!(proc.status.success(), "--print-config failed");
        json::parse(&String::from_utf8(proc.stdout).unwrap()).expect("config is JSON")
    })
}

pub fn config_str(key: &str) -> &'static str {
    config().str_at(key).unwrap_or_else(|| panic!("config has no {key}"))
}

/// The configured syndication targets as `(target, endpoint)`.
pub fn syndicate_to() -> Vec<(&'static str, &'static str)> {
    let targets = config().get("syndicate_to").expect("syndicate_to").as_arr();
    targets
        .iter()
        .map(|t| (t.str_at("target").unwrap(), t.str_at("endpoint").unwrap()))
        .collect()
}

/// Every file under `root` with extension `ext`, sorted.
pub fn files_with_ext(root: &Path, ext: &str) -> Vec<PathBuf> {
    fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).expect("read dir") {
            let p = entry.expect("dir entry").path();
            if p.is_dir() {
                walk(&p, ext, out);
            } else if p.extension().is_some_and(|e| e == ext) {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, ext, &mut out);
    out.sort();
    out
}

/// Every emitted HTML page of the real vault with its text.
pub fn html_files() -> &'static [(PathBuf, String)] {
    static FILES: OnceLock<Vec<(PathBuf, String)>> = OnceLock::new();
    FILES.get_or_init(|| {
        files_with_ext(site(), "html")
            .into_iter()
            .map(|p| (p.clone(), read(&p)))
            .collect()
    })
}

/// `index.html` of each post, excluding the `/notes/` list page itself.
pub fn post_html_files() -> Vec<PathBuf> {
    let notes = site().join("notes");
    files_with_ext(&notes, "html")
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n == "index.html") && p.parent() != Some(notes.as_path()))
        .collect()
}

/// The file's text; for a page, with each tag back on one line
/// (`flat_tags`), so checks match attributes as `<a class="x" href="y">`.
pub fn read(p: &Path) -> String {
    let text = read_as_built(p);
    if p.extension().is_some_and(|e| e == "html") {
        flat_tags(&text)
    } else {
        text
    }
}

/// The file's text byte for byte.
pub fn read_as_built(p: &Path) -> String {
    fs::read_to_string(p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// `html` with the whitespace between a tag's attributes joined to single
/// spaces, undoing the build's one-attribute-per-line layout
/// (ADR 2026-10-28-one-attribute-per-line). Text, quoted values and the
/// contents of `<script>` and `<style>` are untouched.
pub fn flat_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find('<') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if rest.starts_with("<!--") {
            let end = rest.find("-->").map_or(rest.len(), |j| j + 3);
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        let mut quote = None;
        let mut space = false;
        let mut end = rest.len();
        for (j, c) in rest.char_indices() {
            match (quote, c) {
                (None, c) if c.is_ascii_whitespace() => {
                    space = true;
                    continue;
                }
                (None, '>') => {
                    out.push('>');
                    end = j + 1;
                    break;
                }
                (None, '"' | '\'') => quote = Some(c),
                (Some(q), _) if c == q => quote = None,
                _ => {}
            }
            if std::mem::take(&mut space) {
                out.push(' ');
            }
            out.push(c);
        }
        let name: String = rest[1..end].chars().take_while(char::is_ascii_alphanumeric).collect();
        rest = &rest[end..];
        if matches!(name.to_ascii_lowercase().as_str(), "script" | "style") {
            let j = rest.find("</").unwrap_or(rest.len());
            out.push_str(&rest[..j]);
            rest = &rest[j..];
        }
    }
    out.push_str(rest);
    out
}

/// `p` relative to `root`, for failure messages.
pub fn rel(p: &Path, root: &Path) -> String {
    p.strip_prefix(root).unwrap_or(p).display().to_string()
}

/// A copy of `content/` to modify.
pub fn vault_copy(label: &str) -> PathBuf {
    fn copy(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).expect("mkdir");
        for entry in fs::read_dir(src).expect("read dir") {
            let p = entry.expect("dir entry").path();
            let to = dst.join(p.file_name().unwrap());
            if p.is_dir() {
                copy(&p, &to);
            } else {
                fs::copy(&p, &to).expect("copy");
            }
        }
    }
    let dst = temp_dir(label).join("vault");
    copy(&content_dir(), &dst);
    dst
}

/// Every `href="..."` value in `html`, in order.
pub fn hrefs(html: &str) -> Vec<&str> {
    attr_values(html, "href")
}

/// Every `name="..."` attribute value in `html`, in order.
pub fn attr_values<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let key = format!("{name}=\"");
    html.match_indices(&key)
        .filter(|(i, _)| *i == 0 || html.as_bytes()[i - 1].is_ascii_whitespace())
        .filter_map(|(i, _)| {
            let start = i + key.len();
            html[start..].find('"').map(|end| &html[start..start + end])
        })
        .collect()
}

/// `href` without its `#fragment` or `?query`.
pub fn strip_fragment(href: &str) -> &str {
    &href[..href.find(['#', '?']).unwrap_or(href.len())]
}
