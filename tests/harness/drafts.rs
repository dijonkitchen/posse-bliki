//! `draft: true` notes are excluded from the build entirely
//! (spec/content-schema.md).

use std::path::PathBuf;
use std::sync::OnceLock;

use crate::support::{build, read, temp_dir, vault_copy};

fn draft_site() -> &'static PathBuf {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let vault = vault_copy("draft");
        std::fs::write(
            vault.join("notes/secret-draft.md"),
            "---\ntitle: Secret draft\ndate: 2026-05-09\ndraft: true\n---\n\nthis should never appear in output\n",
        )
        .unwrap();
        let out = temp_dir("draft-out");
        build(&vault, &out, &[]).expect("build with a draft");
        out
    })
}

#[test]
fn draft_html_not_emitted() {
    assert!(!draft_site().join("notes/secret-draft/index.html").exists());
}

#[test]
fn draft_not_in_rss() {
    let rss = read(&draft_site().join("index.xml"));
    assert!(!rss.contains("secret-draft") && !rss.contains("Secret draft"));
}

#[test]
fn draft_not_in_jsonfeed() {
    assert!(!read(&draft_site().join("feed.json")).contains("secret-draft"));
}

#[test]
fn draft_not_in_sitemap() {
    assert!(!read(&draft_site().join("sitemap.xml")).contains("secret-draft"));
}
