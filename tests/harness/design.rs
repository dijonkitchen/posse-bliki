//! Pages follow spec/design.md: one self-hosted stylesheet built on the a11y
//! design system.

use std::fs;

use crate::support::{config_str, html_files as pages, site};

const SEMANTIC_TOKENS: &[&str] = &[
    "--text-1",
    "--text-2",
    "--text-link",
    "--surface-page",
    "--surface-sunken",
    "--border-strong",
    "--border-subtle",
    "--focus-halo",
    "--focus-ring",
];

const DARK_QUERY: &str = "@media (prefers-color-scheme: dark) {";

fn html_files() -> &'static [(std::path::PathBuf, String)] {
    let files = pages();
    assert!(files.len() >= 4, "expected several HTML files, got {}", files.len());
    files
}

fn css() -> String {
    fs::read_to_string(site().join("style.css")).expect("read style.css")
}

/// The argument of every `url(...)` in `css`, quotes stripped.
fn css_urls(css: &str) -> Vec<String> {
    css.match_indices("url(")
        .map(|(i, _)| {
            let rest = &css[i + 4..];
            let inner = &rest[..rest.find(')').expect("unclosed url(")];
            inner.trim().trim_matches(|c| c == '"' || c == '\'').to_string()
        })
        .collect()
}

/// Every `<link ...>` tag in `html`.
fn link_tags(html: &str) -> Vec<&str> {
    html.match_indices("<link ")
        .map(|(i, _)| &html[i..i + html[i..].find('>').expect("unclosed <link") + 1])
        .collect()
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

/// The rule body following `selector {` in `css`, if any.
fn rule_after<'a>(css: &'a str, from: usize, selector: &str) -> Option<&'a str> {
    let start = from + css[from..].find(&format!("{selector} {{"))? + selector.len() + 2;
    Some(&css[start..start + css[start..].find('}')?])
}

/// Split the stylesheet at the dark-mode block: (everything before, the block's body).
fn light_and_dark(css: &str) -> (&str, &str) {
    let start = css.find(DARK_QUERY).expect("no prefers-color-scheme: dark block");
    let body = &css[start + DARK_QUERY.len()..];
    let end = body.find("\n}").expect("unclosed dark block");
    (&css[..start], &body[..end])
}

#[test]
fn every_page_links_the_stylesheet() {
    for (p, html) in html_files().iter() {
        assert!(
            html.contains("<link rel=\"stylesheet\" href=\"/style.css\">"),
            "{}",
            p.display()
        );
    }
}

#[test]
fn no_inline_styles() {
    for (p, html) in html_files().iter() {
        let lower = html.to_lowercase();
        assert!(!lower.contains("<style"), "{}: <style> block", p.display());
        assert!(!lower.contains(" style="), "{}: inline style attribute", p.display());
    }
}

#[test]
fn no_third_party_requests_from_pages() {
    // rel=me, alternate and webmention links point elsewhere by design; only
    // links that make the browser fetch something must stay on this origin.
    for (p, html) in html_files().iter() {
        for tag in link_tags(html) {
            let rel = attr(tag, "rel").unwrap_or("");
            if !matches!(rel, "stylesheet" | "preload" | "icon" | "modulepreload") {
                continue;
            }
            let href = attr(tag, "href").unwrap_or("");
            assert!(
                href.starts_with('/') && !href.starts_with("//"),
                "{}: {}",
                p.display(),
                tag
            );
        }
    }
}

#[test]
fn stylesheet_is_self_contained() {
    let css = css();
    assert!(!css.contains("@import"), "style.css uses @import");
    for url in css_urls(&css) {
        assert!(
            !url.contains("//") && !url.starts_with('/') && !url.starts_with("data:"),
            "non-relative url({url})"
        );
    }
}

#[test]
fn stylesheet_urls_resolve() {
    let urls = css_urls(&css());
    assert!(!urls.is_empty(), "expected self-hosted font urls");
    for url in urls {
        assert!(
            site().join(&url).is_file(),
            "url({url}) does not resolve next to style.css"
        );
    }
}

#[test]
fn fonts_ship_with_licences() {
    let fonts = site().join("fonts");
    let mut woff2 = 0;
    for entry in fs::read_dir(&fonts).expect("no fonts/ emitted") {
        let p = entry.expect("dir entry").path();
        if p.extension().is_some_and(|e| e == "woff2") {
            woff2 += 1;
            let bytes = fs::read(&p).expect("read font");
            assert!(bytes.starts_with(b"wOF2"), "{} is not WOFF2", p.display());
        }
    }
    assert!(woff2 > 0, "no fonts emitted");
    assert!(fonts.join("OFL-atkinson-hyperlegible-next.txt").is_file());
    assert!(fonts.join("OFL-jetbrains-mono.txt").is_file());
}

#[test]
fn type_uses_the_design_system_fonts() {
    let css = css();
    assert!(css.contains("font-family: \"Atkinson Hyperlegible Next\""));
    assert!(css.contains("font-family: \"JetBrains Mono\""));
    assert!(css.contains("--font-sans:") && css.contains("--font-code:"));
}

#[test]
fn semantic_tokens_light_and_dark() {
    let css = css();
    let (light, dark) = light_and_dark(&css);
    for token in SEMANTIC_TOKENS {
        assert!(light.contains(&format!("{token}:")), "{token} missing in light");
        assert!(dark.contains(&format!("{token}:")), "{token} missing in dark");
    }
}

#[test]
fn user_preferences_honoured() {
    let css = css();
    for query in [
        "prefers-color-scheme: dark",
        "prefers-reduced-motion: reduce",
        "prefers-contrast: more",
        "forced-colors: active",
    ] {
        assert!(css.contains(&format!("@media ({query})")), "no @media ({query})");
    }
}

#[test]
fn focus_is_always_visible() {
    let css = css();
    let focus = rule_after(&css, 0, ":focus-visible").expect("no :focus-visible rule");
    assert!(
        focus.contains("box-shadow: var(--focus-outline)"),
        "focus rule: {focus}"
    );
    let forced = css
        .find("@media (forced-colors: active)")
        .expect("no forced-colors block");
    let forced_focus = rule_after(&css, forced, ":focus-visible").expect("no forced-colors focus rule");
    assert!(
        forced_focus.contains("outline: 3px"),
        "forced-colors focus rule: {forced_focus}"
    );
}

#[test]
fn no_text_below_16px() {
    let css = css();
    for (i, _) in css.match_indices("font-size:") {
        let value = css[i + "font-size:".len()..].trim_start();
        let digits: String = value.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        if digits.is_empty() {
            continue; // var(--size-*) tokens are checked where they are defined
        }
        let unit = &value[digits.len()..];
        let n: f64 = digits.parse().expect("number");
        let px = if unit.starts_with("px") {
            n
        } else if unit.starts_with("rem") || unit.starts_with("em") {
            n * 16.0
        } else {
            continue;
        };
        assert!(
            px >= 16.0,
            "font-size {digits}{} is below 16px",
            &unit[..unit.find(';').unwrap_or(unit.len())]
        );
    }
    // The size tokens themselves.
    for (i, _) in css.match_indices("--size-") {
        let decl = &css[i..i + css[i..].find(';').unwrap_or(0)];
        if let Some(v) = decl.split(':').nth(1).and_then(|v| v.trim().strip_suffix("rem")) {
            let n: f64 = v.parse().expect("rem number");
            assert!(n >= 1.0, "{decl} is below 16px");
        }
    }
}

#[test]
fn skip_link_first_in_body() {
    for (p, html) in html_files().iter() {
        let Some(i) = html.find("<body>") else {
            continue; // alias redirect stubs carry no body content
        };
        let first = html[i + "<body>".len()..].trim_start();
        assert!(
            first.starts_with("<a class=\"skip-link\" href=\"#main\">"),
            "{}: body starts with {:?}",
            p.display(),
            &first[..first.len().min(60)]
        );
        assert!(
            html.contains("<main id=\"main\">"),
            "{}: no <main id=\"main\">",
            p.display()
        );
    }
}

#[test]
fn home_heading_is_the_site_title() {
    // ADR 2026-10-16-site-title-heads-home: the site's name outranks the author's.
    let home = crate::support::read(&site().join("index.html"));
    let doc = crate::support::html::parse(&home).expect("valid HTML");
    let all = doc.descendants();
    let h1s: Vec<_> = all.iter().filter(|e| e.name == "h1").collect();
    assert_eq!(h1s.len(), 1, "home page has {} <h1>s", h1s.len());
    assert_eq!(
        h1s[0].text(),
        crate::support::config_str("title"),
        "home <h1> is not the site title"
    );
    let card = all
        .iter()
        .find(|e| e.classes().contains(&"h-card"))
        .expect("home page has no h-card");
    assert!(
        !card
            .descendants()
            .iter()
            .any(|e| e.name.starts_with('h') && e.name.len() == 2),
        "the author h-card holds a heading"
    );
}

#[test]
fn site_brand_sits_outside_the_nav() {
    // The nav landmark holds only links, so screen readers list it as navigation.
    for (p, html) in html_files().iter() {
        let Ok(doc) = crate::support::html::parse(html) else {
            continue;
        };
        for nav in doc.descendants().into_iter().filter(|e| e.name == "nav") {
            assert!(
                !nav.descendants().iter().any(|e| e.classes().contains(&"site-brand")),
                "{}: site brand is inside <nav>",
                p.display()
            );
        }
    }
}

#[test]
fn header_shows_title_and_tagline() {
    let esc = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    };
    let title = format!(">{}</a>", esc(config_str("title")));
    let tagline = format!("<p class=\"site-tagline\">{}</p>", esc(config_str("tagline")));
    for (p, html) in html_files().iter() {
        if !html.contains("<body>") {
            continue; // alias redirect stubs carry no header
        }
        assert!(
            html.contains(&title) && html.contains(&tagline),
            "{}: header lacks the title and tagline",
            p.display()
        );
    }
}

#[test]
fn home_leads_with_recent_posts() {
    // ADR 2026-10-16-home-shows-recent-posts: posts first, author in the footer.
    let home = crate::support::read(&site().join("index.html"));
    let doc = crate::support::html::parse(&home).expect("valid HTML");
    let all = doc.descendants();
    let main = all.iter().find(|e| e.name == "main").expect("no <main>");
    let first = main.elements().next().expect("empty <main>");
    assert!(
        first.classes().contains(&"h-feed"),
        "home <main> doesn't start with the h-feed"
    );
    let footer = all
        .iter()
        .find(|e| e.classes().contains(&"site-footer"))
        .expect("no site footer");
    assert!(
        footer.descendants().iter().any(|e| e.classes().contains(&"h-card")),
        "home h-card is not in the footer"
    );
}

#[test]
fn browser_bars_match_the_page() {
    // ADR 2026-10-16-browser-bars-match-page: theme-color repeats
    // --surface-page as hex, so it must track the stylesheet's light value.
    let css = css();
    assert!(
        css.contains("--paper: #fbfaf7;"),
        "--paper changed; update THEME_COLOR in build/site.rs"
    );
    for (p, html) in html_files() {
        for (scheme, colour) in [("light", "#fbfaf7"), ("dark", "#15171b")] {
            let tag =
                format!("<meta name=\"theme-color\" media=\"(prefers-color-scheme: {scheme})\" content=\"{colour}\">");
            assert!(html.contains(&tag), "{} lacks {scheme} theme-color", p.display());
        }
    }
}

/// The style garden's styles: every `build/static/themes/*.css`, by slug.
fn themes() -> Vec<(String, String)> {
    let dir = crate::support::repo().join("build/static/themes");
    let mut out: Vec<(String, String)> = fs::read_dir(&dir)
        .expect("read build/static/themes")
        .map(|f| f.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "css"))
        .map(|p| {
            let slug = p.file_stem().unwrap().to_str().unwrap().to_string();
            (slug, fs::read_to_string(&p).unwrap())
        })
        .collect();
    out.sort();
    assert!(!out.is_empty(), "expected garden styles in build/static/themes");
    out
}

#[test]
fn garden_has_a_page_per_style() {
    for (slug, source) in themes() {
        let css = fs::read_to_string(site().join(format!("themes/{slug}.css"))).expect("read theme");
        assert_eq!(css, source, "themes/{slug}.css differs from its source");
        let html = fs::read_to_string(site().join(format!("garden/{slug}/index.html"))).expect("garden page");
        let sheets: Vec<&str> = link_tags(&html)
            .into_iter()
            .filter(|t| attr(t, "rel") == Some("stylesheet"))
            .filter_map(|t| attr(t, "href"))
            .collect();
        assert_eq!(sheets, ["/style.css", &format!("/themes/{slug}.css")], "garden/{slug}/");
    }
}

#[test]
fn garden_links_every_style_and_marks_the_current_one() {
    let mut pages = vec![("garden/index.html".to_string(), "/garden/".to_string())];
    for (slug, _) in themes() {
        pages.push((format!("garden/{slug}/index.html"), format!("/garden/{slug}/")));
    }
    for (file, own) in &pages {
        let html = fs::read_to_string(site().join(file)).expect("garden page");
        for (_, url) in &pages {
            let current = if url == own { " aria-current=\"page\"" } else { "" };
            assert!(
                html.contains(&format!("<a href=\"{url}\"{current}>")),
                "{file}: link to {url}"
            );
        }
    }
}

#[test]
fn only_garden_pages_add_a_stylesheet() {
    for (p, html) in html_files().iter() {
        let sheets = link_tags(html)
            .into_iter()
            .filter(|t| attr(t, "rel") == Some("stylesheet"))
            .count();
        // garden/index.html is the house style; garden/<slug>/ adds one.
        let rel = p.strip_prefix(site()).unwrap();
        let themed = rel.starts_with("garden") && rel.components().count() == 3;
        assert_eq!(sheets, if themed { 2 } else { 1 }, "{}", p.display());
    }
}

#[test]
fn garden_styles_are_self_contained_and_readable() {
    for (slug, css) in themes() {
        assert!(
            !css.contains("@import") && !css.contains("url("),
            "{slug}.css loads something"
        );
        assert!(
            css.starts_with("/*"),
            "{slug}.css should open with a comment saying what it is"
        );
        let longest = css.lines().map(str::len).max().unwrap_or(0);
        assert!(longest <= 200, "{slug}.css line of {longest} characters");
    }
}
