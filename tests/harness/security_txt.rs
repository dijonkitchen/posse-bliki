//! `/.well-known/security.txt` per RFC 9116 (spec/url-policy.md,
//! ADR 2026-10-28-security-txt).

use crate::support::{build, config_str, content_dir, read, temp_dir};

fn security_txt(today: &str) -> String {
    let out = temp_dir(&format!("security-txt-{today}"));
    build(&content_dir(), &out, &["--today", today]).expect("build");
    read(&out.join(".well-known/security.txt"))
}

#[test]
fn has_the_required_and_canonical_fields() {
    let text = security_txt("2026-10-28");
    let base = config_str("base_url").trim_end_matches('/');
    let fields: Vec<(&str, &str)> = text
        .lines()
        .map(|l| l.split_once(": ").expect("Field: value"))
        .collect();
    let get = |k: &str| {
        fields
            .iter()
            .filter(|(f, _)| *f == k)
            .map(|(_, v)| *v)
            .collect::<Vec<_>>()
    };
    assert!(
        get("Contact")
            .iter()
            .all(|c| c.starts_with("https://") || c.starts_with("mailto:")),
        "{text}"
    );
    assert!(!get("Contact").is_empty(), "no Contact:\n{text}");
    assert_eq!(get("Expires"), ["2027-04-26T00:00:00Z"], "{text}");
    assert_eq!(
        get("Canonical"),
        [format!("{base}/.well-known/security.txt").as_str()],
        "{text}"
    );
}

#[test]
fn expires_moves_with_the_build_date() {
    // The daily rebuild keeps Expires under a year ahead, as RFC 9116 asks.
    assert!(security_txt("2027-01-01").contains("Expires: 2027-06-30T00:00:00Z\n"));
}

#[test]
fn deploy_ships_the_well_known_directory() {
    // upload-pages-artifact skips dot-directories unless told otherwise.
    let deploy = read(&crate::support::repo().join(".github/workflows/deploy.yml"));
    assert!(
        deploy.contains("include-hidden-files: true"),
        "deploy.yml must upload /.well-known/ (include-hidden-files: true)"
    );
}
