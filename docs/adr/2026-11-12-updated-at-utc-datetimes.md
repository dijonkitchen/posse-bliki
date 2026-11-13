# 2026-11-12 — `updated_at`, a UTC datetime

- Date: 2026-11-12
- Status: Accepted
- Decided by: JC, who asked "can it be `updated_at` and use UTC datetimes?",
  after asking whether notes should carry datetimes and update on their own,
  and Claude answering that an Obsidian plugin can stamp the field on save.

## Context

ADR 2026-11-07-evergreen-notes gave notes an `updated` field:
a day, set by hand, that moves a note to the top of Home and Notes.
A day set by hand is easy to forget,
and JC wanted the field to keep itself current,
with the time as well as the day,
under the name such a field usually has.

What shaped the answer:

- A time only says when if it says where.
  [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339#section-5.6)
  requires the offset from UTC on every date-time,
  and calls a local time without one
  "unacceptable for the Internet"
  ([section 4.4](https://www.rfc-editor.org/rfc/rfc3339#section-4.4)).
- Obsidian's own Date & time property writes `2020-08-21T10:30:00`,
  with no offset
  ([Obsidian Help: Properties](https://obsidian.md/help/properties)).
- The build is std-only Rust (ADR 2026-10-06-rust-build-zero-dependencies),
  which has no time zone rules,
  so it can't turn a local time into UTC.
  It can compare two UTC times with arithmetic alone.
- The build already takes "today" from the UTC clock
  (ADR 2026-10-20-scheduled-posts).
- Git's last-commit dates were turned down in ADR 2026-11-07-evergreen-notes.
  Run against the history on 2026-11-10 they bore that out:
  all 67 notes would have shown an update from 2026,
  none because a note was reworked.
  38 came from the two imports,
  28 from changes made across many notes at once (redirects, the link format and removing tags among them),
  and 1 from a one-line fix to a broken video link.

## Decision

- **The field is `updated_at`.**
  `updated` is no longer a field,
  and the build stops on it with "`updated` is now `updated_at`".
- **Its value is a UTC datetime**, `2026-11-12T04:27:28Z`, or a bare day, `2026-11-12`.
  The seconds are optional, and `+00:00` may stand in for `Z`.
  A time in another zone, or in none, fails the build.
- **Readers see the UTC day**, in the Holocene calendar (ADR 2026-10-11-holocene-years-on-the-page).
  Machines get the value as written in `<time datetime>`,
  which HTML allows as a
  [valid global date and time string](https://html.spec.whatwg.org/multipage/text-level-semantics.html#the-time-element).
  A bare day stays a bare day, so the page never claims a time nobody wrote.
  The sitemap's `<lastmod>` keeps to the day,
  which [its format](https://www.sitemaps.org/protocol.html#lastmoddef) allows.
- **A day with no time stands for noon UTC** when two notes are compared.
  Noon UTC falls on that date in nearly every time zone.
- **A post counts as updated from noon UTC on the day after its `date`.**
  `date` is a day in no time zone.
  It isn't over everywhere until the last zone, UTC−12, reaches midnight,
  which is noon UTC the next day:
  the "Anywhere on Earth" rule that
  [IEEE 802.16 uses for deadlines](https://www.ieee802.org/16/aoe.html).
  A save before that is part of publishing the post,
  whether it was made while drafting
  or on the evening the post went out in the Americas, already the next day in UTC.
  It shows no "updated" and moves nothing.
  For a bare day the rule comes to any day after `date`.
- **Lists sort by that update, or else by `date`.**
  The sitemap dates a page by the later of `date` and the UTC day of `updated_at`.
- **`date` stays a day**, and the feeds stay as they are.

## Consequences

- An edit made on a Pacific evening shows the next day's date,
  the way a scheduled post already goes live on the UTC day.
- A rework in the hours after the publish day,
  until 4 or 5 AM Pacific the next morning, isn't shown as an update.
  The next save after that is.
- Only `content/now.md` had `updated`, so the rename changed one line of content.
- Once something stamps `updated_at` on every note,
  a new post that is missing its `date` builds as an evergreen note,
  outside the feeds, instead of failing the build.
  Keeping `draft: true` until a post is ready is the guard.
- Every stamp counts, so saving a typo fix moves a note to the top.
  ADR 2026-11-12-obsidian-stamps-updated-at covers what does the stamping.
- `tests/harness/evergreen.rs` and `tests/harness/frontmatter_schema.rs` hold the rules,
  and `build/date.rs` the parsing.

## Alternatives considered

- **Git's last-commit date.**
  Hands-off, but it dates a note by its last mechanical change, as the history showed,
  and the deploy would need the whole history where it now fetches one commit.
- **Local time with its offset**, such as `2026-11-11T20:27:28-08:00`.
  It says when just as well,
  but one form is simpler to check and to sort by eye,
  and it is the form the plugin writes with one switch.
- **Compare UTC days only**, with no noon rule.
  Simpler, but every post published on a US evening would say "updated" with tomorrow's date
  and sit above the next day's post.
- **Keep the name `updated`.**
  Nothing technical in it.
  `updated_at` is the name JC asked for,
  and it reads as a moment, which the value now is.
- **A datetime on `date` too, or renaming it `published_at`.**
  Not asked for.
  Scheduling is to the day, and the page shows days.
- **Show the time of day on the page.**
  It is there for machines.
  For readers of a bliki the day is enough.
