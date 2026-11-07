//! robots.txt welcomes search crawlers and asks AI training crawlers to
//! stay out (spec/feeds.md, ADR 2026-11-06-no-ai-training).

use crate::support::{config_str, read, site};

/// Each group's user agents and rules, per RFC 9309: consecutive
/// `User-agent` lines start a group, and the rules after them belong to it.
fn groups(text: &str) -> Vec<(Vec<String>, Vec<String>)> {
    let mut out: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    let mut in_agents = false;
    for line in text.lines().map(|l| l.split('#').next().unwrap().trim()) {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim().to_ascii_lowercase(), value.trim().to_string());
        match key.as_str() {
            "user-agent" if in_agents => out.last_mut().unwrap().0.push(value),
            "user-agent" => {
                out.push((vec![value], Vec::new()));
                in_agents = true;
            }
            "allow" | "disallow" => {
                out.last_mut()
                    .expect("rule before any User-agent")
                    .1
                    .push(format!("{key}: {value}"));
                in_agents = false;
            }
            _ => in_agents = false,
        }
    }
    out
}

fn robots() -> String {
    read(&site().join("robots.txt"))
}

#[test]
fn everyone_else_may_crawl_everything() {
    let all = groups(&robots());
    let star: Vec<_> = all.iter().filter(|(a, _)| a.iter().any(|a| a == "*")).collect();
    assert_eq!(star.len(), 1, "want exactly one User-agent: * group");
    assert_eq!(star[0].1, ["allow: /"], "User-agent: * must allow everything");
}

#[test]
fn ai_training_crawlers_are_disallowed() {
    let all = groups(&robots());
    for agent in [
        "GPTBot",
        "ClaudeBot",
        "Google-Extended",
        "Applebot-Extended",
        "meta-externalagent",
        "Amazonbot",
        "CCBot",
    ] {
        let group = all
            .iter()
            .find(|(a, _)| a.iter().any(|a| a.eq_ignore_ascii_case(agent)))
            .unwrap_or_else(|| panic!("robots.txt has no group for {agent}"));
        assert_eq!(group.1, ["disallow: /"], "{agent} must be disallowed from everything");
    }
}

#[test]
fn search_crawlers_are_not_named() {
    // A named search agent would leave the `*` group, so a rule written
    // for it could block search by mistake.
    let named: Vec<String> = groups(&robots())
        .into_iter()
        .flat_map(|(a, _)| a)
        .map(|a| a.to_ascii_lowercase())
        .collect();
    for agent in [
        "googlebot",
        "bingbot",
        "applebot",
        "oai-searchbot",
        "claude-searchbot",
        "amzn-searchbot",
    ] {
        assert!(!named.iter().any(|a| a == agent), "robots.txt names {agent}");
    }
}

#[test]
fn advertises_the_sitemap() {
    let base = config_str("base_url").trim_end_matches('/').to_string();
    assert!(
        robots().lines().any(|l| l == format!("Sitemap: {base}/sitemap.xml")),
        "robots.txt doesn't advertise the sitemap"
    );
}
