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

The directory determines the kind. No `type:` field needed in front-matter.

## Front-matter fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `title` | string | yes | Maps to `<title>` and `p-name`. |
| `date` | ISO-8601 date | yes for posts | Maps to `dt-published`. Pages may omit. |
| `updated` | ISO-8601 date | no | Maps to `dt-updated`. |
| `summary` | string | no | Maps to `p-summary` and `<meta name=description>`. |
| `tags` | string[] | no | Maps to `p-category` per tag. |
| `draft` | bool | no | If `true`, excluded from build entirely. |
| `aliases` | string[] | no | Old slugs that should redirect to this page. |
| `syndication` | url[] | no | URLs of POSSE copies. Maps to `u-syndication`. |

Unknown fields are an error — fail loudly, fix in the schema if you want a new one.

## Date format

Always `YYYY-MM-DD` (or full ISO-8601 with time). Obsidian's daily-note format
is fine. Strings only — no implicit YAML date types, since their stringification
varies by parser.

## Wikilinks

Body content uses Obsidian wikilinks: `[[note-name]]` or `[[note-name|alt text]]`.

A wikilink resolves by **filename stem** across the entire vault. Stems must be
unique (the build fails if two notes have the same stem). Unresolved wikilinks
are a build error — no silent broken links.

## Tags

Two ways to tag, both supported, both equivalent:

- Front-matter: `tags: [foo, bar]`
- Inline body: `#foo #bar` (Obsidian-style)

Inline tags are extracted to the same list as front-matter tags during build.
