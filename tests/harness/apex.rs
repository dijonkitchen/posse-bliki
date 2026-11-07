//! `apex/_redirects`, the bare domain's redirect-only Netlify site
//! (ADR 2026-11-07-fediverse-handle-on-own-domain).

use crate::support::{config_str, read, repo};

/// Each rule as (from, to, status), comments and blank lines dropped.
fn rules() -> Vec<(String, String, String)> {
    read(&repo().join("apex/_redirects"))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            assert_eq!(f.len(), 3, "from, to, status: {l}");
            (f[0].into(), f[1].into(), f[2].into())
        })
        .collect()
}

#[test]
fn fediverse_lookups_go_to_bridgy_fed() {
    // Netlify uses the first matching rule, so these must come before the
    // catch-all. Paths and targets are Bridgy Fed's own
    // (https://fed.brid.gy/docs#fediverse-enhanced).
    let rules = rules();
    let want = [
        (
            "/.well-known/host-meta*",
            "https://fed.brid.gy/.well-known/host-meta:splat",
        ),
        ("/.well-known/webfinger*", "https://fed.brid.gy/.well-known/webfinger"),
    ];
    for (i, (from, to)) in want.iter().enumerate() {
        let (f, t, s) = &rules[i];
        assert_eq!((f.as_str(), t.as_str()), (*from, *to), "rule {}", i + 1);
        assert_eq!(s, "302", "{from} should be a temporary redirect");
    }
}

#[test]
fn everything_else_goes_to_the_canonical_host() {
    let base = config_str("base_url").trim_end_matches('/');
    let rules = rules();
    let last = rules.last().expect("rules");
    assert_eq!(rules.len(), 3, "{rules:?}");
    assert_eq!(
        (last.0.as_str(), last.1.as_str(), last.2.as_str()),
        ("/*", format!("{base}/:splat").as_str(), "301!")
    );
}
