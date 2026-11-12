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
| `content/notes/**/*.md` | post, or an evergreen note (see below) | `post.html` | yes |

Images go in `content/images/` and are linked as `/images/<file>`
(see [performance.md](performance.md)).

The directory determines the kind. No `type:` field needed in front-matter.

`content/now.md`, when published, is linked from every page's header, and
`content/follow.md` and `content/blogroll.md` from every page's footer
(ADR 2026-11-05-now-in-the-header, ADR 2026-11-05-follow-with-a-feed-reader). The now page must have `updated_at` (the build fails
without it), shown under its title as "Updated <date>", since a now page
is only as good as its date (ADR 2026-11-01-now-page).

`TODO(JC)` marks a placeholder still to fill in. No published page may
contain one; keep the note `draft: true` until it's filled in.

## Front-matter fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `title` | string | yes | Maps to `<title>` and `p-name`. |
| `date` | ISO-8601 date | yes for posts | Maps to `dt-published`. Pages and evergreen notes omit it. |
| `updated_at` | UTC datetime, or a day | yes for evergreen notes | Maps to `dt-updated`. Moves the note up Home and Notes. See [`updated_at`](#updated_at). |
| `summary` | string | no | Maps to `p-summary` and `<meta name=description>` (else an excerpt of the text is used). |
| `draft` | bool | no | If `true`, excluded from build entirely. |
| `aliases` | string[] | no | Old slugs that should redirect to this page. |
| `syndication` | url[] | no | URLs of POSSE copies. Maps to `u-syndication`. |

Unknown fields are an error — fail loudly, fix in the schema if you want a new one.
There is no `tags` field: notes relate through links, not tags
(ADR 2026-11-06-drop-tags).
Remove it from a note promoted from Obsidian.

## Evergreen notes and the order of lists

A note in `content/notes/` needs `date`, `updated_at`, or both
(ADR 2026-11-07-evergreen-notes):

- With `date`, it is a post:
  listed on Home and `/notes/`, and carried by the RSS and JSON feeds,
  so the deploy syndicates it.
- With `updated_at` and no `date`, it is an evergreen note:
  listed on Home and `/notes/` like a post, but in no feed, so never syndicated.
  Its page has `dt-updated` ("Updated <date>") and no `dt-published`.
  It is held back until the UTC day of its `updated_at`, like a scheduled post.
- With neither, the build fails.

Home and `/notes/` list notes most recently tended first:
by a note's update, or else its `date`, then by slug.
Each entry shows its publish date,
then " · updated <date>" when the note has been updated since;
an evergreen note shows only "Updated <date>".
The feeds stay in `date` order.

## `updated_at`

`updated_at` says when a note was last reworked
(ADR 2026-11-12-updated-at-utc-datetimes).

- It is a UTC datetime, `2026-11-12T04:27:28Z`,
  or a bare day, `2026-11-12`.
  The seconds are optional, and `+00:00` may stand in for `Z`.
  A time in any other zone, or with no zone, fails the build:
  the build has no time zone rules, and only UTC says when without them.
- Readers see its UTC day, in the Holocene calendar like every date on the page.
  The `datetime` of its `<time>` carries the value as written,
  and the sitemap's `<lastmod>` its day.
- When two notes are compared, a day with no time stands for noon UTC,
  for `updated_at` and `date` alike.
- A post counts as updated once its `updated_at` is at or after noon UTC
  on the day after its `date`.
  `date` is a day in no time zone,
  and by then that day is over everywhere on Earth.
  So a save on the evening a post goes out in the Americas,
  which is already the next day in UTC,
  is part of publishing it, as is any save made while it was being written.
  It shows no "updated" and moves nothing.
  For a bare day this comes to any day after `date`.
- The vault's Obsidian settings stamp it:
  with the Linter plugin installed,
  saving a note with Ctrl+S writes the current UTC time
  (ADR 2026-11-12-obsidian-stamps-updated-at).
  Anywhere else, write it by hand when the text is reworked,
  and leave it alone for a typo or a formatting fix.
  The build never reads git dates.
- The field used to be `updated`.
  The build stops on the old name and says the new one.

## Scheduled posts

A note whose `date` is after today (UTC, or `--today`) is left out of the
build exactly like a draft: no page, no listing, no feed item, no sitemap entry, no alias, no webmention. The first build on or after
its date publishes it; the deploy workflow rebuilds daily, so it goes live
that morning (UTC) with no push (ADR 2026-10-20-scheduled-posts). A link to a
scheduled note from a published one is a broken link, as for a draft.

## Date format

`date` is always `YYYY-MM-DD` (a time after it is allowed and ignored). Obsidian's daily-note format
is fine. `updated_at` has its own form, [above](#updated_at). Strings only — no implicit YAML date types, since their stringification
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
