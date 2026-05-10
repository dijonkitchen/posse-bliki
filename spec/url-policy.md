# URL policy

URLs are forever. Changes here require a redirect from every old URL — see
`aliases` in [content-schema.md](content-schema.md).

## Permalink rules

| Source | URL |
|---|---|
| `content/index.md` | `/` |
| `content/<slug>.md` | `/<slug>/` |
| `content/notes/<slug>.md` | `/notes/<slug>/` |
| `content/notes/<sub>/<slug>.md` | `/notes/<sub>/<slug>/` |

- Slug = filename stem, lowercased, spaces → `-`. Filenames should already
  be in this form; the build will warn but not fail on auto-slugged names.
- Always trailing slash. Each page is emitted as `<url>/index.html`.
- No `.html` in URLs.
- No date prefixes. (We have `dt-published` and feeds for chronology.)

## Special pages

| URL | Source |
|---|---|
| `/index.xml` | RSS 2.0 feed |
| `/feed.json` | JSON Feed v1.1 |
| `/sitemap.xml` | Sitemap |
| `/robots.txt` | Generated |
| `/tags/<tag>/` | Tag list page |
| `/notes/` | Notes list page |
| `/404.html` | 404 page |

## Aliases

Aliases produce a tiny HTML page with `<meta http-equiv=refresh>` AND
`<link rel=canonical>` pointing to the canonical URL. This works on any
static host (no `.htaccess`/`_redirects` required) and is robot-friendly.

Once an alias is added, **never remove it**.
