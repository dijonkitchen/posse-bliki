//! rel=me, rel=author, rel=canonical and webmention; no pingback
//! (spec/indieweb.md, spec/output-contract.md).

use std::path::Path;

use crate::support::{config_str, files_with_ext, html, html_files, mf2, post_html_files, read, rel, site};

fn parsed(path: &Path) -> mf2::Parsed {
    mf2::parse(&html::parse(&read(path)).expect("valid HTML"))
}

#[test]
fn home_has_rel_me() {
    assert!(
        !parsed(&site().join("index.html")).rel("me").is_empty(),
        "home page has no rel=me links"
    );
}

/// The author's address, built from the config the way the site builds it,
/// so the address isn't written out whole here either
/// (ADR 2026-11-06-coded-email).
fn author_email() -> String {
    let user = crate::support::config()
        .get("author")
        .and_then(|a| a.str_at("email_user"))
        .expect("config has no author.email_user");
    let host = config_str("base_url")
        .split("://")
        .nth(1)
        .expect("base_url has a scheme");
    let host = host.split('/').next().unwrap();
    format!("{user}@{}", host.trim_start_matches("www."))
}

fn coded(s: &str) -> String {
    s.chars().map(|c| format!("&#{};", c as u32)).collect()
}

/// Email works for IndieLogin as rel=me, next to GitHub.
#[test]
fn home_rel_me_includes_the_email_address() {
    let mailto = format!("mailto:{}", author_email());
    let me = parsed(&site().join("index.html")).rel("me").to_vec();
    assert!(me.contains(&mailto), "home rel=me has no {mailto}: {me:?}");
    assert!(
        me.iter().any(|u| u.starts_with("https://github.com/")),
        "home rel=me lost GitHub: {me:?}"
    );
}

/// The address appears only in HTML character codes, which browsers and
/// parsers decode and most harvesters miss.
#[test]
fn no_built_file_spells_out_the_email_address() {
    let email = author_email();
    for ext in ["html", "xml", "json", "txt", "opml", "webmanifest"] {
        for p in files_with_ext(site(), ext) {
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            assert!(!text.contains(&email), "{} spells out {email}", rel(&p, site()));
        }
    }
}

/// `[text](mailto:)` in content becomes a coded link to the author.
#[test]
fn contact_links_are_coded() {
    let href = format!("href=\"{}\"", coded(&format!("mailto:{}", author_email())));
    for page in ["bio/index.html", "about/index.html"] {
        let text = read(&site().join(page));
        assert!(text.contains(&href), "{page} has no coded email link");
        assert!(!text.contains("href=\"mailto:\""), "{page} has an empty mailto: link");
    }
}

/// About says why to email, crediting the standing invitations it borrows
/// from, and Bio points there (ADR 2026-11-06-standing-invitation).
#[test]
fn about_gives_a_standing_invitation_to_email() {
    let about = read(&site().join("about/index.html"));
    for source in [
        "https://www.kalzumeus.com/standing-invitation/",
        "https://sive.rs/contact",
    ] {
        assert!(
            about.contains(&format!("href=\"{source}\"")),
            "About doesn't credit {source}"
        );
    }
    let bio = read(&site().join("bio/index.html"));
    assert!(
        bio.contains("href=\"/about/\""),
        "Bio doesn't point to About's invitation"
    );
}

#[test]
fn every_page_advertises_webmention() {
    for (p, text) in html_files() {
        let rels = mf2::parse(&html::parse(text).unwrap());
        assert!(
            !rels.rel("webmention").is_empty(),
            "{}: missing rel=webmention",
            rel(p, site())
        );
        // Pingback is retired in favour of Webmention
        // (ADR 2026-11-06-drop-pingback).
        assert!(
            rels.rel("pingback").is_empty(),
            "{}: still advertises rel=pingback",
            rel(p, site())
        );
    }
}

#[test]
fn every_post_has_rel_author() {
    for f in post_html_files() {
        assert!(
            !parsed(&f).rel("author").is_empty(),
            "{}: missing rel=author",
            rel(&f, site())
        );
    }
}

/// The canonical link on `<dir>/index.html` points to `<base>/<dir>/`. Alias
/// redirect pages point at their target instead (spec/url-policy.md).
#[test]
fn canonical_matches_url() {
    let base = config_str("base_url").trim_end_matches('/');
    let root = site();
    for f in files_with_ext(root, "html")
        .into_iter()
        .filter(|p| p.ends_with("index.html"))
    {
        let text = read(&f);
        if text.contains("http-equiv=\"refresh\"") {
            continue;
        }
        let rels = mf2::parse(&html::parse(&text).unwrap());
        let Some(url) = rels.rel("canonical").first() else {
            continue;
        };
        let dir = f.parent().unwrap().strip_prefix(root).unwrap().display().to_string();
        let expected = if dir.is_empty() {
            format!("{base}/")
        } else {
            format!("{base}/{dir}/")
        };
        assert_eq!(*url, expected, "{}", rel(&f, root));
    }
}

/// The IndieWeb Webring checks the home page for its previous/next links
/// and marks the site inactive without them (ADR 2026-11-06-indieweb-webring).
/// The random link sits beside them (ADR 2026-11-07-webring-random-link).
#[test]
fn every_footer_links_the_webring() {
    let ring = [
        r#"href="https://xn--sr8hvo.ws/previous""#,
        r#"href="https://xn--sr8hvo.ws""#,
        r#"href="https://xn--sr8hvo.ws/next""#,
        r#"href="https://xn--sr8hvo.ws/random""#,
    ];
    let missing: Vec<String> = html_files()
        .iter()
        .filter(|(_, h)| {
            let footer = h.split("<footer class=\"site-footer\">").nth(1).unwrap_or("");
            !ring.iter().all(|l| footer.contains(l))
        })
        .map(|(p, _)| rel(p, site()))
        .collect();
    assert!(
        missing.is_empty(),
        "pages without the webring links in the footer:\n{}",
        missing.join("\n")
    );
}
