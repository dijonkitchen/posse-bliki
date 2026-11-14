//! The site has no tags (spec/content-schema.md, ADR 2026-11-06-drop-tags):
//! a `#word` in a note stays text, and a `tags` field fails the build.

use crate::support::{build, hrefs, html_files, read, rel, site, temp_dir, vault_copy};

#[test]
fn hash_word_stays_text() {
    let vault = vault_copy("hash-word");
    std::fs::write(
        vault.join("notes/hash-word.md"),
        "---\ntitle: Hash word\ncreated_at: 2026-05-09\n---\n\nWe're #1 at #ideas.\n",
    )
    .unwrap();
    let out = temp_dir("hash-word-out");
    build(&vault, &out, &[]).expect("build with a #word");
    let html = read(&out.join("notes/hash-word/index.html"));
    assert!(
        html.contains("We&#39;re #1 at #ideas."),
        "the #words should stay as written"
    );
    assert!(!out.join("tags/ideas").exists(), "a #word should not make a tag page");
}

#[test]
fn tags_field_fails_the_build() {
    let vault = vault_copy("tags-field");
    std::fs::write(
        vault.join("notes/tagged.md"),
        "---\ntitle: Tagged\ncreated_at: 2026-05-09\ntags: [podcast]\n---\n\nbody\n",
    )
    .unwrap();
    let err = build(&vault, &temp_dir("tags-field-out"), &[]).expect_err("build should fail");
    assert!(err.contains("'tags' was unexpected"), "{err}");
}

#[test]
fn no_page_links_to_a_tag_page() {
    for (p, html) in html_files() {
        for href in hrefs(html) {
            assert!(!href.starts_with("/tags/"), "{} links to {href}", rel(p, site()));
        }
    }
}
