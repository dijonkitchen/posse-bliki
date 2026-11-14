# 2026-11-13 — `created_at`, when a note was made

- Date: 2026-11-13
- Status: Accepted
- Decided by: JC:
  "still think created_at and updated_at are best.
  former for when it was made and fallback for date on notes.
  latter that auto-updates on save and is the sort order for notes that can be updated over time."
  Claude had recommended keeping `date`, then offered `published_at`,
  which JC found a poor fit for drafts and for posts meant to show up in the future.
  On a first draft that kept it a day, JC: "created_at soudl be a datetime".
  On a draft where only `updated_at` was stamped, JC, on 2026-11-14:
  "all notes are evergreen no? [...]
  so shouldn't linter always automatically add created_at and updated_at for all of these?"
  Claude had recommended typing `created_at` by hand, to keep evergreen notes possible.

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
- **It is a UTC datetime or a day,**
  in the forms `updated_at` takes (ADR 2026-11-12-updated-at-utc-datetimes):
  `2026-11-13T05:19:33Z`, seconds optional, `+00:00` for `Z`,
  or a bare day such as `2026-11-13`.
  Any other zone fails the build, as for `updated_at`.
  The 67 notes keep their bare days; no time is made up for them.
  The page shows the UTC day,
  the `datetime` of its `<time>` carries the value as written,
  and the feeds carry the time (a bare day as midnight UTC).
  Two posts on one day sort by their times.
- **A timed post goes live at its moment.**
  The build is held back by a moment, not a day:
  a bare day comes at the start of that day in UTC,
  a datetime at that time,
  and the post goes out with the first build after it,
  a push or the daily 06:17 UTC rebuild.
  An evergreen note waits for its `updated_at` the same way.
  `--today` takes a datetime to preview a cutoff;
  a bare day there stands for the end of that day.
- **A save in the 24 hours after `created_at` isn't an update.**
  ADR 2026-11-12-updated-at-utc-datetimes counted a post as updated
  from noon UTC the day after its day.
  With a time to go by, that becomes 24 hours after `created_at`,
  and a bare day still stands for noon UTC, so it ends at the same moment as before.
- **Obsidian stamps it.**
  The vault's Linter settings
  (ADR 2026-11-12-obsidian-stamps-updated-at)
  turn on the YAML Timestamp rule's created date, under the key `created_at`,
  with the note's front matter as its source of truth.
  The first time Linter stamps a note in `content/` or `drafts/` that has no `created_at`,
  it writes the file's creation time on that device, in UTC.
  After that the value in the note stands, on every device.
  Anywhere else, `created_at` is typed when a note is started.

## Consequences

- The built site is unchanged.
- A promoted note or an old draft that still has `date` fails the build
  with a message naming the new field.
- Every note made in Obsidian is a post, dated when its file was made,
  and goes to the feeds once it isn't a draft.
  All 67 notes are posts already.
  An evergreen note, with `updated_at` and no `created_at`,
  can only be made outside Obsidian,
  since Linter adds `created_at` the first time it stamps one.
- A note's date is when its file was made, not when it was published:
  a draft started on the 1st and published on the 20th is dated the 1st.
  Setting `created_at` later by hand moves it, or schedules it if in the future.
- Git keeps no file times, so a file that reached a device by a pull or clone
  has the time it arrived.
  Linter only uses that for a file with no `created_at`,
  which today means the pages: `now.md` was first stamped Nov 10, the clone, not Nov 1.
  The build ignores `created_at` on a page.
- Linter rewrites a `created_at` it didn't write in its own form
  when it next stamps the note,
  so a bare day `2026-01-05` becomes `2026-01-05T00:00:00Z`.
  The day shown and the feeds' time are unchanged.
- A post with a time shows only its UTC day,
  so one written on a Pacific evening shows the next day,
  as `updated_at` already does.
- `tests/harness/created_at.rs` checks timed posts:
  the page, both feeds, the sort, the 24 hours and going live at the moment.
  `tests/harness/evergreen.rs` and `tests/harness/frontmatter_schema.rs` check
  the old name, the forms and that the schema and build agree.

## Alternatives considered

- **Keep `date`.**
  The convention Jekyll, Hugo and Obsidian share, and no rename,
  but it doesn't pair with `updated_at`.
- **`published_at`.**
  Accurate for the feeds, and what Ghost uses to schedule,
  but JC preferred a name that reads true for drafts and future posts.
- **Both `created_at` and `published_at`.**
  Two dates to keep for every post, where nothing here tells them apart.
- **Keep `created_at` a day.**
  Simpler, and with daily rebuilds a time only moves a scheduled post
  to the first build after it.
  JC wanted a datetime, which also orders posts within a day
  and gives the feeds a real time.
- **Require a datetime.**
  Every old note would need a time nobody recorded.
- **Type `created_at` by hand, and have Obsidian stamp only `updated_at`.**
  Keeps evergreen notes possible from Obsidian
  and never dates a note by a file time,
  but a new note has no date until one is typed.
  JC wanted both stamped on every note.
- **Stamp it, and mark evergreen notes some other way**, such as a folder.
  More to build, for a kind of note there are none of today.
