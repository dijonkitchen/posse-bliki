"""HTML and XML templates as plain functions.

Every interpolated value goes through ``e()`` unless it is already HTML
(rendered note bodies). The layout mirrors the markup contract in
``spec/output-contract.md`` and ``spec/indieweb.md``.
"""
from __future__ import annotations

__all__ = ["e", "post", "page", "home", "listing", "redirect", "not_found", "rss", "sitemap"]


def e(value) -> str:
    """Escape for HTML/XML text and attribute values."""
    return (
        str(value)
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&#34;")
        .replace("'", "&#39;")
    )


def _base(site: dict, title: str, meta: str, canonical_url: str | None, content: str) -> str:
    canonical = f'<link rel="canonical" href="{e(canonical_url)}">' if canonical_url else ""
    t = e(site["title"])
    return (
        "<!DOCTYPE html>\n"
        f'<html lang="{e(site["language"])}">\n'
        "<head>\n"
        '<meta charset="utf-8">\n'
        '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
        f"<title>{title}</title>\n"
        f"{meta}\n"
        f"{canonical}\n"
        f'<link rel="alternate" type="application/rss+xml" title="{t}" href="/index.xml">\n'
        f'<link rel="alternate" type="application/feed+json" title="{t}" href="/feed.json">\n'
        f'<link rel="webmention" href="{e(site["webmention"]["endpoint"])}">\n'
        f'<link rel="pingback" href="{e(site["webmention"]["pingback"])}">\n'
        '<link rel="stylesheet" href="/style.css">\n'
        "</head>\n"
        "<body>\n"
        '<header class="site-header">\n'
        "<nav>\n"
        f'<a href="/">{t}</a>\n'
        '<a href="/notes/">notes</a>\n'
        '<a href="/about/">about</a>\n'
        '<a href="/colophon/">colophon</a>\n'
        "</nav>\n"
        "</header>\n"
        "<main>\n"
        f"{content}\n"
        "</main>\n"
        '<footer class="site-footer">\n'
        "<p>\n"
        '<a href="/index.xml">RSS</a> ·\n'
        '<a href="/feed.json">JSON Feed</a> ·\n'
        '<a href="/colophon/">Colophon</a>\n'
        "</p>\n"
        "</footer>\n"
        "</body>\n"
        "</html>\n"
    )


def _titled(page, site: dict) -> str:
    return f"{e(page.title)} — {e(site['title'])}"


def _description(page) -> str:
    return f'<meta name="description" content="{e(page.summary)}">' if page.summary else ""


def _backlinks(backlinks) -> str:
    if not backlinks:
        return ""
    items = "".join(f'<li><a href="{e(b.url)}">{e(b.title)}</a></li>' for b in backlinks)
    return (
        '\n<aside class="backlinks">\n'
        "<h2>Linked from</h2>\n"
        "<ul>\n"
        f"{items}\n"
        "</ul>\n"
        "</aside>\n"
    )


def post(site: dict, page, canonical_url: str, backlinks) -> str:
    updated = ""
    if page.updated:
        u = page.updated.isoformat()
        updated = f' · <time class="dt-updated" datetime="{u}">updated {u}</time>'
    summary = f'<p class="p-summary">{e(page.summary)}</p>' if page.summary else ""
    tags = ""
    if page.tags:
        links = " ".join(
            f'<a class="p-category" href="/tags/{e(t)}/">#{e(t)}</a>' for t in page.tags
        )
        tags = f'<p class="tags">{links}</p>'
    syndication = ""
    if page.syndication:
        links = ", ".join(f'<a class="u-syndication" href="{e(s)}">{e(s)}</a>' for s in page.syndication)
        syndication = f"\n<footer>\n<p>Also on: {links}</p>\n</footer>\n"
    d = page.date.isoformat()
    content = (
        "\n"
        '<article class="h-entry">\n'
        "<header>\n"
        f'<h1 class="p-name">{e(page.title)}</h1>\n'
        '<p class="meta">\n'
        f'<a class="u-url u-uid" href="{e(canonical_url)}"><time class="dt-published" datetime="{d}">{d}</time></a>\n'
        f"{updated}\n"
        f' · <span class="p-author h-card"><a class="u-url p-name" rel="author" href="{e(site["base_url"])}/">{e(site["author"]["name"])}</a></span>\n'
        "</p>\n"
        f"{summary}\n"
        f"{tags}\n"
        "</header>\n"
        '<div class="e-content">\n'
        f"{page.content_html}\n"
        "</div>\n"
        f"{syndication}\n"
        "</article>\n"
        f"{_backlinks(backlinks)}\n"
    )
    return _base(site, _titled(page, site), _description(page), canonical_url, content)


def page(site: dict, page, canonical_url: str, backlinks) -> str:
    content = (
        "\n"
        "<article>\n"
        f"<h1>{e(page.title)}</h1>\n"
        f"{page.content_html}\n"
        "</article>\n"
        f"{_backlinks(backlinks)}\n"
    )
    return _base(site, _titled(page, site), _description(page), canonical_url, content)


def home(site: dict, page, canonical_url: str, posts) -> str:
    base = e(site["base_url"])
    rel_me = "".join(
        f'\n<a class="u-url" rel="me" href="{e(u)}">{e(u)}</a>\n' for u in site["author"]["rel_me"]
    )
    recent = "".join(
        "\n"
        '<li class="h-entry">\n'
        f'<a class="u-url p-name" href="{e(p.url)}">{e(p.title)}</a>\n'
        f'<time class="dt-published" datetime="{p.date.isoformat()}">{p.date.isoformat()}</time>\n'
        "</li>\n"
        for p in posts[:10]
    )
    content = (
        "\n"
        '<section class="h-card">\n'
        "<p>\n"
        f'<span class="p-name">{e(site["author"]["name"])}</span>\n'
        f'<a class="u-url u-uid" rel="me" href="{base}/">{base}/</a>\n'
        f"{rel_me}\n"
        "</p>\n"
        "</section>\n"
        "<article>\n"
        f"{page.content_html}\n"
        "</article>\n"
        '<section class="h-feed">\n'
        '<h2 class="p-name">Recent posts</h2>\n'
        "<ul>\n"
        f"{recent}\n"
        "</ul>\n"
        "</section>\n"
    )
    return _base(site, e(site["title"]), "", canonical_url, content)


def listing(site: dict, list_title: str, posts, canonical_url: str) -> str:
    items = "".join(
        "\n"
        '<li class="h-entry">\n'
        f'<a class="u-url p-name" href="{e(p.url)}">{e(p.title)}</a>\n'
        f'<time class="dt-published" datetime="{p.date.isoformat()}">{p.date.isoformat()}</time>\n'
        + (f' — <span class="p-summary">{e(p.summary)}</span>' if p.summary else "")
        + "\n</li>\n"
        for p in posts
    )
    content = (
        "\n"
        '<section class="h-feed">\n'
        f'<h1 class="p-name">{e(list_title)}</h1>\n'
        "<ul>\n"
        f"{items}\n"
        "</ul>\n"
        "</section>\n"
    )
    return _base(site, f"{e(list_title)} — {e(site['title'])}", "", canonical_url, content)


def redirect(site: dict, target: str, canonical_url: str) -> str:
    meta = (
        "\n"
        f'<meta http-equiv="refresh" content="0; url={e(target)}">\n'
        '<meta name="robots" content="noindex">\n'
    )
    content = f'\n<p>This page has moved to <a href="{e(target)}">{e(target)}</a>.</p>\n'
    return _base(site, "Redirecting…", meta, canonical_url, content)


def not_found(site: dict) -> str:
    content = (
        "\n"
        "<article>\n"
        "<h1>Not found</h1>\n"
        "<p>That page isn't here. Try the <a href=\"/\">home page</a> or the "
        '<a href="/notes/">notes index</a>.</p>\n'
        "</article>\n"
    )
    return _base(site, f"Not found — {e(site['title'])}", "", None, content)


def rss(site: dict, items: list[dict]) -> str:
    base = e(site["base_url"])
    out = [
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">\n'
        "<channel>\n"
        f"<title>{e(site['title'])}</title>\n"
        f"<link>{base}/</link>\n"
        f"<description>{e(site['tagline'])}</description>\n"
        f"<language>{e(site['language'])}</language>\n"
        f'<atom:link href="{base}/index.xml" rel="self" type="application/rss+xml"/>'
    ]
    if items and items[0]["date_rfc822"]:
        out.append(f"\n<lastBuildDate>{e(items[0]['date_rfc822'])}</lastBuildDate>")
    for p in items:
        link = f"{base}{e(p['url'])}"
        description = p["summary"] or p["content_text_excerpt"]
        out.append(
            "\n<item>\n"
            f"<title>{e(p['title'])}</title>\n"
            f"<link>{link}</link>\n"
            f'<guid isPermaLink="true">{link}</guid>\n'
            f"<pubDate>{e(p['date_rfc822'])}</pubDate>\n"
            f"<description>{e(description)}</description>\n"
            "</item>"
        )
    out.append("\n</channel>\n</rss>\n")
    return "".join(out)


def sitemap(urls: list[dict]) -> str:
    entries = "".join(
        f"\n<url>\n<loc>{e(u['url'])}</loc>\n<lastmod>{e(u['lastmod'])}</lastmod>\n</url>"
        for u in urls
    )
    return (
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'
        f"{entries}\n"
        "</urlset>\n"
    )
