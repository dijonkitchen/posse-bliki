//! Aliases produce HTML redirect pages with both meta-refresh and rel=canonical
//! (spec/url-policy.md).

use std::path::PathBuf;
use std::sync::OnceLock;

use crate::support::{build, read, temp_dir, vault_copy};

const ALIAS: &str = "/old-location/";
const TARGET: &str = "/notes/moved-note/";

fn aliased_site() -> &'static PathBuf {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let vault = vault_copy("alias");
        std::fs::write(
            vault.join("notes/moved-note.md"),
            "---\ntitle: A moved note\ndate: 2026-05-08\naliases:\n  - /old-location/\n  - /even-older/\n---\n\nbody\n",
        )
        .unwrap();
        let out = temp_dir("alias-out");
        build(&vault, &out, &[]).expect("build with alias");
        out
    })
}

fn redirect() -> String {
    read(&aliased_site().join("old-location/index.html"))
}

#[test]
fn alias_files_exist() {
    assert!(
        aliased_site().join("old-location/index.html").is_file(),
        "alias {ALIAS} did not produce a redirect file"
    );
}

#[test]
fn alias_has_meta_refresh() {
    let html = redirect();
    assert!(html.contains("http-equiv=\"refresh\""));
    assert!(html.contains(TARGET));
}

#[test]
fn alias_has_canonical() {
    let html = redirect();
    assert!(html.contains("rel=\"canonical\""));
    assert!(html.contains(TARGET));
}
