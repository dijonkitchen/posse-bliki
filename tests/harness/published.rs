//! Only a file marked `published: true` is built, and only it is checked.
//! Anything else in `content/` is skipped before its front matter is read
//! for anything but `published`, so a note still being written can't break
//! the deploy (spec/content-schema.md, ADR 2026-11-14-published-checkbox).

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::support::{build, read, temp_dir, vault_copy};

/// Each way a note can be unpublished, as Obsidian leaves it on a phone:
/// unticked, never ticked, freshly stamped by Linter, or still empty.
const UNPUBLISHED: [(&str, &str); 4] = [
    (
        "unticked",
        "---\ntitle: Unticked\ncreated_at: 2026-05-09\npublished: false\n---\n\nthis should never appear in output\n",
    ),
    (
        "never-ticked",
        "---\ntitle: Never ticked\ncreated_at: 2026-05-09\n---\n\nthis should never appear in output\n",
    ),
    (
        "stamped",
        "---\ncreated_at: 2026-11-14T23:40:00Z\nupdated_at: 2026-11-14T23:40:05Z\n---\n\nhalf a thought, linking [a note](not-yet.md)\n",
    ),
    ("untitled", "half a thought\n"),
];

fn unpublished_site() -> &'static PathBuf {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let vault = vault_copy("unpublished");
        for (slug, text) in UNPUBLISHED {
            fs::write(vault.join(format!("notes/{slug}.md")), text).unwrap();
        }
        let out = temp_dir("unpublished-out");
        build(&vault, &out, &[]).expect("unpublished notes don't break the build");
        out
    })
}

#[test]
fn unpublished_html_not_emitted() {
    for (slug, _) in UNPUBLISHED {
        assert!(
            !unpublished_site().join(format!("notes/{slug}/index.html")).exists(),
            "{slug} was built"
        );
    }
}

#[test]
fn unpublished_not_in_feeds_or_sitemap() {
    for file in ["index.xml", "feed.json", "sitemap.xml", "notes/index.html"] {
        let text = read(&unpublished_site().join(file));
        for (slug, _) in UNPUBLISHED {
            assert!(!text.contains(&format!("/notes/{slug}/")), "{slug} is in {file}");
        }
        assert!(
            !text.contains("should never appear"),
            "an unpublished note's text is in {file}"
        );
    }
}

fn build_with(label: &str, slug: &str, text: &str) -> Result<(), String> {
    let vault = vault_copy(label);
    fs::write(vault.join(format!("notes/{slug}.md")), text).unwrap();
    build(&vault, &temp_dir(&format!("{label}-out")), &[]).map(|_| ())
}

#[test]
fn published_note_is_checked() {
    let err = build_with("published-untitled", "untitled", "---\npublished: true\n---\n\nhi\n")
        .expect_err("a published note with no title fails the build");
    assert!(
        err.contains("'title' is a required property"),
        "unexpected error: {err}"
    );
}

#[test]
fn published_must_be_true_or_false() {
    let err = build_with(
        "published-string",
        "quoted",
        "---\ntitle: Quoted\ncreated_at: 2026-05-09\npublished: \"true\"\n---\n\nhi\n",
    )
    .expect_err("a quoted published fails the build");
    assert!(
        err.contains("`published` must be true or false"),
        "unexpected error: {err}"
    );
}

#[test]
fn draft_is_gone() {
    let err = build_with(
        "published-draft",
        "both",
        "---\ntitle: Both\ncreated_at: 2026-05-09\npublished: true\ndraft: true\n---\n\nhi\n",
    )
    .expect_err("draft on a published note fails the build");
    assert!(err.contains("`draft` is gone"), "unexpected error: {err}");
}

#[test]
fn link_to_an_unpublished_note_fails_the_build() {
    let vault = vault_copy("published-link");
    fs::write(vault.join("notes/later.md"), UNPUBLISHED[1].1).unwrap();
    fs::write(
        vault.join("notes/now-live.md"),
        "---\ntitle: Now live\ncreated_at: 2026-05-09\npublished: true\n---\n\nSee [later](later.md).\n",
    )
    .unwrap();
    let err = build(&vault, &temp_dir("published-link-out"), &[]).expect_err("a link to an unpublished note fails");
    assert!(err.contains("broken link 'later.md'"), "unexpected error: {err}");
}
