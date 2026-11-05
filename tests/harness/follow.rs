//! The follow page: how to follow the site and its blogroll in a feed reader
//! (spec/feeds.md, ADR 2026-11-05-follow-with-a-feed-reader).

use std::fs;

use crate::support::{build, config_str, content_dir, html_files, read, rel, site, temp_dir, vault_copy};

const FOOTER_LINK: &str = r#"<a href="/follow/">Follow</a>"#;

fn footer(html: &str) -> &str {
    html.split("<footer class=\"site-footer\">").nth(1).unwrap_or("")
}

/// The `<ul class="follow-feeds">` the build adds to the follow page.
fn feeds(html: &str) -> &str {
    html.split("<ul class=\"follow-feeds\">")
        .nth(1)
        .and_then(|rest| rest.split("</ul>").next())
        .expect("no follow-feeds list on the follow page")
}

#[test]
fn follow_page_suggests_a_feed_reader() {
    let md = read(&content_dir().join("follow.md"));
    assert!(md.contains("FeedFlow"), "follow.md doesn't suggest FeedFlow");
    assert!(
        md.contains("(blogroll.md)"),
        "follow.md doesn't point readers at the blogroll"
    );
    let html = read(&site().join("follow/index.html"));
    assert!(html.contains(r#"href="https://www.feedflow.dev/""#));
}

/// JC asked the page to explain that following is open web standards,
/// with no algorithm or tracking in between.
#[test]
fn follow_page_explains_open_standards_without_algorithms_or_tracking() {
    let md = read(&content_dir().join("follow.md"));
    for spec in [
        "https://www.rssboard.org/rss-specification",
        "https://www.jsonfeed.org/version/1.1/",
        "https://opml.org/spec2.opml",
    ] {
        assert!(md.contains(&format!("({spec})")), "follow.md doesn't link {spec}");
    }
    for point in ["Open web standards", "No algorithm", "No tracking"] {
        assert!(md.contains(point), "follow.md doesn't say \"{point}\"");
    }
}

#[test]
fn follow_page_spells_out_the_feed_and_opml_addresses() {
    let html = read(&site().join("follow/index.html"));
    let list = feeds(&html);
    let base = config_str("base_url");
    for (path, kind) in [("/index.xml", "application/rss+xml"), ("/blogroll.opml", "text/x-opml")] {
        assert!(
            list.contains(&format!(r#"<a href="{path}" type="{kind}">{base}<wbr>{path}</a>"#)),
            "follow page doesn't spell out {base}{path}:\n{list}"
        );
        assert!(
            site().join(path.trim_start_matches('/')).is_file(),
            "{path} isn't built"
        );
    }
}

#[test]
fn every_page_footer_links_the_follow_page_first() {
    let wrong: Vec<String> = html_files()
        .iter()
        .filter(|(_, h)| {
            let f = footer(h);
            let follow = f.find(FOOTER_LINK);
            let rss = f.find(r#"<a href="/index.xml">RSS</a>"#);
            !matches!((follow, rss), (Some(a), Some(b)) if a < b)
        })
        .map(|(p, _)| rel(p, site()))
        .collect();
    assert!(
        wrong.is_empty(),
        "pages whose footer doesn't link Follow before RSS:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn substack_subscribe_link_lands_on_the_follow_page() {
    let html = read(&site().join("subscribe/index.html"));
    assert!(
        html.contains("url=/follow/"),
        "/subscribe/ doesn't redirect to /follow/"
    );
}

#[test]
fn no_follow_page_means_no_follow_link() {
    let vault = vault_copy("no-follow");
    fs::remove_file(vault.join("follow.md")).unwrap();
    for page in ["about.md", "blogroll.md"] {
        let p = vault.join(page);
        let kept = read(&p).replace("(follow.md)", "(colophon.md)");
        fs::write(&p, kept).unwrap();
    }
    let out = temp_dir("no-follow-out");
    build(&vault, &out, &[]).expect("build without a follow page");
    let home = read(&out.join("index.html"));
    assert!(!home.contains("/follow/"), "links a follow page that doesn't exist");
}

#[test]
fn without_a_blogroll_the_follow_page_lists_only_the_feed() {
    let vault = vault_copy("follow-no-blogroll");
    fs::remove_file(vault.join("blogroll.md")).unwrap();
    for page in ["follow.md", "colophon.md"] {
        let p = vault.join(page);
        fs::write(&p, read(&p).replace("[blogroll](blogroll.md)", "blogroll")).unwrap();
    }
    let out = temp_dir("follow-no-blogroll-out");
    build(&vault, &out, &[]).expect("build without a blogroll");
    let html = read(&out.join("follow/index.html"));
    let list = feeds(&html);
    assert!(list.contains("/index.xml"), "no feed address:\n{list}");
    assert!(
        !list.contains("blogroll.opml"),
        "lists an OPML file that isn't built:\n{list}"
    );
}
