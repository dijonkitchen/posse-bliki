//! A note dated after `--today` is held back from the build, like a draft,
//! and goes live on the first build on or after its date (spec/content-schema.md,
//! ADR 2026-10-20-scheduled-posts).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::support::{build, read, temp_dir, vault_copy};

const POST: &str = "---\ntitle: Scheduled post\ndate: 2099-01-02\n---\n\nnot out until 2099\n";

fn vault() -> &'static PathBuf {
    static VAULT: OnceLock<PathBuf> = OnceLock::new();
    VAULT.get_or_init(|| {
        let vault = vault_copy("scheduled");
        std::fs::write(vault.join("notes/scheduled-post.md"), POST).unwrap();
        vault
    })
}

fn built(today: &str) -> PathBuf {
    let out = temp_dir(&format!("scheduled-{}", today));
    build(vault(), &out, &["--today", today]).expect("build with a scheduled post");
    out
}

fn before() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| built("2099-01-01"))
}

fn on_the_day() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| built("2099-01-02"))
}

const OUTPUTS: [&str; 4] = ["index.html", "index.xml", "feed.json", "sitemap.xml"];

#[test]
fn held_back_before_its_date() {
    assert!(!before().join("notes/scheduled-post/index.html").exists());
    for f in OUTPUTS {
        assert!(
            !read(&before().join(f)).contains("scheduled-post"),
            "{} lists it early",
            f
        );
    }
}

#[test]
fn live_on_its_date() {
    assert!(on_the_day().join("notes/scheduled-post/index.html").exists());
    for f in OUTPUTS {
        assert!(
            read(&on_the_day().join(f)).contains("scheduled-post"),
            "{} misses it",
            f
        );
    }
}

#[test]
fn link_to_a_scheduled_post_fails_the_build() {
    let vault = vault_copy("scheduled-link");
    std::fs::write(vault.join("notes/scheduled-post.md"), POST).unwrap();
    std::fs::write(
        vault.join("notes/teaser.md"),
        "---\ntitle: Teaser\ndate: 2026-01-01\n---\n\nSee [soon](scheduled-post.md).\n",
    )
    .unwrap();
    let err = build(&vault, &temp_dir("scheduled-link-out"), &["--today", "2098-12-31"])
        .expect_err("a live post links to a held-back one");
    assert!(err.contains("broken link 'scheduled-post.md'"), "{}", err);
}

#[test]
fn bad_today_is_rejected() {
    let err = build(vault(), &temp_dir("scheduled-bad"), &["--today", "2099-02-30"]).expect_err("invalid date");
    assert!(err.contains("--today"), "{}", err);
}
