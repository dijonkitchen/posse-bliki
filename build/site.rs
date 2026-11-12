//! The build pipeline: load notes, validate, resolve links, render pages,
//! feeds, sitemap and redirects. Templates are plain Rust functions whose
//! output matches the former Jinja templates byte for byte (autoescape on).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::blogroll;
use crate::date::{self, Date, Stamp};
use crate::frontmatter::{self, FrontMatter, LoadError, Value};
use crate::image;
use crate::indent;
use crate::json::{self, Json};
use crate::markdown;
use crate::text;
use crate::util::{escape as e, percent_decode, py_repr_str};
use crate::webmention::{self, Mention};

const STYLE_CSS: &str = include_str!("static/style.css");

/// The styles in every page's style picker besides Auto, Light and Dark:
/// (slug, name, menu group, stylesheet), in menu order. See ADR
/// 2026-10-18-style-picker, ADR 2026-10-20-zen-garden-styles, ADR
/// 2026-10-21-doctor-who-style, ADR 2026-10-22-star-trek-style and ADR
/// 2026-10-22-ft-solarized-and-editor-styles.
const THEMES: &[(&str, &str, &str, &str)] = &[
    (
        "terminal",
        "Terminal",
        "Originals",
        include_str!("static/themes/terminal.css"),
    ),
    (
        "broadsheet",
        "Broadsheet",
        "Originals",
        include_str!("static/themes/broadsheet.css"),
    ),
    (
        "solarized-light",
        "Solarized Light",
        "Editors",
        include_str!("static/themes/solarized-light.css"),
    ),
    (
        "editor-dark",
        "Editor Dark",
        "Editors",
        include_str!("static/themes/editor-dark.css"),
    ),
    (
        "tranquille",
        "Tranquille",
        "CSS Zen Garden",
        include_str!("static/themes/tranquille.css"),
    ),
    (
        "mid-century",
        "Mid Century",
        "CSS Zen Garden",
        include_str!("static/themes/mid-century.css"),
    ),
    (
        "steel",
        "Steel",
        "CSS Zen Garden",
        include_str!("static/themes/steel.css"),
    ),
    (
        "apothecary",
        "Apothecary",
        "CSS Zen Garden",
        include_str!("static/themes/apothecary.css"),
    ),
    ("diablo", "Diablo", "Seasonal", include_str!("static/themes/diablo.css")),
    (
        "holiday",
        "Holiday",
        "Seasonal",
        include_str!("static/themes/holiday.css"),
    ),
    (
        "doctor-who",
        "Doctor Who",
        "Seasonal",
        include_str!("static/themes/doctor-who.css"),
    ),
    (
        "star-trek",
        "Star Trek",
        "Seasonal",
        include_str!("static/themes/star-trek.css"),
    ),
    (
        "geocities",
        "GeoCities",
        "Throwbacks",
        include_str!("static/themes/geocities.css"),
    ),
    (
        "angelfire",
        "Angelfire",
        "Throwbacks",
        include_str!("static/themes/angelfire.css"),
    ),
    ("xanga", "Xanga", "Throwbacks", include_str!("static/themes/xanga.css")),
    (
        "jekyll",
        "Jekyll",
        "Throwbacks",
        include_str!("static/themes/jekyll.css"),
    ),
    (
        "facebook",
        "Facebook",
        "Throwbacks",
        include_str!("static/themes/facebook.css"),
    ),
];
/// Loads the picked style and remembers it across pages; the picker works
/// without it on the current page (ADR 2026-10-18-style-picker,
/// ADR 2026-10-20-zen-garden-styles).
const STYLE_PICKER_JS: &str = include_str!("static/style-picker.js");
/// Keeps pages as they're read for reading offline; style-picker.js starts
/// it (ADR 2026-10-29-offline-reading).
const SERVICE_WORKER_JS: &str = include_str!("static/sw.js");
/// Site icons, after the spork on JC's Substack (spec/performance.md):
/// `favicon.ico` (32px) for old browsers and clients that request it
/// unprompted, `icon.svg` for the tab (redrawn solid so it reads at 16px,
/// with a dark-mode colour), and the original drawing at 180px for iOS and
/// 192/512px for the web manifest. The manifest's paths are relative, so
/// they work under a sub-path.
const ICONS: &[(&str, &[u8])] = &[
    ("favicon.ico", include_bytes!("static/icons/favicon.ico")),
    ("icon.svg", include_bytes!("static/icons/icon.svg")),
    (
        "apple-touch-icon.png",
        include_bytes!("static/icons/apple-touch-icon.png"),
    ),
    ("icon-192.png", include_bytes!("static/icons/icon-192.png")),
    ("icon-512.png", include_bytes!("static/icons/icon-512.png")),
    (
        "manifest.webmanifest",
        include_bytes!("static/icons/manifest.webmanifest"),
    ),
];

/// Longest `<meta name="description">` written from a page's own text;
/// search results show about this much (spec/performance.md).
const DESCRIPTION_MAX: usize = 160;
/// Longest RSS `<description>` cut from a post's text (spec/feeds.md).
const RSS_DESCRIPTION_MAX: usize = 200;
/// Self-hosted fonts and their licences (spec/design.md), copied to `fonts/`.
const FONTS: &[(&str, &[u8])] = &[
    (
        "atkinson-hyperlegible-next-latin.woff2",
        include_bytes!("static/fonts/atkinson-hyperlegible-next-latin.woff2"),
    ),
    (
        "atkinson-hyperlegible-next-italic-latin.woff2",
        include_bytes!("static/fonts/atkinson-hyperlegible-next-italic-latin.woff2"),
    ),
    (
        "jetbrains-mono-latin.woff2",
        include_bytes!("static/fonts/jetbrains-mono-latin.woff2"),
    ),
    (
        "OFL-atkinson-hyperlegible-next.txt",
        include_bytes!("static/fonts/OFL-atkinson-hyperlegible-next.txt"),
    ),
    (
        "OFL-jetbrains-mono.txt",
        include_bytes!("static/fonts/OFL-jetbrains-mono.txt"),
    ),
];

/// Site configuration (edit before going live).
pub struct Config {
    pub title: &'static str,
    pub tagline: &'static str,
    pub base_url: &'static str,
    pub language: &'static str,
    /// The full name; other sites see it after the nickname
    /// (ADR 2026-11-06-profile-name).
    pub author_name: &'static str,
    /// Short name readers see in bylines, "JC"; the full name stays in the markup (ADR 2026-10-16-site-title-heads-home).
    pub author_nickname: &'static str,
    /// A one-line bio for the home h-card's `p-note`, in JC's own words
    /// from the bio page (ADR 2026-11-06-h-card-bio-note).
    pub author_note: &'static str,
    /// The home h-card's `u-photo`, a path under `content/images/`
    /// (ADR 2026-11-06-h-card-photo).
    pub author_photo: &'static str,
    /// The part of the author's email address before the `@`; the domain is
    /// the site's. Never written out whole in the repo, and only published
    /// in HTML character codes (ADR 2026-11-06-coded-email).
    pub author_email_user: &'static str,
    pub author_url: &'static str,
    pub rel_me: &'static [&'static str],
    pub webmention_endpoint: &'static str,
    /// POSSE targets each post links to and is sent to as a webmention
    /// once deployed: `(target, webmention endpoint)`.
    pub syndicate_to: &'static [(&'static str, &'static str)],
    /// Received webmentions keyed by `webmention::url_key` of their target;
    /// empty unless `--webmentions` is given.
    pub mentions: HashMap<String, Vec<Mention>>,
    /// Notes dated after this day are held back until it arrives
    /// (ADR 2026-10-20-scheduled-posts); `--today` overrides the UTC clock.
    pub today: Date,
}

pub fn default_config() -> Config {
    Config {
        title: "Dijon Kitchen",
        tagline: "JC's napkin logic bliki. Philosophy as a service.",
        base_url: "https://www.dijonkitchen.org",
        language: "en",
        author_name: "Jonathan Chen",
        author_nickname: "JC",
        author_note: "Full-stack, entrepreneurial engineer that has built businesses from zero to one.",
        author_photo: "/images/jc.jpg",
        author_email_user: "jc",
        author_url: "https://www.dijonkitchen.org",
        rel_me: &["https://github.com/dijonkitchen"],
        webmention_endpoint: "https://webmention.io/www.dijonkitchen.org/webmention",
        syndicate_to: &[
            ("https://brid.gy/publish/mastodon", "https://brid.gy/publish/webmention"),
            ("https://brid.gy/publish/bluesky", "https://brid.gy/publish/webmention"),
            ("https://fed.brid.gy/", "https://fed.brid.gy/webmention"),
        ],
        mentions: HashMap::new(),
        today: Date::today_utc(),
    }
}

impl Config {
    pub fn to_json(&self) -> String {
        json::dumps(&Json::Obj(vec![
            ("title".into(), json::s(self.title)),
            ("tagline".into(), json::s(self.tagline)),
            ("base_url".into(), json::s(self.base_url)),
            ("language".into(), json::s(self.language)),
            (
                "author".into(),
                Json::Obj(vec![
                    ("name".into(), json::s(self.author_name)),
                    ("nickname".into(), json::s(self.author_nickname)),
                    ("note".into(), json::s(self.author_note)),
                    ("photo".into(), json::s(self.author_photo)),
                    ("email_user".into(), json::s(self.author_email_user)),
                    ("url".into(), json::s(self.author_url)),
                    (
                        "rel_me".into(),
                        Json::Arr(self.rel_me.iter().map(|u| json::s(u)).collect()),
                    ),
                ]),
            ),
            (
                "webmention".into(),
                Json::Obj(vec![("endpoint".into(), json::s(self.webmention_endpoint))]),
            ),
            (
                "syndicate_to".into(),
                Json::Arr(
                    self.syndicate_to
                        .iter()
                        .map(|(t, ep)| Json::Obj(vec![("target".into(), json::s(t)), ("endpoint".into(), json::s(ep))]))
                        .collect(),
                ),
            ),
        ]))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Home,
    Page,
    Post,
}

struct Note {
    rel: String,
    slug: String,
    kind: Kind,
    fm: FrontMatter,
    body: String,
    url: String,
    out_path: PathBuf,
    content_html: String,
    backlinks: Vec<usize>,
    date: Option<Date>,
    updated_at: Option<Stamp>,
}

impl Note {
    fn title(&self) -> &str {
        frontmatter::get(&self.fm, "title")
            .and_then(Value::as_str)
            .unwrap_or("")
    }

    /// `summary` when truthy.
    fn summary(&self) -> Option<&str> {
        frontmatter::get(&self.fm, "summary")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
    }

    fn str_list(&self, key: &str) -> Vec<String> {
        match frontmatter::get(&self.fm, key) {
            Some(Value::List(l)) => l.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            _ => Vec::new(),
        }
    }

    fn draft(&self) -> bool {
        frontmatter::get(&self.fm, "draft").is_some_and(Value::truthy)
    }

    fn post_date(&self) -> Result<Date, String> {
        self.date.ok_or_else(|| {
            format!(
                "{}: note has no `date` (required for dt-published); give it `updated_at` instead to make it an evergreen note",
                self.rel
            )
        })
    }

    /// A note in `notes/` with `updated_at` but no `date`: listed with the
    /// other notes but never posted to the feeds (ADR 2026-11-07-evergreen-notes).
    fn evergreen(&self) -> bool {
        self.kind == Kind::Post && self.date.is_none() && self.updated_at.is_some()
    }

    /// The day a note goes live: its `date`, or the UTC day of an evergreen
    /// note's `updated_at`.
    fn goes_live(&self) -> Option<Date> {
        if self.evergreen() {
            self.updated_at.map(|u| u.day)
        } else {
            self.date
        }
    }

    /// The update readers are told about. `date` is a day in no time zone,
    /// so it isn't over everywhere until noon UTC the day after, and a save
    /// before then is part of publishing the note, not an update to it
    /// (ADR 2026-11-12-updated-at-utc-datetimes).
    fn update(&self) -> Option<Stamp> {
        let u = self.updated_at?;
        match self.date {
            Some(d) if u.seconds() < d.noon_utc() + date::DAY => None,
            _ => Some(u),
        }
    }

    /// When the note was last tended, in seconds since 1970: its update,
    /// or else its `date`.
    fn tended(&self) -> Option<i64> {
        self.update().map(|u| u.seconds()).or(self.date.map(|d| d.noon_utc()))
    }

    /// The day the page last changed, for the sitemap: the later of `date`
    /// and the UTC day of `updated_at`.
    fn last_modified(&self) -> Option<Date> {
        match (self.date, self.updated_at.map(|u| u.day)) {
            (Some(d), Some(u)) => Some(d.max(u)),
            (d, u) => d.or(u),
        }
    }
}

/// Recursively collect `*.md` files, sorted like Python's `sorted(Path.rglob(...))`.
fn collect_md(dir: &Path, rel: &mut Vec<String>, out: &mut Vec<Vec<String>>) -> Result<(), String> {
    let rd = fs::read_dir(dir).map_err(|err| format!("cannot read {}: {}", dir.display(), err))?;
    for entry in rd {
        let entry = entry.map_err(|err| format!("cannot read {}: {}", dir.display(), err))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let ft = entry.file_type().map_err(|err| err.to_string())?;
        rel.push(name.clone());
        if ft.is_dir() {
            collect_md(&entry.path(), rel, out)?;
        } else if name.ends_with(".md") && !entry.path().is_dir() {
            out.push(rel.clone());
        }
        rel.pop();
    }
    Ok(())
}

fn kind_for(parts: &[String]) -> Kind {
    if parts.len() == 1 && parts[0] == "index.md" {
        Kind::Home
    } else if parts[0] == "notes" {
        Kind::Post
    } else {
        Kind::Page
    }
}

fn stem(name: &str) -> &str {
    // Path.stem: strip the last suffix (a leading dot does not start a suffix)
    match name.rfind('.') {
        Some(i) if i > 0 => &name[..i],
        _ => name,
    }
}

fn url_for(parts: &[String], kind: Kind) -> String {
    match kind {
        Kind::Home => "/".into(),
        Kind::Page => format!("/{}/", text::slugify(stem(parts.last().unwrap()))),
        Kind::Post => {
            let mut p: Vec<String> = parts[..parts.len() - 1].to_vec();
            p.push(text::slugify(stem(parts.last().unwrap())));
            format!("/{}/", p.join("/"))
        }
    }
}

fn out_path_for(out: &Path, url: &str) -> PathBuf {
    if url == "/" {
        out.join("index.html")
    } else {
        out.join(url.trim_matches('/')).join("index.html")
    }
}

fn load_notes(content: &Path) -> Result<Vec<Note>, String> {
    let mut files = Vec::new();
    collect_md(content, &mut Vec::new(), &mut files)?;
    files.sort();
    let mut notes = Vec::new();
    for parts in files {
        if parts.iter().any(|p| p.starts_with('.')) {
            continue;
        }
        let rel = parts.join("/");
        let path = parts.iter().fold(content.to_path_buf(), |acc, p| acc.join(p));
        let bytes = fs::read(&path).map_err(|err| format!("{}: {}", rel, err))?;
        let textv = String::from_utf8(bytes).map_err(|_| format!("{}: file is not valid UTF-8", rel))?;
        let (fm, body) = match frontmatter::load(&textv) {
            Ok(x) => x,
            Err(LoadError::NotMapping) => return Err(format!("{}: front-matter is not a YAML mapping", rel)),
            Err(LoadError::Yaml(m)) => return Err(format!("{}: {}", rel, m)),
        };
        // Said before the schema check, whose own words for these two are
        // an unknown field and a pattern (ADR 2026-11-12-updated-at-utc-datetimes).
        if frontmatter::get(&fm, "updated").is_some() {
            return Err(format!("{}: `updated` is now `updated_at`; rename the field", rel));
        }
        let updated_at = match frontmatter::get(&fm, "updated_at").and_then(Value::as_str) {
            None | Some("") => None,
            Some(s) => Some(Stamp::parse(s).ok_or_else(|| {
                format!(
                    "{}: `updated_at` must be a day (2026-11-12) or a UTC datetime ending in Z (2026-11-12T04:27:28Z), not {}",
                    rel,
                    py_repr_str(s)
                )
            })?),
        };
        if let Err(msg) = frontmatter::validate(&fm) {
            return Err(format!("{}: front-matter invalid: {}", rel, msg));
        }
        let kind = kind_for(&parts);
        let slug = if kind == Kind::Home {
            "index".to_string()
        } else {
            text::slugify(stem(parts.last().unwrap()))
        };
        let parse_date = |key: &str| -> Result<Option<Date>, String> {
            match frontmatter::get(&fm, key).and_then(Value::as_str) {
                None | Some("") => Ok(None),
                Some(s) => Date::parse(s)
                    .map(Some)
                    .ok_or_else(|| format!("{}: invalid {} {}", rel, key, py_repr_str(s))),
            }
        };
        let date = parse_date("date")?;
        let url = url_for(&parts, kind);
        notes.push(Note {
            rel,
            slug,
            kind,
            body: body.to_string(),
            fm,
            url,
            out_path: PathBuf::new(),
            content_html: String::new(),
            backlinks: Vec::new(),
            date,
            updated_at,
        });
    }
    Ok(notes)
}

fn write(p: &Path, content: &str) -> Result<(), String> {
    write_bytes(p, content.as_bytes())
}

fn write_bytes(p: &Path, content: &[u8]) -> Result<(), String> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("cannot create {}: {}", parent.display(), err))?;
    }
    fs::write(p, content).map_err(|err| format!("cannot write {}: {}", p.display(), err))
}

/// The path part of `base_url`: `/posse-bliki` for a GitHub Pages project
/// site, empty when the site is served from a domain root.
fn base_path(base_url: &str) -> String {
    let rest = base_url.split_once("://").map_or(base_url, |(_, r)| r);
    rest.find('/')
        .map_or("", |i| &rest[i..])
        .trim_end_matches('/')
        .to_string()
}

/// Root-relative URL openers the pages and feeds emit: attributes in HTML,
/// the same escaped inside RSS (`&quot;`) and JSON Feed (`\"`) bodies, and
/// the meta-refresh target of alias redirects.
const ROOT_RELATIVE: &[&str] = &[
    "href=\"/",
    "src=\"/",
    "href=&quot;/",
    "src=&quot;/",
    "href=\\\"/",
    "src=\\\"/",
    "url=/",
];

/// Prefix every root-relative link with `prefix` (`/notes/` becomes
/// `/posse-bliki/notes/`). Protocol-relative `//host` links are left alone.
fn prefix_root_relative(text: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let rest = &text[i..];
        match ROOT_RELATIVE.iter().find(|p| rest.starts_with(**p)) {
            Some(p) if !rest[p.len()..].starts_with('/') => {
                out.push_str(&p[..p.len() - 1]);
                out.push_str(prefix);
                out.push('/');
                i += p.len();
            }
            _ => {
                let c = rest.chars().next().unwrap();
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
    out
}

/// Rewrite the emitted site so it works when served under a sub-path.
fn apply_base_path(dir: &Path, prefix: &str) -> Result<(), String> {
    if prefix.is_empty() {
        return Ok(());
    }
    let rd = fs::read_dir(dir).map_err(|err| format!("cannot read {}: {}", dir.display(), err))?;
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            apply_base_path(&p, prefix)?;
        } else if matches!(p.extension().and_then(|x| x.to_str()), Some("html" | "xml" | "json")) {
            let text = fs::read_to_string(&p).map_err(|err| format!("cannot read {}: {}", p.display(), err))?;
            write(&p, &prefix_root_relative(&text, prefix))?;
        }
    }
    Ok(())
}

// --- templates ---

/// Where to report a security problem: GitHub's private vulnerability
/// reporting for this repository (ADR 2026-10-28-security-txt).
/// robots.txt tokens whose operators say what they gather may train AI
/// models (ADR 2026-11-06-no-ai-training). The same companies' search
/// agents (Googlebot, Applebot, OAI-SearchBot, Claude-SearchBot,
/// Amzn-SearchBot) stay allowed.
const AI_TRAINING_AGENTS: &[&str] = &[
    "GPTBot",
    "ClaudeBot",
    "Google-Extended",
    "Applebot-Extended",
    "meta-externalagent",
    "Amazonbot",
    "CCBot",
];

const SECURITY_CONTACT: &str = "https://github.com/dijonkitchen/posse-bliki/security/advisories/new";
/// How far ahead security.txt expires; RFC 9116 asks for under a year.
const SECURITY_TXT_DAYS: i64 = 180;

/// `content/blogroll.md`, whose list also ships as OPML (ADR 2026-10-25-blogroll).
const BLOGROLL: &str = "blogroll";
const BLOGROLL_OPML: &str = "/blogroll.opml";

/// `content/now.md`, a now page after Derek Sivers's (ADR 2026-11-01-now-page).
const NOW: &str = "now";

/// `content/bio.md`, the work bio, which opens with the h-card's photo
/// (ADR 2026-11-06-h-card-photo).
const BIO: &str = "bio";

/// `content/follow.md`, how to follow the site in a feed reader
/// (ADR 2026-11-05-follow-with-a-feed-reader).
const FOLLOW: &str = "follow";

/// The `/tags/<tag>/` pages built before tags were dropped, and where each
/// now redirects (ADR 2026-11-06-drop-tags).
const OLD_TAG_PAGES: &[(&str, &str)] = &[("meta", "/colophon/"), ("podcast", "/notes/")];

/// The optional pages every page links to, when the vault has them.
#[derive(Clone, Copy)]
struct FooterLinks {
    blogroll: bool,
    now: bool,
    follow: bool,
}

fn footer_links(notes: &[Note]) -> FooterLinks {
    let has = |slug: &str| notes.iter().any(|n| n.slug == slug && n.kind == Kind::Page);
    FooterLinks {
        blogroll: has(BLOGROLL),
        now: has(NOW),
        follow: has(FOLLOW),
    }
}

fn base(site: &Config, links: FooterLinks, title: &str, meta: &str, canonical: Option<&str>, content: &str) -> String {
    page(site, links, title, meta, canonical, content, false)
}

/// `home` makes the site title the page's `<h1>` (ADR 2026-10-16-site-title-heads-home).
/// `links` adds the header link to `/now/` and the footer links to `/follow/`
/// and `/blogroll/` when the vault has them (ADR 2026-11-05-now-in-the-header,
/// ADR 2026-11-05-follow-with-a-feed-reader).
fn page(
    site: &Config,
    links: FooterLinks,
    title: &str,
    meta: &str,
    canonical: Option<&str>,
    content: &str,
    home: bool,
) -> String {
    let t = e(site.title);
    let brand = if home {
        format!("<h1 class=\"site-title\"><a href=\"/\" aria-current=\"page\">{t}</a></h1>")
    } else {
        format!("<a class=\"site-title\" href=\"/\">{t}</a>")
    };
    let canon = canonical
        .map(|c| format!("<link rel=\"canonical\" href=\"{}\">", e(c)))
        .unwrap_or_default();
    // Profiles stay machine-readable without showing on the page (ADR 2026-10-16-site-title-heads-home).
    let rel_me: String = if home {
        site.rel_me
            .iter()
            .map(|u| format!("<link rel=\"me\" href=\"{}\">\n", e(u)))
            .chain(std::iter::once(format!(
                "<link rel=\"me\" href=\"{}\">\n",
                coded_mailto(site)
            )))
            .collect()
    } else {
        String::new()
    };
    // The home page's representative h-card, for IndieWeb tools (ADR 2026-10-16-home-shows-recent-posts).
    let card = if home {
        format!(
            "<span class=\"h-card\">By <a class=\"u-url u-uid\" rel=\"me\" href=\"{}/\">{}</a><data class=\"p-note\" value=\"{}\"></data><data class=\"u-photo\" value=\"{}{}\"></data></span> \u{b7}\n",
            e(site.base_url),
            author_label(site),
            e(site.author_note),
            e(site.base_url),
            e(site.author_photo)
        )
    } else {
        String::new()
    };
    // Theme stylesheets load only when picked: style-picker.js adds the one
    // in use, and with JavaScript off the <noscript> copy links them all so
    // the picker still works through :has() (ADR 2026-10-20-zen-garden-styles).
    let theme_links: String = THEMES
        .iter()
        .map(|(slug, ..)| format!("<link rel=\"stylesheet\" href=\"/themes/{slug}.css\">\n"))
        .collect();
    let now = if links.now { "<a href=\"/now/\">Now</a>\n" } else { "" };
    let follow = if links.follow {
        "<a href=\"/follow/\">Follow</a> \u{b7}\n"
    } else {
        ""
    };
    let blogroll = if links.blogroll {
        "<a href=\"/blogroll/\">Blogroll</a> \u{b7}\n"
    } else {
        ""
    };
    // style-picker.js runs in <head>, before the picker exists, so it gets
    // the menu's styles from its own tag (ADR 2026-10-30-style-before-first-paint).
    let theme_slugs = THEMES.iter().map(|(slug, ..)| *slug).collect::<Vec<_>>().join(" ");
    let mut theme_options = String::new();
    for (i, (slug, name, group, _)) in THEMES.iter().enumerate() {
        if i == 0 || THEMES[i - 1].2 != *group {
            if i > 0 {
                theme_options.push_str("</optgroup>\n");
            }
            theme_options.push_str(&format!("<optgroup label=\"{group}\">\n"));
        }
        theme_options.push_str(&format!("<option value=\"{slug}\">{name}</option>\n"));
    }
    theme_options.push_str("</optgroup>\n");
    let html = format!(
        "<!DOCTYPE html>\n<html lang=\"{lang}\">\n<head>\n<meta charset=\"utf-8\">\n{CSP}\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n{THEME_COLOR}<title>{title}</title>\n{meta}\n{canon}\n\
<link rel=\"alternate\" type=\"application/rss+xml\" title=\"{t}\" href=\"/index.xml\">\n\
<link rel=\"alternate\" type=\"application/feed+json\" title=\"{t}\" href=\"/feed.json\">\n\
<link rel=\"webmention\" href=\"{wm}\">\n{rel_me}\
<link rel=\"icon\" href=\"/favicon.ico\" sizes=\"32x32\">\n\
<link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\">\n\
<link rel=\"apple-touch-icon\" href=\"/apple-touch-icon.png\">\n\
<link rel=\"manifest\" href=\"/manifest.webmanifest\">\n\
<link rel=\"preload\" href=\"/fonts/atkinson-hyperlegible-next-latin.woff2\" as=\"font\" type=\"font/woff2\" crossorigin>\n\
<link rel=\"stylesheet\" href=\"/style.css\">\n<noscript>\n{theme_links}</noscript>\n\
<script src=\"/style-picker.js\" data-styles=\"{theme_slugs}\"></script>\n</head>\n<body>\n<a class=\"skip-link\" href=\"#main\">Skip to content</a>\n\
<header class=\"site-header\">\n\
<div class=\"site-brand\">\n{brand}\n<p class=\"site-tagline\">{tagline}</p>\n</div>\n<nav aria-label=\"Site\">\n<a href=\"/notes/\">Notes</a>\n{now}<a href=\"/about/\">About</a>\n\
</nav>\n<label class=\"style-picker\">Style\n<select id=\"style\">\n<option value=\"auto\" selected>Auto</option>\n\
<option value=\"light\">Light</option>\n<option value=\"dark\">Dark</option>\n{theme_options}</select>\n</label>\n</header>\n<main id=\"main\">\n{content}\n</main>\n<footer class=\"site-footer\">\n<p>\n{card}{follow}\
<a href=\"/index.xml\">RSS</a> \u{b7}\n<a href=\"/feed.json\">JSON Feed</a> \u{b7}\n{blogroll}<a href=\"/colophon/\">Colophon</a>\n\
</p>\n{WEBRING}</footer>\n</body>\n</html>\n",
        lang = e(site.language),
        tagline = e(site.tagline),
        wm = e(site.webmention_endpoint),
    );
    indent::indent(&html)
}

fn description_meta(text: &str) -> String {
    format!("<meta name=\"description\" content=\"{}\">", e(text))
}

/// A note's description: its `summary`, else the start of its text, else
/// the site tagline (spec/performance.md: every indexable page has one).
fn summary_meta(site: &Config, n: &Note) -> String {
    let text = match n.summary() {
        Some(s) => s.to_string(),
        None => text::excerpt(&n.content_html, DESCRIPTION_MAX),
    };
    description_meta(if text.is_empty() { site.tagline } else { &text })
}

/// Prepare a note's images and embeds (spec/performance.md): the first may be the
/// largest thing on the first screen so it loads eagerly, the rest load
/// lazily, all decode off the main thread, and each self-hosted image
/// (`/images/...`, from `content/images/`) gets its `width` and `height` so
/// the page doesn't jump when it arrives. A missing self-hosted image is a
/// build error, like a broken link.
fn prepare_images(html: &str, content: &Path, rel: &str) -> Result<String, String> {
    let mut out = String::with_capacity(html.len());
    for (i, part) in html.split("<img ").enumerate() {
        if i > 0 {
            out.push_str("<img ");
            if i > 1 {
                out.push_str("loading=\"lazy\" ");
            }
            out.push_str("decoding=\"async\" ");
            let tag = &part[..part.find('>').unwrap_or(part.len())];
            let src = tag
                .split_once("src=\"")
                .and_then(|(_, r)| r.split_once('"'))
                .map(|(s, _)| s);
            if let Some(src) = src.filter(|s| s.starts_with("/images/")) {
                let file = content.join(percent_decode(src.trim_start_matches('/')));
                let bytes =
                    fs::read(&file).map_err(|_| format!("{}: image {} not found in content/images/", rel, src))?;
                let (w, h) =
                    image::size(&bytes).ok_or_else(|| format!("{}: cannot read the size of image {}", rel, src))?;
                out.push_str(&format!("width=\"{}\" height=\"{}\" ", w, h));
            }
        }
        out.push_str(part);
    }
    // Embeds (video players) are heavy and never the main content: always lazy.
    Ok(out.replace("<iframe ", "<iframe loading=\"lazy\" "))
}

/// Copy `content/images/` to `images/` in the output, as is.
fn copy_images(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Ok(());
    }
    let rd = fs::read_dir(from).map_err(|err| format!("cannot read {}: {}", from.display(), err))?;
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let dest = to.join(p.file_name().unwrap());
        if p.is_dir() {
            copy_images(&p, &dest)?;
        } else {
            let bytes = fs::read(&p).map_err(|err| format!("cannot read {}: {}", p.display(), err))?;
            write_bytes(&dest, &bytes)?;
        }
    }
    Ok(())
}

fn backlinks_block(notes: &[Note], n: &Note) -> String {
    if n.backlinks.is_empty() {
        return String::new();
    }
    let items: String = n
        .backlinks
        .iter()
        .map(|&b| format!("<li><a href=\"{}\">{}</a></li>", e(&notes[b].url), e(notes[b].title())))
        .collect();
    format!(
        "\n<aside class=\"backlinks\">\n<h2>Linked from</h2>\n<ul>\n{}\n</ul>\n</aside>\n",
        items
    )
}

fn render_post(site: &Config, notes: &[Note], n: &Note, canonical: &str) -> Result<String, String> {
    let links = footer_links(notes);
    let canon = e(canonical);
    // An evergreen note has no publish date, so its permalink carries the
    // date it was last updated (ADR 2026-11-07-evergreen-notes).
    let dates = match (n.evergreen(), n.updated_at) {
        (true, Some(u)) => format!(
            "<a class=\"u-url u-uid\" href=\"{canon}\"><time class=\"dt-updated\" datetime=\"{}\">Updated {}</time></a>\n",
            u.iso(),
            u.day.human()
        ),
        _ => {
            let d = n.post_date()?;
            let updated = n
                .update()
                .map(|u| {
                    format!(
                        " \u{b7} <time class=\"dt-updated\" datetime=\"{}\">updated {}</time>",
                        u.iso(),
                        u.day.human()
                    )
                })
                .unwrap_or_default();
            format!(
                "<a class=\"u-url u-uid\" href=\"{canon}\"><time class=\"dt-published\" datetime=\"{}\">{}</time></a>\n{updated}",
                d.iso(),
                d.human()
            )
        }
    };
    let summary = n
        .summary()
        .map(|s| format!("<p class=\"p-summary\">{}</p>", e(s)))
        .unwrap_or_default();
    let synd = n.str_list("syndication");
    let synd = if synd.is_empty() {
        String::new()
    } else {
        let links: Vec<String> = synd
            .iter()
            .map(|s| format!("<a class=\"u-syndication\" href=\"{}\">{}</a>", e(s), e(s)))
            .collect();
        format!("\n<footer>\n<p>Also on: {}</p>\n</footer>\n", links.join(", "))
    };
    // Bridgy only publishes a page that links to it; the links carry no text.
    let posse: String = site
        .syndicate_to
        .iter()
        .map(|(t, _)| format!("\n<a href=\"{}\" hidden></a>", e(t)))
        .collect();
    let mentions = site
        .mentions
        .get(&webmention::url_key(canonical))
        .map_or(String::new(), |m| webmention::render(m));
    let content = format!(
        "\n<article class=\"h-entry\">\n<header>\n<h1 class=\"p-name\">{title}</h1>\n<p class=\"meta\">\n\
{dates}\n \u{b7} <span class=\"p-author h-card\"><a class=\"u-url\" rel=\"author\" href=\"{base}/\">{author}</a></span>\n\
</p>\n{summary}\n</header>\n<div class=\"e-content\">\n{html}\n</div>\n{synd}{posse}{mentions}\n</article>\n{back}\n",
        title = e(n.title()),
        base = e(site.base_url),
        author = author_label(site),
        html = n.content_html,
        back = backlinks_block(notes, n),
    );
    Ok(base(
        site,
        links,
        &format!("{} \u{2014} {}", e(n.title()), e(site.title)),
        &summary_meta(site, n),
        Some(canonical),
        &content,
    ))
}

/// The IndieWeb Webring's previous/next links, from JC's webring dashboard.
/// The ring checks the home page for them; every footer has them so a
/// reader can hop on from any page (ADR 2026-11-06-indieweb-webring).
const WEBRING: &str = "<p class=\"webring\">\n\
<a href=\"https://xn--sr8hvo.ws/previous\" aria-label=\"Previous site in the IndieWeb Webring\">\u{2190}</a>\n\
An <a href=\"https://xn--sr8hvo.ws\">IndieWeb Webring</a> <span aria-hidden=\"true\">\u{1f578}\u{1f48d}</span>\n\
<a href=\"https://xn--sr8hvo.ws/next\" aria-label=\"Next site in the IndieWeb Webring\">\u{2192}</a>\n\
</p>\n";

/// Paints the phone's browser bars in the page colour, light and dark, so the
/// page reads full bleed (ADR 2026-10-16-browser-bars-match-page). These are
/// --surface-page as hex, since theme-color takes no CSS variables.
const THEME_COLOR: &str = "<meta name=\"theme-color\" media=\"(prefers-color-scheme: light)\" content=\"#fbfaf7\">\n\
<meta name=\"theme-color\" media=\"(prefers-color-scheme: dark)\" content=\"#15171b\">\n";

/// Pages load only the site's own files, so the policy allows nothing else.
/// A `<meta>` because GitHub Pages can't send headers (ADR 2026-10-28-csp-meta-tag).
const CSP: &str = "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; \
script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; manifest-src 'self'; connect-src 'self'; \
base-uri 'none'; form-action 'none'\">\n";

/// The author as readers see it, "JC", with the full name kept for mf2
/// parsers as `p-name` (ADR 2026-10-16-site-title-heads-home).
/// The author's address: `author_email_user` at the site's domain, without
/// a leading `www.` (ADR 2026-11-06-coded-email).
fn author_email(site: &Config) -> String {
    let host = site.base_url.split("://").nth(1).unwrap_or(site.base_url);
    let host = host.split('/').next().unwrap_or(host);
    format!("{}@{}", site.author_email_user, host.trim_start_matches("www."))
}

/// `mailto:` and the author's address, every character as an HTML numeric
/// character reference. Browsers and HTML parsers (IndieLogin's included)
/// decode it; most spam harvesters search the raw source and miss it
/// (ADR 2026-11-06-coded-email).
fn coded_mailto(site: &Config) -> String {
    format!("mailto:{}", author_email(site))
        .chars()
        .map(|c| format!("&#{};", c as u32))
        .collect()
}

/// The name other sites and readers show for the author, "JC (Jonathan
/// Chen)": the nickname people know, with the full name people search for
/// (ADR 2026-11-06-profile-name).
pub fn author_display_name(site: &Config) -> String {
    format!("{} ({})", site.author_nickname, site.author_name)
}

fn author_label(site: &Config) -> String {
    format!(
        "<span class=\"p-nickname\">{}</span><data class=\"p-name\" value=\"{}\"></data>",
        e(site.author_nickname),
        e(&author_display_name(site))
    )
}

/// The follow page's addresses to paste into a feed reader, written out in
/// full since that's what a reader asks for. The build adds them because
/// content can't link a site URL (ADR 2026-11-05-follow-with-a-feed-reader).
/// `<wbr>` lets a phone wrap before the file name rather than inside it.
fn follow_feeds(site: &Config, links: FooterLinks) -> String {
    let item = |label: &str, path: &str, kind: &str| {
        format!(
            "<li>{label}: <a href=\"{path}\" type=\"{kind}\">{}<wbr>{path}</a></li>\n",
            e(site.base_url)
        )
    };
    let mut list = item("This bliki's feed", "/index.xml", "application/rss+xml");
    if links.blogroll {
        list.push_str(&item("The blogroll, as OPML", BLOGROLL_OPML, "text/x-opml"));
    }
    format!("<ul class=\"follow-feeds\">\n{list}</ul>\n")
}

fn render_page(site: &Config, notes: &[Note], n: &Note, canonical: &str) -> Result<String, String> {
    let (after, mut meta) = if n.slug == BLOGROLL {
        (
            format!(
                "<p class=\"blogroll-opml\">All of these as one <a href=\"{BLOGROLL_OPML}\" type=\"text/x-opml\">OPML file</a> to import into your feed reader.</p>\n"
            ),
            format!(
                "<link rel=\"alternate\" type=\"text/x-opml\" title=\"{} blogroll\" href=\"{BLOGROLL_OPML}\">\n",
                e(site.title)
            ),
        )
    } else if n.slug == FOLLOW {
        (follow_feeds(site, footer_links(notes)), String::new())
    } else {
        (String::new(), String::new())
    };
    meta.push_str(&summary_meta(site, n));
    // A now page is only as good as its date, so it must have one
    // (ADR 2026-11-01-now-page).
    let updated = if n.slug == NOW {
        let u = n
            .updated_at
            .ok_or("now.md: a now page needs `updated_at`, so readers know how current it is")?;
        format!(
            "<p class=\"now-updated\">Updated <time datetime=\"{}\">{}</time></p>\n",
            u.iso(),
            u.day.human()
        )
    } else {
        String::new()
    };
    // The blogroll's OPML link goes right under the reading list it
    // exports, before any later section (ADR 2026-11-07-indie-web-links-on-the-blogroll).
    let (body, after) = if n.slug == BLOGROLL {
        let (list, rest) = blogroll::split(&n.content_html);
        (format!("{list}{after}{rest}"), String::new())
    } else {
        (n.content_html.clone(), after)
    };
    let content = format!(
        "\n<article>\n<h1>{}</h1>\n{updated}{body}\n{after}</article>\n{}\n",
        e(n.title()),
        backlinks_block(notes, n)
    );
    Ok(base(
        site,
        footer_links(notes),
        &format!("{} \u{2014} {}", e(n.title()), e(site.title)),
        &meta,
        Some(canonical),
        &content,
    ))
}

/// A listed note's dates: when it was published, and when it was updated
/// if that came later, so a note that moved up the list says why. An
/// evergreen note shows only when it was updated (ADR 2026-11-07-evergreen-notes).
fn list_dates(n: &Note) -> Result<String, String> {
    let time = |class: &str, iso: String, day: Date, label: &str| {
        format!(
            "<time class=\"{class}\" datetime=\"{iso}\">{label}{}</time>",
            day.human()
        )
    };
    Ok(match (n.evergreen(), n.date, n.update()) {
        (true, _, Some(u)) => time("dt-updated", u.iso(), u.day, "Updated "),
        (_, Some(d), Some(u)) => format!(
            "{}\n \u{b7}\u{a0}{}",
            time("dt-published", d.iso(), d, ""),
            time("dt-updated", u.iso(), u.day, "updated ")
        ),
        _ => {
            let d = n.post_date()?;
            time("dt-published", d.iso(), d, "")
        }
    })
}

fn render_home(site: &Config, notes: &[Note], n: &Note, listed: &[usize], canonical: &str) -> Result<String, String> {
    let mut recent = String::new();
    for &p in listed.iter().take(10) {
        let pn = &notes[p];
        let dates = list_dates(pn)?;
        // Title words alone say little ("raw", "meld"), so each post shows a
        // short excerpt (ADR 2026-10-16-home-post-excerpts).
        let excerpt = match pn.summary() {
            Some(s) => s.to_string(),
            None => text::excerpt(&pn.content_html, DESCRIPTION_MAX),
        };
        let excerpt = if excerpt.is_empty() {
            String::new()
        } else {
            format!("<p class=\"p-summary\">{}</p>\n", e(&excerpt))
        };
        recent.push_str(&format!(
            "\n<li class=\"h-entry\">\n<a class=\"u-url p-name\" href=\"{}\">{}</a>\n{}\n{}</li>\n",
            e(&pn.url),
            e(pn.title()),
            dates,
            excerpt
        ));
    }
    let intro = if n.content_html.trim().is_empty() {
        String::new()
    } else {
        format!("\n<article>\n{}\n</article>\n", n.content_html)
    };
    let content = format!(
        "{intro}\n<section class=\"h-feed\">\n<h2 class=\"p-name\">Recent notes</h2>\n<ul>\n{recent}\n</ul>\n\
<p><a href=\"/notes/\">All notes</a></p>\n</section>\n"
    );
    let meta = match n.summary() {
        Some(s) => description_meta(s),
        None => description_meta(site.tagline),
    };
    Ok(page(
        site,
        footer_links(notes),
        &e(site.title),
        &meta,
        Some(canonical),
        &content,
        true,
    ))
}

fn render_list(
    site: &Config,
    notes: &[Note],
    list_title: &str,
    description: &str,
    listed: &[usize],
    canonical: &str,
) -> Result<String, String> {
    let mut items = String::new();
    for &p in listed {
        let pn = &notes[p];
        let dates = list_dates(pn)?;
        let summary = pn
            .summary()
            .map(|s| format!(" \u{2014} <span class=\"p-summary\">{}</span>", e(s)))
            .unwrap_or_default();
        items.push_str(&format!(
            "\n<li class=\"h-entry\">\n<a class=\"u-url p-name\" href=\"{}\">{}</a>\n{}\n{}\n</li>\n",
            e(&pn.url),
            e(pn.title()),
            dates,
            summary
        ));
    }
    let content = format!(
        "\n<section class=\"h-feed\">\n<h1 class=\"p-name\">{}</h1>\n<ul>\n{}\n</ul>\n</section>\n",
        e(list_title),
        items
    );
    Ok(base(
        site,
        footer_links(notes),
        &format!("{} \u{2014} {}", e(list_title), e(site.title)),
        &description_meta(description),
        Some(canonical),
        &content,
    ))
}

fn render_redirect(site: &Config, links: FooterLinks, target: &str, canonical: &str) -> String {
    let t = e(target);
    let meta = format!(
        "\n<meta http-equiv=\"refresh\" content=\"0; url={}\">\n<meta name=\"robots\" content=\"noindex\">\n",
        t
    );
    let content = format!("\n<p>This page has moved to <a href=\"{}\">{}</a>.</p>\n", t, t);
    base(site, links, "Redirecting\u{2026}", &meta, Some(canonical), &content)
}

/// GitHub Pages serves this for every missing address, at any depth, so its
/// links stay root-relative (ADR 2026-11-06-not-found-page).
fn render_404(site: &Config, links: FooterLinks) -> String {
    let follow = if links.follow {
        "<li><a href=\"/follow/\">Follow</a> in a feed reader to get new posts</li>\n"
    } else {
        ""
    };
    let content = format!(
        "\n<article>\n<h1>Page not found</h1>\n<p>There's no page at this address. It may have a typo, or the link that \
brought you here may be broken. Try one of these:</p>\n<ul>\n<li>The <a href=\"/\">home page</a> for the latest \
notes</li>\n<li>All <a href=\"/notes/\">notes</a>, most recently updated first</li>\n{follow}</ul>\n</article>\n"
    );
    base(
        site,
        links,
        &format!("Page not found \u{2014} {}", e(site.title)),
        "\n<meta name=\"robots\" content=\"noindex\">\n",
        None,
        &content,
    )
}

/// What the service worker shows for a page that wasn't kept, when the
/// reader is offline (ADR 2026-10-29-offline-reading).
fn render_offline(site: &Config, links: FooterLinks) -> String {
    let content = "\n<article>\n<h1>You're offline</h1>\n<p>This page isn't saved on this device yet. Pages you've read here are, so try the <a href=\"/\">home page</a> or go back.</p>\n</article>\n";
    base(
        site,
        links,
        &format!("Offline \u{2014} {}", e(site.title)),
        "\n<meta name=\"robots\" content=\"noindex\">\n",
        None,
        content,
    )
}

// --- main entry point ---

pub fn build_site(content: &Path, out: &Path, site: &Config) -> Result<(), String> {
    let mut notes: Vec<Note> = load_notes(content)?
        .into_iter()
        .filter(|n| !n.draft() && n.goes_live().is_none_or(|d| d <= site.today))
        .collect();

    // unique slugs
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (i, n) in notes.iter().enumerate() {
        if n.slug == "index" {
            continue;
        }
        if let Some(&j) = seen.get(n.slug.as_str()) {
            return Err(format!(
                "duplicate slug {}: {} vs {}",
                py_repr_str(&n.slug),
                notes[j].rel,
                n.rel
            ));
        }
        seen.insert(&n.slug, i);
    }

    for n in notes.iter_mut() {
        n.out_path = out_path_for(out, &n.url);
    }
    let slug_to_url: HashMap<String, String> = notes.iter().map(|n| (n.slug.clone(), n.url.clone())).collect();
    let rel_to_url: HashMap<String, String> = notes.iter().map(|n| (n.rel.clone(), n.url.clone())).collect();

    // render markdown
    for n in notes.iter_mut() {
        let body = text::resolve_wikilinks(&n.body, &slug_to_url, &n.rel)?;
        let mut html = text::resolve_md_links(&markdown::render(&body), &n.rel, &rel_to_url)?;
        if n.slug == BIO && n.kind == Kind::Page {
            // Content can't hold raw HTML, so the build adds the photo; it
            // goes through prepare_images like any other for its size and
            // the missing-image check (ADR 2026-11-06-h-card-photo).
            html.insert_str(
                0,
                &format!(
                    "<p class=\"bio-photo\"><img src=\"{}\" alt=\"{}\"></p>\n",
                    e(site.author_photo),
                    e(site.author_nickname)
                ),
            );
        }
        // `[text](mailto:)` with no address emails the author, so content
        // never spells the address out (ADR 2026-11-06-coded-email).
        let html = html.replace("href=\"mailto:\"", &format!("href=\"{}\"", coded_mailto(site)));
        n.content_html = prepare_images(&html, content, &n.rel)?;
    }

    // backlinks
    let by_url: HashMap<String, usize> = notes.iter().enumerate().map(|(i, n)| (n.url.clone(), i)).collect();
    let mut back: Vec<Vec<usize>> = vec![Vec::new(); notes.len()];
    for (i, n) in notes.iter().enumerate() {
        for url in text::hrefs(&n.content_html) {
            if let Some(&t) = by_url.get(&url) {
                if t != i && !back[t].contains(&i) {
                    back[t].push(i);
                }
            }
        }
    }
    for (i, mut b) in back.into_iter().enumerate() {
        b.sort_by(|x, y| notes[*x].url.cmp(&notes[*y].url));
        notes[i].backlinks = b;
    }

    if out.exists() {
        fs::remove_dir_all(out).map_err(|err| format!("cannot remove {}: {}", out.display(), err))?;
    }
    fs::create_dir_all(out).map_err(|err| format!("cannot create {}: {}", out.display(), err))?;

    let base_url = site.base_url.trim_end_matches('/');
    let canonical = |url: &str| format!("{}{}", base_url, url);

    // The feeds carry dated posts, newest first. Home and /notes/ list every
    // note, evergreen ones too, most recently tended first, so a note moves
    // up when it is updated (ADR 2026-11-07-evergreen-notes).
    let mut posts: Vec<usize> = (0..notes.len())
        .filter(|&i| notes[i].kind == Kind::Post && !notes[i].evergreen())
        .collect();
    posts.sort_by(|&a, &b| {
        let ka = (notes[a].date.unwrap_or(date::MIN), &notes[a].slug);
        let kb = (notes[b].date.unwrap_or(date::MIN), &notes[b].slug);
        kb.cmp(&ka)
    });
    let mut listed: Vec<usize> = (0..notes.len()).filter(|&i| notes[i].kind == Kind::Post).collect();
    listed.sort_by(|&a, &b| {
        let ka = (notes[a].tended().unwrap_or(i64::MIN), &notes[a].slug);
        let kb = (notes[b].tended().unwrap_or(i64::MIN), &notes[b].slug);
        kb.cmp(&ka)
    });

    // pages
    for n in &notes {
        let c = canonical(&n.url);
        let html = match n.kind {
            Kind::Home => render_home(site, &notes, n, &listed, &c)?,
            Kind::Post => render_post(site, &notes, n, &c)?,
            Kind::Page => render_page(site, &notes, n, &c)?,
        };
        write(&n.out_path, &html)?;
    }

    // /notes/ list
    write(
        &out.join("notes").join("index.html"),
        &render_list(
            site,
            &notes,
            "Notes",
            &format!("Every note on {}, most recently updated first.", site.title),
            &listed,
            &canonical("/notes/"),
        )?,
    )?;

    // alias redirects
    for n in &notes {
        for alias in n.str_list("aliases") {
            let dir = alias.trim_matches('/');
            let p = if dir.is_empty() {
                out.join("index.html")
            } else {
                out.join(dir).join("index.html")
            };
            write(
                &p,
                &render_redirect(site, footer_links(&notes), &n.url, &canonical(&n.url)),
            )?;
        }
    }

    // spec/url-policy.md: the Jekyll site's A-to-Z post list (ADR 2026-10-16-old-all-page).
    let notes_index = render_redirect(site, footer_links(&notes), "/notes/", &canonical("/notes/"));
    write(&out.join("all/index.html"), &notes_index)?;
    write(&out.join("all.html"), &notes_index)?;
    for (tag, to) in OLD_TAG_PAGES {
        write(
            &out.join("tags").join(tag).join("index.html"),
            &render_redirect(site, footer_links(&notes), to, &canonical(to)),
        )?;
    }

    // 404
    write(&out.join("404.html"), &render_404(site, footer_links(&notes)))?;
    write(&out.join("offline.html"), &render_offline(site, footer_links(&notes)))?;

    // spec/feeds.md: the blogroll's sites as OPML (ADR 2026-10-25-blogroll).
    if let Some(n) = notes.iter().find(|n| n.slug == BLOGROLL && n.kind == Kind::Page) {
        let sites = blogroll::sites(&n.content_html)?;
        let title = format!("{} blogroll", site.title);
        write(
            &out.join(BLOGROLL_OPML.trim_start_matches('/')),
            &blogroll::opml(&title, &author_display_name(site), &format!("{}/", base_url), &sites),
        )?;
    }

    // RSS
    let mut rss = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\">\n<channel>\n\
<title>{}</title>\n<link>{}/</link>\n<description>{}</description>\n<language>{}</language>\n\
<atom:link href=\"{}/index.xml\" rel=\"self\" type=\"application/rss+xml\"/>",
        e(site.title),
        e(site.base_url),
        e(site.tagline),
        e(site.language),
        e(site.base_url)
    );
    let rfc = |i: usize| notes[i].date.unwrap_or(date::EPOCH).rfc822();
    if let Some(&first) = posts.first() {
        rss.push_str(&format!("\n<lastBuildDate>{}</lastBuildDate>", e(&rfc(first))));
    }
    // spec/feeds.md: the RSS feed carries at most the 50 newest posts, each
    // with a plain summary and the whole post (ADR 2026-11-07-full-text-rss).
    for &p in posts.iter().take(50) {
        let n = &notes[p];
        let description = match n.summary() {
            Some(s) => s.to_string(),
            None => text::excerpt(&n.content_html, RSS_DESCRIPTION_MAX),
        };
        rss.push_str(&format!(
            "\n<item>\n<title>{t}</title>\n<link>{b}{u}</link>\n<guid isPermaLink=\"true\">{b}{u}</guid>\n<pubDate>{d}</pubDate>\n<description>{desc}</description>\n<content:encoded>{full}</content:encoded>\n</item>",
            t = e(n.title()),
            b = e(site.base_url),
            u = e(&n.url),
            d = e(&rfc(p)),
            desc = e(&description),
            full = e(&text::absolute_urls(&n.content_html, site.base_url, &n.url)),
        ));
    }
    rss.push_str("\n</channel>\n</rss>\n");
    write(&out.join("index.xml"), &rss)?;
    // spec/feeds.md: the feed URLs this domain served before, Substack's and
    // Jekyll's, get the same feed (ADR 2026-10-16-old-feed-urls).
    for old in ["feed", "feed.xml"] {
        write(&out.join(old), &rss)?;
    }

    // JSON Feed
    let items: Vec<Json> = posts
        .iter()
        .map(|&p| {
            let n = &notes[p];
            let mut item = vec![
                ("id".to_string(), Json::Str(canonical(&n.url))),
                ("url".to_string(), Json::Str(canonical(&n.url))),
                ("title".to_string(), json::s(n.title())),
                ("content_html".to_string(), json::s(&n.content_html)),
                (
                    "date_published".to_string(),
                    Json::Str(n.date.unwrap_or(date::EPOCH).iso_datetime()),
                ),
            ];
            if let Some(s) = n.summary() {
                item.push(("summary".to_string(), json::s(s)));
            }
            Json::Obj(item)
        })
        .collect();
    let feed = Json::Obj(vec![
        ("version".into(), json::s("https://jsonfeed.org/version/1.1")),
        ("title".into(), json::s(site.title)),
        ("home_page_url".into(), Json::Str(format!("{}/", base_url))),
        ("feed_url".into(), Json::Str(format!("{}/feed.json", base_url))),
        ("language".into(), json::s(site.language)),
        (
            "authors".into(),
            Json::Arr(vec![Json::Obj(vec![
                ("name".into(), json::s(&author_display_name(site))),
                ("url".into(), json::s(site.author_url)),
            ])]),
        ),
        ("items".into(), Json::Arr(items)),
    ]);
    write(&out.join("feed.json"), &(json::dumps(&feed) + "\n"))?;

    // sitemap
    let mut order: Vec<usize> = (0..notes.len()).collect();
    order.sort_by(|&a, &b| notes[a].url.cmp(&notes[b].url));
    let mut urls: Vec<(String, String)> = order
        .iter()
        .map(|&i| {
            let n = &notes[i];
            (canonical(&n.url), n.last_modified().unwrap_or(date::EPOCH).iso())
        })
        .collect();
    urls.push((canonical("/notes/"), date::EPOCH.iso()));
    let mut sm = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">",
    );
    for (u, lastmod) in &urls {
        sm.push_str(&format!(
            "\n<url>\n<loc>{}</loc>\n<lastmod>{}</lastmod>\n</url>",
            e(u),
            e(lastmod)
        ));
    }
    sm.push_str("\n</urlset>\n");
    write(&out.join("sitemap.xml"), &sm)?;

    // robots.txt: search crawlers welcome, AI training crawlers asked out
    // (ADR 2026-11-06-no-ai-training).
    let training_agents: String = AI_TRAINING_AGENTS
        .iter()
        .map(|a| format!("User-agent: {a}\n"))
        .collect();
    write(
        &out.join("robots.txt"),
        &format!(
            "# Search engines and everyone else may crawl everything.\nUser-agent: *\nAllow: /\n\n\
# AI training crawlers may not.\n{training_agents}Disallow: /\n\nSitemap: {base_url}/sitemap.xml\n"
        ),
    )?;

    // security.txt (RFC 9116), dated off the build's today so the daily
    // rebuild keeps Expires ahead (ADR 2026-10-28-security-txt).
    let expires = Date::from_days_since_epoch(site.today.days_from_epoch() + SECURITY_TXT_DAYS);
    write(
        &out.join(".well-known").join("security.txt"),
        &format!(
            "Contact: {SECURITY_CONTACT}\nExpires: {}T00:00:00Z\nPreferred-Languages: en\nCanonical: {}/.well-known/security.txt\n",
            expires.iso(),
            base_url
        ),
    )?;

    // static assets
    write(&out.join("style.css"), STYLE_CSS)?;
    for (slug, _, _, css) in THEMES {
        write(&out.join("themes").join(format!("{slug}.css")), css)?;
    }
    write(&out.join("style-picker.js"), STYLE_PICKER_JS)?;
    write(&out.join("sw.js"), SERVICE_WORKER_JS)?;
    for (name, bytes) in ICONS {
        write_bytes(&out.join(name), bytes)?;
    }
    copy_images(&content.join("images"), &out.join("images"))?;
    for (name, bytes) in FONTS {
        write_bytes(&out.join("fonts").join(name), bytes)?;
    }
    apply_base_path(out, &base_path(site.base_url))?;
    Ok(())
}
