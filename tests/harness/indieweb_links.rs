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
