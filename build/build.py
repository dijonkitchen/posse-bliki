"""Static site build for posse-bliki.

This module is the only place build logic lives. Keep it small. Behaviour
is defined by ``spec/`` and verified by ``tests/`` — that's the contract.
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime as _dt
import email.utils
import json
import re
import shutil
import sys
from pathlib import Path

import jinja2
import jsonschema
import yaml
from markdown_it import MarkdownIt


class _StringDatesLoader(yaml.SafeLoader):
    """SafeLoader that does NOT auto-convert ISO dates to ``datetime.date``.

    Per ``spec/content-schema.md``, front-matter dates are strings. Disabling
    the timestamp resolver keeps them as strings without requiring quoting.
    """


_StringDatesLoader.yaml_implicit_resolvers = {
    k: [(t, r) for t, r in v if t != "tag:yaml.org,2002:timestamp"]
    for k, v in _StringDatesLoader.yaml_implicit_resolvers.items()
}

REPO_ROOT = Path(__file__).resolve().parent.parent
TEMPLATES_DIR = Path(__file__).resolve().parent / "templates"
STATIC_DIR = Path(__file__).resolve().parent / "static"
SCHEMA_PATH = REPO_ROOT / "spec" / "content-schema.json"


class BuildError(RuntimeError):
    pass


def default_config() -> dict:
    """Site config. Edit before going live."""
    return {
        "title": "posse-bliki",
        "tagline": "A personal POSSE bliki.",
        "base_url": "https://example.com",
        "language": "en",
        "author": {
            "name": "Author Name",
            "url": "https://example.com",
            "rel_me": [
                "https://github.com/dijonkitchen",
            ],
        },
        "webmention": {
            "endpoint": "https://webmention.io/example.com/webmention",
            "pingback": "https://webmention.io/example.com/xmlrpc",
        },
    }


# --- data model ---


@dataclasses.dataclass
class Note:
    path: Path
    rel_path: Path
    slug: str
    kind: str            # "home" | "page" | "post"
    fm: dict
    body: str
    url: str = ""
    out_path: Path = dataclasses.field(default_factory=Path)
    content_html: str = ""
    backlinks: list["Note"] = dataclasses.field(default_factory=list)

    @property
    def title(self) -> str:
        return self.fm["title"]

    @property
    def date(self) -> _dt.date | None:
        v = self.fm.get("date")
        return _parse_date(v) if v else None

    @property
    def updated(self) -> _dt.date | None:
        v = self.fm.get("updated")
        return _parse_date(v) if v else None

    @property
    def tags(self) -> list[str]:
        return sorted(set(self.fm.get("tags", [])))

    @property
    def summary(self) -> str | None:
        return self.fm.get("summary")

    @property
    def syndication(self) -> list[str]:
        return list(self.fm.get("syndication", []))

    @property
    def aliases(self) -> list[str]:
        return list(self.fm.get("aliases", []))

    @property
    def draft(self) -> bool:
        return bool(self.fm.get("draft", False))


# --- parsing helpers ---


_FRONT_MATTER_RE = re.compile(r"\A---\n(.+?)\n---\n?(.*)\Z", re.DOTALL)
_TAG_PATTERN = re.compile(r"^[a-z0-9][a-z0-9-]*$")

# Match (and skip) fenced/inline code, OR a wikilink, OR an inline #tag.
# Code regions are pass-through so that ``[[foo]]`` and ``#foo`` examples in
# documentation don't trigger resolution.
_CODE_OR_WIKILINK_RE = re.compile(
    r"(?P<fence>```[\s\S]*?```)"
    r"|(?P<inline>`[^`\n]+`)"
    r"|\[\[(?P<wikitarget>[^\]\|]+?)(?:\|(?P<wikialt>[^\]]+))?\]\]"
)
_CODE_OR_TAG_RE = re.compile(
    r"(?P<fence>```[\s\S]*?```)"
    r"|(?P<inline>`[^`\n]+`)"
    r"|(?:^|(?<=\s))#(?P<tag>[a-z0-9][a-z0-9-]*)\b"
)


def _parse_date(v) -> _dt.date:
    if isinstance(v, _dt.datetime):
        return v.date()
    if isinstance(v, _dt.date):
        return v
    return _dt.date.fromisoformat(str(v)[:10])


def _slugify(stem: str) -> str:
    s = stem.lower().strip()
    s = re.sub(r"[^a-z0-9]+", "-", s)
    return s.strip("-")


def _split_front_matter(text: str) -> tuple[dict, str]:
    m = _FRONT_MATTER_RE.match(text)
    if not m:
        return {}, text
    fm = yaml.load(m.group(1), Loader=_StringDatesLoader) or {}
    if not isinstance(fm, dict):
        raise BuildError("front-matter is not a YAML mapping")
    return fm, m.group(2)


def _kind_for(rel_path: Path) -> str:
    if rel_path == Path("index.md"):
        return "home"
    if rel_path.parts[0] == "notes":
        return "post"
    return "page"


def _url_for(rel_path: Path, kind: str) -> str:
    if kind == "home":
        return "/"
    parts = list(rel_path.with_suffix("").parts)
    parts[-1] = _slugify(parts[-1])
    if kind == "page":
        return f"/{parts[-1]}/"
    # post: parts starts with "notes"
    return "/" + "/".join(parts) + "/"


def _out_path_for(out_dir: Path, url: str) -> Path:
    if url == "/":
        return out_dir / "index.html"
    return out_dir / url.strip("/") / "index.html"


def _resolve_wikilinks(text: str, slug_to_url: dict[str, str], context: str) -> str:
    def repl(m: re.Match) -> str:
        if m.group("fence") is not None or m.group("inline") is not None:
            return m.group(0)
        target = m.group("wikitarget").strip()
        alt = (m.group("wikialt") or "").strip()
        target_no_anchor = target.split("#", 1)[0]
        target_slug = _slugify(target_no_anchor.split("/")[-1])
        url = slug_to_url.get(target_slug)
        if not url:
            raise BuildError(
                f"{context}: unresolved wikilink [[{target}]] (slug {target_slug!r})"
            )
        anchor = "#" + _slugify(target.split("#", 1)[1]) if "#" in target else ""
        label = alt or target_no_anchor.split("/")[-1]
        return f"[{label}]({url}{anchor})"

    return _CODE_OR_WIKILINK_RE.sub(repl, text)


def _extract_inline_tags(text: str) -> tuple[str, list[str]]:
    found: list[str] = []

    def repl(m: re.Match) -> str:
        if m.group("fence") is not None or m.group("inline") is not None:
            return m.group(0)
        tag = m.group("tag")
        found.append(tag)
        return f"[#{tag}](/tags/{tag}/)"

    return _CODE_OR_TAG_RE.sub(repl, text), found


def _md() -> MarkdownIt:
    return MarkdownIt("gfm-like", {"html": False, "linkify": False, "typographer": False})


def _strip_html(html: str) -> str:
    return re.sub(r"<[^>]+>", "", html)


# --- build phases ---


def _load_notes(content_dir: Path, validator: jsonschema.Draft202012Validator) -> list[Note]:
    notes: list[Note] = []
    for path in sorted(content_dir.rglob("*.md")):
        rel = path.relative_to(content_dir)
        if any(part.startswith(".") for part in rel.parts):
            continue
        text = path.read_text(encoding="utf-8")
        try:
            fm, body = _split_front_matter(text)
        except BuildError as e:
            raise BuildError(f"{rel}: {e}") from None
        errs = sorted(validator.iter_errors(fm), key=lambda e: list(e.path))
        if errs:
            msg = "; ".join(f"{list(e.path)!r}: {e.message}" for e in errs)
            raise BuildError(f"{rel}: front-matter invalid: {msg}")
        kind = _kind_for(rel)
        slug = "index" if kind == "home" else _slugify(rel.stem)
        notes.append(Note(path=path, rel_path=rel, slug=slug, kind=kind, fm=fm, body=body))
    return notes


def _validate_unique_slugs(notes: list[Note]) -> None:
    seen: dict[str, Note] = {}
    for n in notes:
        if n.slug == "index":
            continue
        if n.slug in seen:
            raise BuildError(
                f"duplicate slug {n.slug!r}: {seen[n.slug].rel_path} vs {n.rel_path}"
            )
        seen[n.slug] = n


def _validate_tags(notes: list[Note]) -> None:
    for n in notes:
        for t in n.fm.get("tags", []):
            if not _TAG_PATTERN.match(t):
                raise BuildError(f"{n.rel_path}: invalid tag {t!r}")


def _render_markdown(notes: list[Note], slug_to_url: dict[str, str]) -> None:
    md = _md()
    for n in notes:
        body = _resolve_wikilinks(n.body, slug_to_url, str(n.rel_path))
        body, inline_tags = _extract_inline_tags(body)
        n.fm["tags"] = sorted(set(n.fm.get("tags", [])) | set(inline_tags))
        n.content_html = md.render(body)


def _compute_backlinks(notes: list[Note]) -> None:
    by_url = {n.url: n for n in notes}
    for n in notes:
        for url in re.findall(r'href="([^"#]+)(?:#[^"]*)?"', n.content_html):
            target = by_url.get(url)
            if target is None or target is n:
                continue
            if n not in target.backlinks:
                target.backlinks.append(n)
    for n in notes:
        n.backlinks.sort(key=lambda b: b.url)


def _env() -> jinja2.Environment:
    return jinja2.Environment(
        loader=jinja2.FileSystemLoader(str(TEMPLATES_DIR)),
        autoescape=jinja2.select_autoescape(["html", "xml"]),
        keep_trailing_newline=True,
        undefined=jinja2.StrictUndefined,
    )


def _write(p: Path, content: str) -> None:
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content, encoding="utf-8")


def _rfc822(d: _dt.date) -> str:
    dt = _dt.datetime.combine(d, _dt.time(0, 0), tzinfo=_dt.timezone.utc)
    return email.utils.format_datetime(dt)


def _iso_datetime(d: _dt.date) -> str:
    return _dt.datetime.combine(d, _dt.time(0, 0), tzinfo=_dt.timezone.utc).isoformat()


# --- main entry point ---


def build_site(content_dir: Path, out_dir: Path, config: dict) -> None:
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    jsonschema.Draft202012Validator.check_schema(schema)
    validator = jsonschema.Draft202012Validator(schema)

    notes = [n for n in _load_notes(content_dir, validator) if not n.draft]
    _validate_unique_slugs(notes)

    for n in notes:
        n.url = _url_for(n.rel_path, n.kind)
        n.out_path = _out_path_for(out_dir, n.url)

    slug_to_url = {n.slug: n.url for n in notes}
    _render_markdown(notes, slug_to_url)
    _validate_tags(notes)
    _compute_backlinks(notes)

    if out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True)

    env = _env()
    site = config
    base = site["base_url"].rstrip("/")
    canonical = lambda url: base + url

    posts = sorted(
        (n for n in notes if n.kind == "post"),
        key=lambda n: (n.date or _dt.date.min, n.slug),
        reverse=True,
    )

    # --- pages ---
    for n in notes:
        ctx = dict(
            site=site,
            page=n,
            canonical_url=canonical(n.url),
            posts=posts,
            backlinks=n.backlinks,
        )
        if n.kind == "home":
            tmpl = env.get_template("home.html")
        elif n.kind == "post":
            tmpl = env.get_template("post.html")
        else:
            tmpl = env.get_template("page.html")
        _write(n.out_path, tmpl.render(**ctx))

    # --- /notes/ list ---
    list_tmpl = env.get_template("list.html")
    _write(
        out_dir / "notes" / "index.html",
        list_tmpl.render(
            site=site,
            list_title="Notes",
            posts=posts,
            canonical_url=canonical("/notes/"),
        ),
    )

    # --- tag pages ---
    by_tag: dict[str, list[Note]] = {}
    for n in posts:
        for t in n.tags:
            by_tag.setdefault(t, []).append(n)
    for tag, tag_posts in sorted(by_tag.items()):
        url = f"/tags/{tag}/"
        _write(
            out_dir / "tags" / tag / "index.html",
            list_tmpl.render(
                site=site,
                list_title=f"#{tag}",
                posts=tag_posts,
                canonical_url=canonical(url),
            ),
        )

    # --- alias redirects ---
    redirect_tmpl = env.get_template("redirect.html")
    for n in notes:
        for alias in n.aliases:
            _write(
                out_dir / alias.strip("/") / "index.html",
                redirect_tmpl.render(
                    site=site,
                    target=n.url,
                    canonical_url=canonical(n.url),
                ),
            )

    # --- 404 ---
    nf_tmpl = env.get_template("404.html")
    _write(out_dir / "404.html", nf_tmpl.render(site=site, canonical_url=None))

    # --- RSS ---
    rss_tmpl = env.get_template("feed.xml")
    rss_items = []
    for p in posts:
        excerpt = _strip_html(p.content_html)[:200]
        rss_items.append(
            dict(
                title=p.title,
                url=p.url,
                date_rfc822=_rfc822(p.date or _dt.date(1970, 1, 1)),
                summary=p.summary or "",
                content_text_excerpt=excerpt,
            )
        )
    _write(out_dir / "index.xml", rss_tmpl.render(site=site, posts=rss_items))

    # --- JSON Feed ---
    feed = {
        "version": "https://jsonfeed.org/version/1.1",
        "title": site["title"],
        "home_page_url": base + "/",
        "feed_url": base + "/feed.json",
        "language": site["language"],
        "authors": [{"name": site["author"]["name"], "url": site["author"]["url"]}],
        "items": [
            {
                "id": canonical(p.url),
                "url": canonical(p.url),
                "title": p.title,
                "content_html": p.content_html,
                "date_published": _iso_datetime(p.date or _dt.date(1970, 1, 1)),
                **({"summary": p.summary} if p.summary else {}),
                "tags": p.tags,
            }
            for p in posts
        ],
    }
    _write(out_dir / "feed.json", json.dumps(feed, indent=2) + "\n")

    # --- sitemap ---
    sitemap_tmpl = env.get_template("sitemap.xml")
    urls = [
        dict(
            url=canonical(n.url),
            lastmod=(n.updated or n.date or _dt.date(1970, 1, 1)).isoformat(),
        )
        for n in sorted(notes, key=lambda x: x.url)
    ]
    urls.append(dict(url=canonical("/notes/"), lastmod=_dt.date(1970, 1, 1).isoformat()))
    for tag in sorted(by_tag):
        urls.append(dict(url=canonical(f"/tags/{tag}/"), lastmod=_dt.date(1970, 1, 1).isoformat()))
    _write(out_dir / "sitemap.xml", sitemap_tmpl.render(site=site, urls=urls))

    # --- robots.txt ---
    _write(
        out_dir / "robots.txt",
        f"User-agent: *\nAllow: /\n\nSitemap: {base}/sitemap.xml\n",
    )

    # --- static assets ---
    if STATIC_DIR.exists():
        for s in sorted(STATIC_DIR.rglob("*")):
            if s.is_file():
                rel = s.relative_to(STATIC_DIR)
                _write(out_dir / rel, s.read_text(encoding="utf-8"))


# --- CLI ---


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="build", description="Build the posse-bliki site.")
    parser.add_argument("--content", default=str(REPO_ROOT / "content"))
    parser.add_argument("--out", default=str(REPO_ROOT / "public"))
    parser.add_argument("--serve", action="store_true", help="Serve and watch for changes.")
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args(argv)

    content = Path(args.content)
    out = Path(args.out)
    cfg = default_config()
    try:
        build_site(content, out, cfg)
    except BuildError as e:
        print(f"build error: {e}", file=sys.stderr)
        return 1
    print(f"built {len(list(out.rglob('*.html')))} pages → {out}")

    if args.serve:
        _serve(out, args.port, content, cfg)
    return 0


def _serve(out: Path, port: int, content: Path, cfg: dict) -> None:
    import functools
    import http.server
    import threading

    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(out))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", port), handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    print(f"serving at http://127.0.0.1:{port}/")
    try:
        from watchfiles import watch

        for _ in watch(content):
            try:
                build_site(content, out, cfg)
                print("rebuilt")
            except BuildError as e:
                print(f"build error: {e}", file=sys.stderr)
    except KeyboardInterrupt:
        pass
    finally:
        server.shutdown()


if __name__ == "__main__":
    raise SystemExit(main())
