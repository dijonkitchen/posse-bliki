//! rel=me, rel=author, rel=canonical, webmention and pingback
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

#[test]
fn every_page_advertises_webmention() {
    for (p, text) in html_files() {
        let rels = mf2::parse(&html::parse(text).unwrap());
        assert!(
            !rels.rel("webmention").is_empty(),
            "{}: missing rel=webmention",
            rel(p, site())
        );
        assert!(
            !rels.rel("pingback").is_empty(),
            "{}: missing rel=pingback",
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
#[test]
fn every_footer_links_the_webring() {
    let ring = [
        r#"href="https://xn--sr8hvo.ws/previous""#,
        r#"href="https://xn--sr8hvo.ws""#,
        r#"href="https://xn--sr8hvo.ws/next""#,
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
