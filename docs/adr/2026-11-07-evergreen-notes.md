# 2026-11-07 — Evergreen notes, and lists ordered by when a note was last tended

- Date: 2026-11-07
- Status: Accepted; the `updated` field is now `updated_at`, a UTC datetime, by [2026-11-12-updated-at-utc-datetimes](2026-11-12-updated-at-utc-datetimes.md), and Obsidian stamps it, by [2026-11-12-obsidian-stamps-updated-at](2026-11-12-obsidian-stamps-updated-at.md); the `date` field is now `created_at`, by [2026-11-13-created-at](2026-11-13-created-at.md)
- Decided by: JC ("Go"),
  after asking "wouldn't it be good to have updated notes automatically be on top?"
  and Claude recommending the `updated` field over git dates.

## Context

The first scaffold
([PR #1](https://github.com/dijonkitchen/posse-bliki/pull/1))
made every note in `content/notes/` a dated post.
A note without `date` failed the build,
and every note went into the RSS and JSON feeds,
which the deploy uses to decide what to syndicate
(ADR 2026-10-11-webmentions-at-deploy-time).
No decision chose that.

It fits a blog, where posts come out once and are rarely revisited.
It fits a garden less well, and this site is a bliki.
Maggie Appleton describes a digital garden as
"a collection of evolving ideas that aren't strictly organised by their publication date"
([A Brief History & Ethos of the Digital Garden](https://maggieappleton.com/garden-history)),
and shows on each note when she "planted and last tended" it.
Mike Caulfield's
[The Garden and the Stream](https://hapgood.us/2015/10/17/the-garden-and-the-stream-a-technopastoral/)
sets the time-ordered stream against a garden,
where "to ask what happened first is trivial at best".

Two things followed from the dated-post model:

- A note promoted from the private vault had to take a `date`,
  so it showed up in the feeds as if it were new,
  and a batch of them would flood subscribers.
  The deploy refuses to syndicate more than five new posts at once
  (`MAX_NEW` in `build/webmention.rs`), so a batch would also stop syndication for that deploy.
- Updating a note did nothing visible beyond the "updated" line on its own page.
  Home and Notes were sorted by `date` only,
  so a note that had just been reworked stayed where it was first published.

## Decision

- **Evergreen notes.**
  A note in `content/notes/` with `updated` but no `date` is an evergreen note.
  It builds like any other note
  and is listed on the home page and on `/notes/`,
  but stays out of the RSS and JSON feeds,
  so it is never syndicated.
  Its page marks when it was updated with `dt-updated`
  and has no `dt-published`, since it was never posted.
  It waits for its `updated` date the way a scheduled post waits for its `date`
  (ADR 2026-10-20-scheduled-posts).
  A note with neither field still fails the build,
  so a forgotten date is still caught.
- **Lists ordered by last tended.**
  Home and `/notes/` put the note with the latest of `date` and `updated` first.
  A listed note shows its publish date,
  plus "updated <date>" when the update is later,
  or only "Updated <date>" for an evergreen note,
  so a reader can see why a note moved up.
  Home's heading is now "Recent notes".
- **Feeds keep publish order.**
  The RSS and JSON feeds still list dated posts by `date`, newest first.
  Feed readers track items by their ID, not their place in the file,
  so moving an item up would not show it to anyone again.
- **The front-matter field, not git.**
  "Updated" means the `updated` field the author sets.
  Git's last-commit date would need the full history in CI,
  where the checkout fetches one commit,
  and every typo fix or front-matter tweak would move a note to the top.

## Consequences

- Promoting notes from the vault no longer means posting them.
  A note gets `date` when it is worth announcing,
  and `updated` when it is worth tending in public.
- Editing a note and bumping `updated` puts it at the top of Home and Notes,
  and nothing is sent to subscribers.
- No note has `updated` today, so the live order doesn't change until one does.
- An Obsidian plugin can set `updated` on save,
  but that would bring back the typo-fix problem, so it's left to the author.
- The harness checks evergreen notes and the order in `tests/harness/evergreen.rs`,
  and every post page still needs `dt-published` when it is in the feed.

## Alternatives considered

- **Sort by git's last-commit date.**
  Hands-off, but every small fix counts as tending,
  and CI would need the full history to know the dates.
- **Re-sort the feeds by `updated` too.**
  Readers don't re-show an item they have already seen,
  so it would only reorder the file.
  Sending updates out belongs to syndication, if ever, not to the order.
- **A separate "Garden" list for evergreen notes.**
  It splits notes into two places when one list in order of last tended
  already shows both.
- **Growth stages (seedling, budding, evergreen) as a field.**
  More to maintain for every note.
  Whether a note has a `date` already says whether it was posted.
