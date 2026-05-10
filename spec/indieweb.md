# IndieWeb standards supported

Conformance to each item is enforced by a test in `tests/`.

## Built in

| Standard | Spec | Where |
|---|---|---|
| **microformats2 h-entry** | <https://microformats.org/wiki/h-entry> | every post page |
| **microformats2 h-card** | <https://microformats.org/wiki/h-card> | home page |
| **microformats2 h-feed** | <https://microformats.org/wiki/h-feed> | home, `/notes/`, `/tags/<tag>/` |
| **rel=me** | <https://microformats.org/wiki/rel-me> | home page, footer |
| **rel=author** | <https://microformats.org/wiki/rel-author> | post pages |
| **rel=canonical** | every page |
| **u-syndication** | <https://indieweb.org/u-syndication> | post pages with `syndication:` front-matter |
| **RSS 2.0** | every page advertises via `<link rel=alternate>` |
| **JSON Feed 1.1** | every page advertises via `<link rel=alternate>` |
| **Sitemap** | `/sitemap.xml`, advertised in `robots.txt` |

## Wired up via external services (no server needed)

| Standard | Service | Configuration |
|---|---|---|
| **Webmention** (receive) | <https://webmention.io> | `<link rel=webmention>` on every page |
| **Pingback** (receive) | <https://webmention.io> | `<link rel=pingback>` on every page |
| **Webmention** (send) | <https://telegraph.p3k.io> or <https://brid.gy> | manual or via build hook |
| **ActivityPub** | <https://fed.brid.gy> | follow `@<domain>@<domain>` after Bridgy Fed setup |
| **WebSub** | <https://websub.rocks/hub> | optional `<link rel=hub>` if enabled |

## Out of scope (deliberately)

- **Micropub** server — would require a dynamic backend; not worth it for a
  static site with Obsidian as the editor. If browser/mobile posting becomes
  important, run a Micropub-to-git bridge (e.g. `quill` → GitHub commit).
- **IndieAuth** server — not needed unless we run Micropub. Use
  <https://indielogin.com> as a relying-party-only flow if logging in
  somewhere is required; the home `h-card` + `rel=me` is enough for now.

## Verifying conformance

```sh
uv run pytest tests/test_microformats.py tests/test_indieweb_links.py
```

External validators (run manually before major releases):

- <https://indiewebify.me/>
- <https://pin13.net/mf2/>
- <https://validator.w3.org/feed/>
