# Content schema

Every file in `content/` is markdown with YAML front-matter.

The machine-readable schema lives at [`content-schema.json`](content-schema.json)
(JSON Schema, draft 2020-12). Tests validate every note against it.

## Locations

| Path | Kind | Template | h-entry? |
|---|---|---|---|
| `content/index.md` | home | `home.html` | no (h-card instead) |
| `content/<page>.md` | page | `page.html` | no |
| `content/notes/**/*.md` | post | `post.html` | yes |

Images go in `content/images/` and are linked as `/images/<file>`
(see [performance.md](performance.md)).

The directory determines the kind. No `type:` field needed in front-matter.

## Front-matter fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `title` | string | yes | Maps to `<title>` and `p-name`. |
| `date` | ISO-8601 date | yes for posts | Maps to `dt-published`. Pages may omit. |
| `updated` | ISO-8601 date | no | Maps to `dt-updated`. |
| `summary` | string | no | Maps to `p-summary` and `<meta name=description>` (else an excerpt of the text is used). |
| `tags` | string[] | no | Maps to `p-category` per tag. |
| `draft` | bool | no | If `true`, excluded from build entirely. |
| `aliases` | string[] | no | Old slugs that should redirect to this page. |
| `syndication` | url[] | no | URLs of POSSE copies. Maps to `u-syndication`. |

Unknown fields are an error — fail loudly, fix in the schema if you want a new one.

## Scheduled posts

A note whose `date` is after today (UTC, or `--today`) is left out of the
build exactly like a draft: no page, no listing, no tag page entry, no feed
item, no sitemap entry, no alias, no webmention. The first build on or after
its date publishes it; the deploy workflow rebuilds daily, so it goes live
that morning (UTC) with no push (ADR 2026-10-20-scheduled-posts). A link to a
scheduled note from a published one is a broken link, as for a draft.

## Date format

Always `YYYY-MM-DD` (or full ISO-8601 with time). Obsidian's daily-note format
is fine. Strings only — no implicit YAML date types, since their stringification
varies by parser.

## Links between notes

Notes link with standard Markdown links to the file, relative to the linking
note: `[text](other-note.md)`, `[text](../about.md#heading)`. The same path
works on GitHub, in Obsidian and on the site (ADR 2026-10-16-relative-markdown-links).

The build rewrites each relative `.md` href to the linked note's URL, with a
`#heading` slugged the same way wikilink anchors are. A link to a file that isn't a published
note (missing, a draft, or outside `content/`) is a build error — no silent
broken links.

Content in this repo uses no wikilinks and no site-URL links (`/other-note`)
between notes; the harness fails on either.

### Wikilinks (promoted notes)

The build still resolves Obsidian wikilinks, `[[note-name]]` or
`[[note-name|alt text]]`, so a note promoted from a wikilink vault builds.
A wikilink resolves by **filename stem** across the entire vault. Stems must be
unique (the build fails if two notes have the same stem). Unresolved wikilinks
are a build error. Convert them to Markdown links before committing.

## Tags

Two ways to tag, both supported, both equivalent:

- Front-matter: `tags: [foo, bar]`
- Inline body: `#foo #bar` (Obsidian-style)

Inline tags are extracted to the same list as front-matter tags during build.
