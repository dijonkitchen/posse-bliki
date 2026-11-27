# 2026-11-27 — The feeds match Home: newest update first

- Date: 2026-11-27
- Status: Accepted; replaces the "Feeds keep publish order" part of [2026-11-07-evergreen-notes](2026-11-07-evergreen-notes.md)
- Decided by: JC, who asked "But feed should sort by updated at right?",
  then "Both. Updates should be few",
  and "Feed should match home right?",
  and said "Yes" to Claude's recommendation below.

## Context

Home and `/notes/` list notes most recently tended first:
by a note's update, or else its `created_at`
(ADR 2026-11-07-evergreen-notes, `spec/content-schema.md`).
The feeds kept publish order, newest `created_at` first,
on the reasoning that "feed readers track items by their ID, not their place in the file".

That reasoning holds, and it cuts both ways.
A reader decides whether an item is new by its ID:
"The guid enables an aggregator to detect when an item has been received previously",
so it "does not need to be presented to a user again"
([RSS Best Practices Profile](https://www.rssboard.org/rss-profile)).
Where a reader then puts the item is up to the date the item carries,
not its place in the file.

Linter stamps a note's `created_at` when the file is made (ADR 2026-11-13-created-at).
So a draft started on Monday and published on Friday
was a new item in every reader, but dated Monday,
and landed below anything posted since.
Home already put it first,
because ticking `published` is an edit, and Linter stamps `updated_at` for it.

## Decision

- **The feeds carry Home's posts in Home's order,** less the evergreen notes:
  by a post's update, or else its `created_at`, then by slug.
  The RSS feed's 50 posts are the 50 most recently tended.
- **An RSS item is dated by when its post was last tended.**
  RSS gives an item one date,
  which "indicates the publication date and time of the item"
  ([RSS Best Practices Profile](https://www.rssboard.org/rss-profile)),
  so it is the date Home goes by:
  the update if there is one, or else `created_at`.
  `lastBuildDate` follows, from the first item.
- **A JSON Feed item keeps `date_published` from `created_at`, and gains `date_modified`** from its update,
  since JSON Feed has a field that "specifies the modification date in RFC 3339 format"
  ([JSON Feed 1.1](https://www.jsonfeed.org/version/1.1/)).
  Its items are in the RSS feed's order.
- **What counts as an update doesn't change.**
  It is `updated_at`, from 24 hours after `created_at` on
  (ADR 2026-11-13-created-at).
- **Syndication doesn't change.**
  The deploy syndicates the posts whose URLs are new to `feed.json`
  (`pending` in `build/webmention.rs`),
  and `feed.json` carries every post, so moving one up sends nothing.

## Consequences

- The feeds are the same today: none of the 67 posts has an update yet.
- A draft ticked more than a day after it was started
  goes to the top of the feeds, dated when it was ticked, as on Home.
  One ticked within a day keeps its `created_at`, which is less than a day old.
- Every edit Linter stamps after the first 24 hours is an update, a typo fix included.
  It moves the post to the top of Home and of the feeds,
  and changes its RSS date.
  A reader that has seen the post mostly won't show it again, since its ID is the same;
  some move it up.
  An old post outside the RSS feed's 50 comes back into it when it is updated.
- A JSON Feed reader that sorts by `date_published` still places a ticked draft by `created_at`.
- `tests/harness/evergreen.rs` checks the order against `/notes/`,
  RSS dating an updated post by its update,
  and `date_modified` appearing only on a real update.

## Alternatives considered

- **Keep the feeds in publish order** (ADR 2026-11-07-evergreen-notes).
  Moving an item in the file shows it to nobody again, as that ADR said,
  but a draft published late is placed by when it was started,
  and the feeds disagree with Home.
- **Reorder the feeds but keep `created_at` as the RSS date.**
  Readers go by the date, not the file's order, so it would change nothing they show.
- **Bump a post only on a deliberate change,** with a field set by hand.
  It would keep typo fixes out of the feeds,
  but it's one more field to remember, and JC expects updates to be few.
- **Date a post when it is ticked, with a "Publish now" command.**
  JC chose to set `created_at` by hand when that matters.
