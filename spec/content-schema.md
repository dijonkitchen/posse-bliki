# Content schema

Every file in `content/` is markdown with YAML front-matter.

The machine-readable schema lives at [`content-schema.json`](content-schema.json)
(JSON Schema, draft 2020-12). Tests validate every note against it.

## Locations

| Path | Kind | Template | h-entry? |
|---|---|---|---|
| `content/index.md` | home | `home.html` | no (h-card instead) |
| `content/<page>.md` | page | `page.html` | no |
| `content/blogroll.md` | page, also `/blogroll.opml` | `page.html` | no (see [feeds.md](feeds.md)) |
| `content/now.md` | page, a [now page](https://nownownow.com/about) | `page.html` | no (see below) |
| `content/follow.md` | page, how to follow in a feed reader | `page.html` | no (see [feeds.md](feeds.md)) |
| `content/bio.md` | page, the work bio; opens with the author's photo | `page.html` | no (ADR 2026-11-06-h-card-photo) |
| `content/notes/**/*.md` | post | `post.html` | yes |

Images go in `content/images/` and are linked as `/images/<file>`
(see [performance.md](performance.md)).

The directory determines the kind. No `type:` field needed in front-matter.

`content/now.md`, when published, is linked from every page's header, and
`content/follow.md` and `content/blogroll.md` from every page's footer
(ADR 2026-11-05-now-in-the-header, ADR 2026-11-05-follow-with-a-feed-reader). The now page must have `updated` (the build fails
without it), shown under its title as "Updated <date>", since a now page
is only as good as its date (ADR 2026-11-01-now-page).

`TODO(JC)` marks a placeholder still to fill in. No published page may
contain one; keep the note `draft: true` until it's filled in.

## Front-matter fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `title` | string | yes | Maps to `<title>` and `p-name`. |
| `date` | ISO-8601 date | yes for posts | Maps to `dt-published`. Pages may omit. |
| `updated` | ISO-8601 date | no | Maps to `dt-updated`. |
| `summary` | string | no | Maps to `p-summary` and `<meta name=description>` (else an excerpt of the text is used). |
| `draft` | bool | no | If `true`, excluded from build entirely. |
| `aliases` | string[] | no | Old slugs that should redirect to this page. |
| `syndication` | url[] | no | URLs of POSSE copies. Maps to `u-syndication`. |

Unknown fields are an error — fail loudly, fix in the schema if you want a new one.
There is no `tags` field: notes relate through links, not tags
(ADR 2026-11-06-drop-tags).
Remove it from a note promoted from Obsidian.

## Scheduled posts

A note whose `date` is after today (UTC, or `--today`) is left out of the
build exactly like a draft: no page, no listing, no feed item, no sitemap entry, no alias, no webmention. The first build on or after
its date publishes it; the deploy workflow rebuilds daily, so it goes live
that morning (UTC) with no push (ADR 2026-10-20-scheduled-posts). A link to a
scheduled note from a published one is a broken link, as for a draft.

## Posts first published elsewhere

A post JC first published on another site, such as 𝕏, Threads or Bluesky,
comes over as an ordinary note
(ADR 2026-11-07-silo-posts-as-notes).
The IndieWeb calls this [PESOS](https://indieweb.org/PESOS).

- Only JC's own threads and posts that still hold up come over,
  picked by hand.
  Reposts and replies to other people stay where they are.
- A thread becomes one note, its posts joined into paragraphs.
- The note gets a `title`, like any post.
- `date` is the day the original was posted,
  so the note sorts among the notes of its time,
  not at the top of the home page.
- `syndication` lists the original's URL,
  so the page links it under "Also on"
  ([u-syndication](https://indieweb.org/u-syndication)).
- Shortened links (`t.co` and the like) are replaced by the address they lead to,
  since the shortener may not outlive the site that made it.
- A post that only links to a note already here isn't brought over.
  Its URL goes in that note's `syndication` instead.
- A few at a time:
  no more than 5 new notes in one deploy,
  so feed readers get a handful, not a flood.
  The build doesn't send webmentions for them (see [indieweb.md](indieweb.md)).

## Date format

Always `YYYY-MM-DD` (or full ISO-8601 with time). Obsidian's daily-note format
is fine. Strings only — no implicit YAML date types, since their stringification
varies by parser.

## No raw HTML

The build escapes HTML in notes, so a tag like `<iframe>` shows as text.
Notes are Markdown only, and a raw tag in a note's prose (outside code) fails
the harness. Videos are links to the video's page, not embeds
(ADR 2026-10-28-videos-as-links).

## Emailing the author

A link to `mailto:` with no address, `[email me](mailto:)`, links to the
author's address, which the build writes in HTML character codes so the
address never appears in the repo or as plain text on the site
(ADR 2026-11-06-coded-email).

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

## No tags

The site has no tags (ADR 2026-11-06-drop-tags).
A `#word` in a note's text is plain text, not a tag,
and the front-matter has no `tags` field.
To relate two notes, link one to the other.
