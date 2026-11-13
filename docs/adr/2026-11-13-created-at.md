# 2026-11-13 — `created_at`, the day a note was made

- Date: 2026-11-13
- Status: Accepted
- Decided by: JC:
  "still think created_at and updated_at are best.
  former for when it was made and fallback for date on notes.
  latter that auto-updates on save and is the sort order for notes that can be updated over time."
  Claude had recommended keeping `date`, then offered `published_at`,
  which JC found a poor fit for drafts and for posts meant to show up in the future.

## Context

ADR 2026-11-12-updated-at-utc-datetimes renamed `updated` to `updated_at`.
That left `date` beside it, a day typed by hand,
which makes a note a post, dates it on the page and in the feeds,
and holds it back while it is in the future
(ADR 2026-10-20-scheduled-posts).
JC wanted the two names to pair.

Three names were weighed:

- `date` is what [Jekyll](https://jekyllrb.com/docs/front-matter/) and Hugo use,
  but it doesn't say which date.
- `published_at` says what the feeds and microformats call the field
  (RSS `pubDate`, JSON Feed `date_published`, `dt-published`),
  and Ghost schedules a post by
  "setting `published_at` to a datetime in the future"
  ([Ghost Admin API](https://docs.ghost.org/admin-api/posts/scheduling-a-post)).
  JC found it a poor fit for drafts, which aren't published,
  and for posts meant to show up later.
- `created_at` says when the note was made,
  which is true of a draft and of a scheduled post alike.
  Hugo's own `date` is "typically the creation date"
  ([Hugo: Front matter](https://gohugo.io/content-management/front-matter/)).

## Decision

- **The field is `created_at`.**
  `date` is no longer a field,
  and the build stops on it with "`date` is now `created_at`".
  All 67 notes were renamed, values unchanged.
- **It does everything `date` did.**
  It makes a note a post, dates the post on the page and in the feeds,
  and holds a post back until its day comes.
  Lists sort by a note's update, or else by `created_at`,
  as JC described.
- **It is a day, typed by hand.**
  A time after it fails the build, where `date` let one through and ignored it.
  The site rebuilds once a day,
  so a day is all the build can act on for scheduling.
- **Obsidian doesn't stamp it.**
  The Linter plugin's created date comes from the file's creation time on disk,
  and git keeps no file times,
  so a note would be dated to the day it reached each device: the clone, for every note.
  Stamping would also give every evergreen note and page a `created_at`,
  and a `created_at` makes an evergreen note a post that goes to the feeds.

## Consequences

- The built site is unchanged.
- A promoted note or an old draft that still has `date` fails the build
  with a message naming the new field.
- An evergreen note is still one with `updated_at` and no `created_at`.
  If every note should carry `created_at`,
  the feeds will need another way to tell a post from an evergreen note.
- `tests/harness/evergreen.rs` and `tests/harness/frontmatter_schema.rs` check
  the old name, the day-only form and that the schema and build agree.

## Alternatives considered

- **Keep `date`.**
  The convention Jekyll, Hugo and Obsidian share, and no rename,
  but it doesn't pair with `updated_at`.
- **`published_at`.**
  Accurate for the feeds, and what Ghost uses to schedule,
  but JC preferred a name that reads true for drafts and future posts.
- **Both `created_at` and `published_at`.**
  Two dates to keep for every post, where nothing here tells them apart.
- **Let `created_at` take a UTC time, like `updated_at`.**
  Possible later. Until the site rebuilds more than daily,
  a time could only delay a post to the next build.
