//! The build pipeline: load notes, validate, resolve links, render pages,
//! feeds, sitemap and redirects. Templates are plain Rust functions whose
//! output matches the former Jinja templates byte for byte (autoescape on).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::date::{self, Date};
use crate::frontmatter::{self, FrontMatter, LoadError, Value};
use crate::json::{self, Json};
use crate::markdown;
use crate::text;
use crate::util::{escape as e, py_repr_str};

const STYLE_CSS: &str = include_str!("static/style.css");

/// Site configuration (edit before going live).
pub struct Config {
    pub title: &'static str,
    pub tagline: &'static str,
    pub base_url: &'static str,
    pub language: &'static str,
    pub author_name: &'static str,
    pub author_url: &'static str,
    pub rel_me: &'static [&'static str],
    pub webmention_endpoint: &'static str,
    pub pingback: &'static str,
}

pub fn default_config() -> Config {
    Config {
        title: "posse-bliki",
        tagline: "A personal POSSE bliki.",
        base_url: "https://example.com",
        language: "en",
        author_name: "Author Name",
        author_url: "https://example.com",
        rel_me: &["https://github.com/dijonkitchen"],
        webmention_endpoint: "https://webmention.io/example.com/webmention",
        pingback: "https://webmention.io/example.com/xmlrpc",
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
                    ("url".into(), json::s(self.author_url)),
                    ("rel_me".into(), Json::Arr(self.rel_me.iter().map(|u| json::s(u)).collect())),
                ]),
            ),
            (
                "webmention".into(),
                Json::Obj(vec![
                    ("endpoint".into(), json::s(self.webmention_endpoint)),
                    ("pingback".into(), json::s(self.pingback)),
                ]),
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
    tags: Vec<String>,
    date: Option<Date>,
    updated: Option<Date>,
}

impl Note {
    fn title(&self) -> &str {
        frontmatter::get(&self.fm, "title").and_then(Value::as_str).unwrap_or("")
    }

    /// `summary` when truthy.
    fn summary(&self) -> Option<&str> {
        frontmatter::get(&self.fm, "summary").and_then(Value::as_str).filter(|s| !s.is_empty())
    }

    fn str_list(&self, key: &str) -> Vec<String> {
        match frontmatter::get(&self.fm, key) {
            Some(Value::List(l)) => l.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
            _ => Vec::new(),
        }
    }

    fn draft(&self) -> bool {
        frontmatter::get(&self.fm, "draft").map_or(false, Value::truthy)
    }

    fn post_date(&self) -> Result<Date, String> {
        self.date.ok_or_else(|| format!("{}: post has no `date` (required for dt-published)", self.rel))
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
        if let Err(msg) = frontmatter::validate(&fm) {
            return Err(format!("{}: front-matter invalid: {}", rel, msg));
        }
        let kind = kind_for(&parts);
        let slug = if kind == Kind::Home { "index".to_string() } else { text::slugify(stem(parts.last().unwrap())) };
        let parse_date = |key: &str| -> Result<Option<Date>, String> {
            match frontmatter::get(&fm, key).and_then(Value::as_str) {
                None | Some("") => Ok(None),
                Some(s) => Date::parse(s)
                    .map(Some)
                    .ok_or_else(|| format!("{}: invalid {} {}", rel, key, py_repr_str(s))),
            }
        };
        let date = parse_date("date")?;
        let updated = parse_date("updated")?;
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
            tags: Vec::new(),
            date,
            updated,
        });
    }
    Ok(notes)
}

fn write(p: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("cannot create {}: {}", parent.display(), err))?;
    }
    fs::write(p, content).map_err(|err| format!("cannot write {}: {}", p.display(), err))
}

/// The path part of `base_url`: `/posse-bliki` for a GitHub Pages project
/// site, empty when the site is served from a domain root.
fn base_path(base_url: &str) -> String {
    let rest = base_url.split_once("://").map_or(base_url, |(_, r)| r);
    rest.find('/').map_or("", |i| &rest[i..]).trim_end_matches('/').to_string()
}

/// Root-relative URL openers the pages and feeds emit: attributes in HTML,
/// the same escaped inside RSS (`&quot;`) and JSON Feed (`\"`) bodies, and
/// the meta-refresh target of alias redirects.
const ROOT_RELATIVE: &[&str] = &[
    "href=\"/", "src=\"/", "href=&quot;/", "src=&quot;/", "href=\\\"/", "src=\\\"/", "url=/",
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

fn base(site: &Config, title: &str, meta: &str, canonical: Option<&str>, content: &str) -> String {
    let t = e(site.title);
    let canon = canonical.map(|c| format!("<link rel=\"canonical\" href=\"{}\">", e(c))).unwrap_or_default();
    format!(
        "<!DOCTYPE html>\n<html lang=\"{lang}\">\n<head>\n<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n{meta}\n{canon}\n\
<link rel=\"alternate\" type=\"application/rss+xml\" title=\"{t}\" href=\"/index.xml\">\n\
<link rel=\"alternate\" type=\"application/feed+json\" title=\"{t}\" href=\"/feed.json\">\n\
<link rel=\"webmention\" href=\"{wm}\">\n<link rel=\"pingback\" href=\"{pb}\">\n\
<link rel=\"stylesheet\" href=\"/style.css\">\n</head>\n<body>\n<header class=\"site-header\">\n<nav>\n\
<a href=\"/\">{t}</a>\n<a href=\"/notes/\">notes</a>\n<a href=\"/about/\">about</a>\n<a href=\"/colophon/\">colophon</a>\n\
</nav>\n</header>\n<main>\n{content}\n</main>\n<footer class=\"site-footer\">\n<p>\n\
<a href=\"/index.xml\">RSS</a> \u{b7}\n<a href=\"/feed.json\">JSON Feed</a> \u{b7}\n<a href=\"/colophon/\">Colophon</a>\n\
</p>\n</footer>\n</body>\n</html>\n",
        lang = e(site.language),
        wm = e(site.webmention_endpoint),
        pb = e(site.pingback),
    )
}

fn summary_meta(n: &Note) -> String {
    n.summary().map(|s| format!("<meta name=\"description\" content=\"{}\">", e(s))).unwrap_or_default()
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
    format!("\n<aside class=\"backlinks\">\n<h2>Linked from</h2>\n<ul>\n{}\n</ul>\n</aside>\n", items)
}

fn render_post(site: &Config, notes: &[Note], n: &Note, canonical: &str) -> Result<String, String> {
    let d = n.post_date()?;
    let updated = n
        .updated
        .map(|u| format!(" \u{b7} <time class=\"dt-updated\" datetime=\"{}\">updated {}</time>", u.iso(), u.ymd()))
        .unwrap_or_default();
    let summary = n.summary().map(|s| format!("<p class=\"p-summary\">{}</p>", e(s))).unwrap_or_default();
    let tags = if n.tags.is_empty() {
        String::new()
    } else {
        let links: Vec<String> = n
            .tags
            .iter()
            .map(|t| format!("<a class=\"p-category\" href=\"/tags/{}/\">#{}</a>", e(t), e(t)))
            .collect();
        format!("<p class=\"tags\">{}</p>", links.join(" "))
    };
    let synd = n.str_list("syndication");
    let synd = if synd.is_empty() {
        String::new()
    } else {
        let links: Vec<String> =
            synd.iter().map(|s| format!("<a class=\"u-syndication\" href=\"{}\">{}</a>", e(s), e(s))).collect();
        format!("\n<footer>\n<p>Also on: {}</p>\n</footer>\n", links.join(", "))
    };
    let content = format!(
        "\n<article class=\"h-entry\">\n<header>\n<h1 class=\"p-name\">{title}</h1>\n<p class=\"meta\">\n\
<a class=\"u-url u-uid\" href=\"{canon}\"><time class=\"dt-published\" datetime=\"{iso}\">{ymd}</time></a>\n\
{updated}\n \u{b7} <span class=\"p-author h-card\"><a class=\"u-url p-name\" rel=\"author\" href=\"{base}/\">{author}</a></span>\n\
</p>\n{summary}\n{tags}\n</header>\n<div class=\"e-content\">\n{html}\n</div>\n{synd}\n</article>\n{back}\n",
        title = e(n.title()),
        canon = e(canonical),
        iso = d.iso(),
        ymd = d.ymd(),
        base = e(site.base_url),
        author = e(site.author_name),
        html = n.content_html,
        back = backlinks_block(notes, n),
    );
    Ok(base(site, &format!("{} \u{2014} {}", e(n.title()), e(site.title)), &summary_meta(n), Some(canonical), &content))
}

fn render_page(site: &Config, notes: &[Note], n: &Note, canonical: &str) -> String {
    let content = format!(
        "\n<article>\n<h1>{}</h1>\n{}\n</article>\n{}\n",
        e(n.title()),
        n.content_html,
        backlinks_block(notes, n)
    );
    base(site, &format!("{} \u{2014} {}", e(n.title()), e(site.title)), &summary_meta(n), Some(canonical), &content)
}

fn render_home(site: &Config, notes: &[Note], n: &Note, posts: &[usize], canonical: &str) -> Result<String, String> {
    let rel_me: String =
        site.rel_me.iter().map(|u| format!("\n<a class=\"u-url\" rel=\"me\" href=\"{}\">{}</a>\n", e(u), e(u))).collect();
    let mut recent = String::new();
    for &p in posts.iter().take(10) {
        let pn = &notes[p];
        let d = pn.post_date()?;
        recent.push_str(&format!(
            "\n<li class=\"h-entry\">\n<a class=\"u-url p-name\" href=\"{}\">{}</a>\n<time class=\"dt-published\" datetime=\"{}\">{}</time>\n</li>\n",
            e(&pn.url),
            e(pn.title()),
            d.iso(),
            d.ymd()
        ));
    }
    let content = format!(
        "\n<section class=\"h-card\">\n<p>\n<span class=\"p-name\">{author}</span>\n\
<a class=\"u-url u-uid\" rel=\"me\" href=\"{base}/\">{base}/</a>\n{rel_me}\n</p>\n</section>\n<article>\n{html}\n</article>\n\
<section class=\"h-feed\">\n<h2 class=\"p-name\">Recent posts</h2>\n<ul>\n{recent}\n</ul>\n</section>\n",
        author = e(site.author_name),
        base = e(site.base_url),
        html = n.content_html,
    );
    Ok(base(site, &e(site.title), "", Some(canonical), &content))
}

fn render_list(site: &Config, notes: &[Note], list_title: &str, posts: &[usize], canonical: &str) -> Result<String, String> {
    let mut items = String::new();
    for &p in posts {
        let pn = &notes[p];
        let d = pn.post_date()?;
        let summary =
            pn.summary().map(|s| format!(" \u{2014} <span class=\"p-summary\">{}</span>", e(s))).unwrap_or_default();
        items.push_str(&format!(
            "\n<li class=\"h-entry\">\n<a class=\"u-url p-name\" href=\"{}\">{}</a>\n<time class=\"dt-published\" datetime=\"{}\">{}</time>\n{}\n</li>\n",
            e(&pn.url),
            e(pn.title()),
            d.iso(),
            d.ymd(),
            summary
        ));
    }
    let content = format!(
        "\n<section class=\"h-feed\">\n<h1 class=\"p-name\">{}</h1>\n<ul>\n{}\n</ul>\n</section>\n",
        e(list_title),
        items
    );
    Ok(base(site, &format!("{} \u{2014} {}", e(list_title), e(site.title)), "", Some(canonical), &content))
}

fn render_redirect(site: &Config, target: &str, canonical: &str) -> String {
    let t = e(target);
    let meta = format!("\n<meta http-equiv=\"refresh\" content=\"0; url={}\">\n<meta name=\"robots\" content=\"noindex\">\n", t);
    let content = format!("\n<p>This page has moved to <a href=\"{}\">{}</a>.</p>\n", t, t);
    base(site, "Redirecting\u{2026}", &meta, Some(canonical), &content)
}

fn render_404(site: &Config) -> String {
    let content = "\n<article>\n<h1>Not found</h1>\n<p>That page isn't here. Try the <a href=\"/\">home page</a> or the <a href=\"/notes/\">notes index</a>.</p>\n</article>\n";
    base(site, &format!("Not found \u{2014} {}", e(site.title)), "", None, content)
}

// --- main entry point ---

pub fn build_site(content: &Path, out: &Path, site: &Config) -> Result<(), String> {
    let mut notes: Vec<Note> = load_notes(content)?.into_iter().filter(|n| !n.draft()).collect();

    // unique slugs
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (i, n) in notes.iter().enumerate() {
        if n.slug == "index" {
            continue;
        }
        if let Some(&j) = seen.get(n.slug.as_str()) {
            return Err(format!("duplicate slug {}: {} vs {}", py_repr_str(&n.slug), notes[j].rel, n.rel));
        }
        seen.insert(&n.slug, i);
    }

    for n in notes.iter_mut() {
        n.out_path = out_path_for(out, &n.url);
    }
    let slug_to_url: HashMap<String, String> = notes.iter().map(|n| (n.slug.clone(), n.url.clone())).collect();

    // render markdown
    for n in notes.iter_mut() {
        let body = text::resolve_wikilinks(&n.body, &slug_to_url, &n.rel)?;
        let (body, inline_tags) = text::extract_inline_tags(&body);
        let mut tags = n.str_list("tags");
        tags.extend(inline_tags);
        tags.sort();
        tags.dedup();
        n.tags = tags;
        n.content_html = markdown::render(&body);
    }
    for n in &notes {
        for t in &n.tags {
            if !frontmatter::is_tag(t) {
                return Err(format!("{}: invalid tag {}", n.rel, py_repr_str(t)));
            }
        }
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

    let mut posts: Vec<usize> = (0..notes.len()).filter(|&i| notes[i].kind == Kind::Post).collect();
    posts.sort_by(|&a, &b| {
        let ka = (notes[a].date.unwrap_or(date::MIN), &notes[a].slug);
        let kb = (notes[b].date.unwrap_or(date::MIN), &notes[b].slug);
        kb.cmp(&ka)
    });

    // pages
    for n in &notes {
        let c = canonical(&n.url);
        let html = match n.kind {
            Kind::Home => render_home(site, &notes, n, &posts, &c)?,
            Kind::Post => render_post(site, &notes, n, &c)?,
            Kind::Page => render_page(site, &notes, n, &c),
        };
        write(&n.out_path, &html)?;
    }

    // /notes/ list
    write(&out.join("notes").join("index.html"), &render_list(site, &notes, "Notes", &posts, &canonical("/notes/"))?)?;

    // tag pages
    let mut by_tag: Vec<(String, Vec<usize>)> = Vec::new();
    for &p in &posts {
        for t in &notes[p].tags {
            match by_tag.iter_mut().find(|(k, _)| k == t) {
                Some((_, v)) => v.push(p),
                None => by_tag.push((t.clone(), vec![p])),
            }
        }
    }
    by_tag.sort_by(|a, b| a.0.cmp(&b.0));
    for (tag, tag_posts) in &by_tag {
        let url = format!("/tags/{}/", tag);
        let html = render_list(site, &notes, &format!("#{}", tag), tag_posts, &canonical(&url))?;
        write(&out.join("tags").join(tag).join("index.html"), &html)?;
    }

    // alias redirects
    for n in &notes {
        for alias in n.str_list("aliases") {
            let dir = alias.trim_matches('/');
            let p = if dir.is_empty() { out.join("index.html") } else { out.join(dir).join("index.html") };
            write(&p, &render_redirect(site, &n.url, &canonical(&n.url)))?;
        }
    }

    // 404
    write(&out.join("404.html"), &render_404(site))?;

    // RSS
    let mut rss = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n\
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
    for &p in &posts {
        let n = &notes[p];
        let description = match n.summary() {
            Some(s) => s.to_string(),
            None => text::strip_html(&n.content_html).chars().take(200).collect(),
        };
        rss.push_str(&format!(
            "\n<item>\n<title>{t}</title>\n<link>{b}{u}</link>\n<guid isPermaLink=\"true\">{b}{u}</guid>\n<pubDate>{d}</pubDate>\n<description>{desc}</description>\n</item>",
            t = e(n.title()),
            b = e(site.base_url),
            u = e(&n.url),
            d = e(&rfc(p)),
            desc = e(&description),
        ));
    }
    rss.push_str("\n</channel>\n</rss>\n");
    write(&out.join("index.xml"), &rss)?;

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
                ("date_published".to_string(), Json::Str(n.date.unwrap_or(date::EPOCH).iso_datetime())),
            ];
            if let Some(s) = n.summary() {
                item.push(("summary".to_string(), json::s(s)));
            }
            item.push(("tags".to_string(), Json::Arr(n.tags.iter().map(|t| json::s(t)).collect())));
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
                ("name".into(), json::s(site.author_name)),
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
            (canonical(&n.url), n.updated.or(n.date).unwrap_or(date::EPOCH).iso())
        })
        .collect();
    urls.push((canonical("/notes/"), date::EPOCH.iso()));
    for (tag, _) in &by_tag {
        urls.push((canonical(&format!("/tags/{}/", tag)), date::EPOCH.iso()));
    }
    let mut sm = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">",
    );
    for (u, lastmod) in &urls {
        sm.push_str(&format!("\n<url>\n<loc>{}</loc>\n<lastmod>{}</lastmod>\n</url>", e(u), e(lastmod)));
    }
    sm.push_str("\n</urlset>\n");
    write(&out.join("sitemap.xml"), &sm)?;

    // robots.txt
    write(&out.join("robots.txt"), &format!("User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n", base_url))?;

    // static assets
    write(&out.join("style.css"), STYLE_CSS)?;
    apply_base_path(out, &base_path(site.base_url))?;
    Ok(())
}
