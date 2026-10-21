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

// Fallback faces are sized to Atkinson so the swap doesn't shift text, and
// come straight after it in the stack, before the plain system fonts
// (ADR 2026-10-20-metric-matched-fallback-fonts).
#[test]
fn fallback_fonts_are_metric_matched() {
    let css = css();
    let stack = css
        .split("--font-sans:")
        .nth(1)
        .and_then(|rest| rest.split(';').next())
        .expect("--font-sans defined");
    let families: Vec<&str> = stack.split(',').map(|f| f.trim().trim_matches('"')).collect();
    let fallbacks = ["Atkinson Fallback Verdana", "Atkinson Fallback Arial"];
    assert_eq!(&families[2..4], &fallbacks, "--font-sans: {stack}");
    for family in fallbacks {
        let faces: Vec<&str> = css
            .split("@font-face")
            .skip(1)
            .map(|rest| rest.split('}').next().unwrap_or(""))
            .filter(|face| face.contains(&format!("font-family: \"{family}\"")))
            .collect();
        assert!(!faces.is_empty(), "no @font-face for {family}");
        for face in faces {
            assert!(face.contains("src: local("), "{family} must use local fonts only");
            assert!(!face.contains("url("), "{family} must not download anything");
            for prop in [
                "size-adjust:",
                "ascent-override:",
                "descent-override:",
                "line-gap-override:",
            ] {
                assert!(face.contains(prop), "{family} is missing {prop}");
            }
        }
    }
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
    // Overscroll on phones paints from the root's own background.
    let root = rule_after(&css, 0, "html").expect("html rule");
    assert!(
        root.contains("background: var(--surface-page);"),
        "html lacks the page background"
    );
    for (p, html) in html_files() {
        for (scheme, colour) in [("light", "#fbfaf7"), ("dark", "#15171b")] {
            let tag =
                format!("<meta name=\"theme-color\" media=\"(prefers-color-scheme: {scheme})\" content=\"{colour}\">");
            assert!(html.contains(&tag), "{} lacks {scheme} theme-color", p.display());
        }
    }
}

/// The style picker's extra styles: every `build/static/themes/*.css`, by slug.
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
    assert!(!out.is_empty(), "expected styles in build/static/themes");
    out
}

#[test]
fn styles_load_only_when_picked() {
    // ADR 2026-10-20-zen-garden-styles: the only stylesheet a page loads up
    // front is the house one. The themes are linked inside <noscript>, for
    // the picker to work with JavaScript off; with it on, style-picker.js
    // adds just the one picked.
    let mut expected = Vec::new();
    for (slug, source) in themes() {
        let css = fs::read_to_string(site().join(format!("themes/{slug}.css"))).expect("read theme");
        assert_eq!(css, source, "themes/{slug}.css differs from its source");
        expected.push(format!("/themes/{slug}.css"));
    }
    expected.sort();
    for (p, html) in html_files().iter() {
        let head = &html[..html.find("</head>").expect("no </head>")];
        let (before, noscript) = match head.split_once("<noscript>\n") {
            Some((b, n)) => (b, &n[..n.find("</noscript>").expect("unclosed <noscript>")]),
            None => panic!("{}: no <noscript> with the styles", p.display()),
        };
        let sheets = |part: &str| -> Vec<String> {
            link_tags(part)
                .into_iter()
                .filter(|t| attr(t, "rel") == Some("stylesheet"))
                .filter_map(|t| attr(t, "href"))
                .map(String::from)
                .collect()
        };
        assert_eq!(sheets(before), ["/style.css"], "{}", p.display());
        let mut linked = sheets(noscript);
        linked.sort();
        assert_eq!(linked, expected, "{}", p.display());
        assert!(
            !html[html.find("</head>").unwrap()..].contains("rel=\"stylesheet\""),
            "{}: a stylesheet outside <head>",
            p.display()
        );
    }
}

#[test]
fn seasonal_styles_exist() {
    // style-picker.js picks Diablo, Doctor Who, Holiday or a throwback by
    // date for readers who never picked a style (ADR
    // 2026-10-20-zen-garden-styles, ADR 2026-10-21-doctor-who-style).
    let js = fs::read_to_string(crate::support::repo().join("build/static/style-picker.js")).expect("read script");
    let slugs: Vec<String> = themes().into_iter().map(|(s, _)| s).collect();
    for seasonal in ["diablo", "doctor-who", "holiday"] {
        assert!(
            js.contains(&format!("return \"{seasonal}\"")),
            "script lost the {seasonal} season"
        );
        assert!(slugs.iter().any(|s| s == seasonal), "no themes/{seasonal}.css");
    }
    // On April Fools' Day it picks from the menu's Throwbacks, all of them.
    let (_, html) = html_files().first().expect("a page");
    let group = &html[html
        .find("<optgroup label=\"Throwbacks\">")
        .expect("no Throwbacks group")..];
    let group = &group[..group.find("</optgroup>").unwrap()];
    let menu: Vec<String> = group
        .match_indices("<option ")
        .filter_map(|(i, _)| attr(&group[i..], "value"))
        .map(|v| format!("\"{v}\""))
        .collect();
    let line = js
        .lines()
        .find(|l| l.contains("var throwbacks = ["))
        .expect("no throwbacks list");
    let list = &line[line.find('[').unwrap() + 1..line.find(']').unwrap()];
    let script: Vec<String> = list.split(", ").map(String::from).collect();
    assert_eq!(script, menu, "style-picker.js throwbacks differ from the menu");
}

/// Relative luminance of a `#rrggbb` colour, per WCAG 2.
fn luminance(hex: &str) -> f64 {
    let channel = |i: usize| {
        let c = f64::from(u8::from_str_radix(&hex[i..i + 2], 16).expect("hex colour")) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5)
}

fn contrast(a: &str, b: &str) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[test]
fn styles_keep_aaa_contrast() {
    // Every style's text and link colours meet WCAG AAA 7:1 on each surface
    // (ADR 2026-10-18-style-picker), the selected surface with body text.
    for (slug, css) in themes() {
        let token = |name: &str| -> String {
            let decl = format!("--{name}: #");
            let i = css.find(&decl).unwrap_or_else(|| panic!("{slug}.css: no hex --{name}"));
            css[i + decl.len() - 1..i + decl.len() + 6].to_string()
        };
        let mut pairs = vec![("text-1", "surface-selected")];
        for fg in ["text-1", "text-2", "text-link", "text-link-hover"] {
            for bg in ["surface-page", "surface-card", "surface-sunken"] {
                pairs.push((fg, bg));
            }
        }
        for (fg, bg) in pairs {
            let ratio = contrast(&token(fg), &token(bg));
            assert!(ratio >= 7.0, "{slug}.css: --{fg} on --{bg} is {ratio:.2}:1, under 7:1");
        }
    }
}

#[test]
fn every_page_has_the_style_picker() {
    let mut options = vec!["auto".to_string(), "light".into(), "dark".into()];
    for (slug, _) in themes() {
        options.push(slug);
    }
    for (p, html) in html_files().iter() {
        if html.contains("http-equiv=\"refresh\"") {
            continue;
        }
        let start = html
            .find("<select id=\"style\">")
            .unwrap_or_else(|| panic!("{}: no picker", p.display()));
        let select = &html[start..start + html[start..].find("</select>").unwrap()];
        let values: Vec<&str> = select
            .match_indices("<option ")
            .filter_map(|(i, _)| attr(&select[i..], "value"))
            .collect();
        let mut sorted_options: Vec<&str> = options[3..].iter().map(String::as_str).collect();
        sorted_options.sort();
        assert_eq!(&values[..3], ["auto", "light", "dark"], "{}", p.display());
        let mut rest = values[3..].to_vec();
        rest.sort();
        assert_eq!(rest, sorted_options, "{}", p.display());
        assert!(
            select.contains("<option value=\"auto\" selected>"),
            "{}: Auto not the default",
            p.display()
        );
        let script = "</select>\n</label>\n<script src=\"/style-picker.js\"></script>";
        assert!(
            html.contains(script),
            "{}: script not right after the picker",
            p.display()
        );
    }
}

#[test]
fn styles_apply_only_while_picked() {
    for (slug, css) in themes() {
        let scope = format!(":root:has(#style [value=\"{slug}\"]:checked) {{");
        let body = css.find(&scope).unwrap_or_else(|| panic!("{slug}.css: no {scope}"));
        // Everything after the opening comment is the scoped block, plus
        // @keyframes, which can't be nested.
        let start = css.find("*/").unwrap() + 2;
        assert_eq!(css[start..body].trim(), "", "{slug}.css: rules before the scope");
        let mut depth = 0;
        let mut end = body;
        for (i, c) in css[body..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = body + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        let rest = css[end..].trim();
        assert!(
            rest.is_empty() || rest.starts_with("@keyframes"),
            "{slug}.css: rules outside the scope"
        );
    }
}

#[test]
fn styles_are_self_contained_and_readable() {
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

#[test]
fn pinned_dark_matches_the_dark_scheme() {
    let css = css();
    let (_, dark) = light_and_dark(&css);
    let pinned = rule_after(&css, 0, ":root:has(#style [value=\"dark\"]:checked)").expect("no pinned dark rule");
    let tokens = |block: &str| -> Vec<String> {
        block
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("--"))
            .map(String::from)
            .collect()
    };
    assert!(!tokens(dark).is_empty());
    assert_eq!(tokens(dark), tokens(pinned), "pinned Dark differs from the dark scheme");
}
