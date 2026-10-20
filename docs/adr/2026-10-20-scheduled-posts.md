# 2026-10-20 — Future-dated posts go live on their date

- Date: 2026-10-20
- Status: Accepted (JC chose "Yes, as proposed", 2026-10-20)
- Raised by: JC, who asked how posts get scheduled and whether the site
  could publish only items dated in the past.

## Context

The build skipped only `draft: true` notes. A post with a future `date`
went live on the next push to main, showing a date that had not come yet,
and was syndicated to Bridgy that same day. To schedule a post you had to
keep it as a draft and remember to flip it on the day.

The deploy workflow already rebuilds every day at 06:17 UTC so new
webmentions show up, and syndication already sends only the posts that are
new against the live feed.

## Decision

- A note whose front-matter `date` is after today is left out of the build
  the same way a draft is: pages, lists, tags, feeds, sitemap, aliases and
  webmentions.
- "Today" is the UTC date from the clock, or `--today YYYY-MM-DD` (`make
  build TODAY=…`). The flag keeps the build a function of its inputs, so
  tests pin the date and a scheduled post can be previewed locally.
- The existing daily rebuild publishes the post on its date with no push,
  and the syndication step sends it to Bridgy because it is new in the feed.
- A published note that links to a scheduled one fails the build with the
  usual broken-link error, as a link to a draft does, so no live page ever
  points at a 404.

## Consequences

- To schedule a post, give it a future `date` and merge it. It appears at
  the 06:17 UTC rebuild on that date (late evening the day before in US
  time zones). Moving the cron changes that time.
- Dates have no time of day, so scheduling is to the day.
- If a scheduled run fails or GitHub skips it, the post goes live on the
  next successful build.
- A post that teases a scheduled one can't link to it until it is live;
  add the link in a later edit.

## Alternatives

- Keep using `draft: true` and flip it by hand: works, but needs a push on
  the day.
- Show future posts with a "scheduled" badge: publishes the text early.
- Read the clock with no override: simpler, but tests would depend on the
  day they run and there would be no way to preview.
