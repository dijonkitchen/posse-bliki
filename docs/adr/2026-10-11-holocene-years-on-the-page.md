# 2026-10-11 — Holocene years on the page

- Date: 2026-10-11
- Status: Accepted
- Decided by: JC, asking for "a 1 in front of year dates".

## Context

JC wants the site's dates to nod to the [Long Now Foundation](https://longnow.org),
which writes years with five digits (02026) to think in 10,000-year spans,
and to Kurzgesagt's Human Era calendar,
which counts from the start of the Holocene, roughly when human civilisation
began: 2026 CE is 12026 HE. Adding 10,000 to the year is all it takes.

Dates also feed machines: `dt-published` for microformats parsers and
webmention receivers, RSS `pubDate`, JSON Feed `date_published`, and the
sitemap's `lastmod`. Those formats require Gregorian ISO or RFC 822 years.

## Decision

- Every date a reader sees shows the Holocene year: `12026-10-11`. That is
  post dates, "updated" dates, list and home page dates, and received
  webmention dates. `Date::human()` in `build/date.rs` is the one place
  that formats them.
- Every machine-readable date stays Gregorian: `<time datetime="2026-10-11">`,
  feeds, the sitemap, and URLs. mf2 parsers read the `datetime` attribute,
  so h-entries are unchanged.
- Front matter keeps Gregorian `YYYY-MM-DD` (spec/content-schema.md), so
  Obsidian and every other tool still read it.

## Consequences

- A visitor may read 12026 as a typo; the colophon explains it.
- Years mentioned in prose are untouched. Only dates the build formats change.
- Switching back is one function.

## Alternatives considered

- **Long Now style, 02026.** A leading zero rather than a one; JC asked for
  the one, which also makes the year mean something (years since the Holocene).
- **Holocene years everywhere, feeds included.** Breaks RSS, JSON Feed,
  microformats and webmentions, which all expect real calendar dates.
