# 2026-10-16 — The site title heads the home page

- Date: 2026-10-16
- Status: Accepted
- Raised by: JC, who saw the site title rendered smaller than the author's
  name on the home page and asked whether it was right and accessible.
  Accepted when JC merges the PR that adds this ADR.

## Context

The home page's `<h1>` was the author h-card, "JC (Jonathan Chen)", at the
full h1 size. "Dijon Kitchen" sat above it as a small link in the header, so
the page read as a profile page that happened to be on a site.

- The home page is the site's front door. Its main heading should name the
  site, so the visual order and the heading outline (what screen-reader users
  jump through) both start with "Dijon Kitchen".
- The site title and tagline were inside `<nav>`, so screen readers announced
  them as part of the "Site" navigation landmark.
- The h-card showed its links as raw URLs, `https://www.dijonkitchen.org/` and
  `https://github.com/dijonkitchen`, which screen readers spell out in full.

An axe-core run (WCAG 2.1 A/AA plus best practice, light and dark, 375px and
1280px) found no violations before or after this change; the problems were
hierarchy and landmark structure, which axe doesn't judge.

## Decision

- On the home page only, the header's site title is the page's `<h1>`, at the
  design system's h1 size. Its link carries `aria-current="page"`. Every other
  page keeps the small title link and its own `<h1>`.
- The title and tagline move out of `<nav>` into the header. The nav holds
  only the page links.
- The author h-card becomes one short byline under the header: "By JC"
  (JC asked for just that, without the GitHub link). "JC" links to the home
  page as the h-card's `u-url u-uid` (a representative h-card, which Bridgy
  reads). The full name stays in the markup as
  `<data class="p-name" value="Jonathan Chen">`, so parsers and search still
  see it (ADR 2026-10-14-site-and-author-naming).
- Post bylines show "JC" the same way, also at JC's request. This replaces
  the visible "JC (Jonathan Chen)" from ADR 2026-10-14-site-and-author-naming;
  that ADR's markup (`p-nickname` "JC", `p-name` "Jonathan Chen") and feed
  author name stand.
- The GitHub profile moves to `<link rel="me">` in the home page's `<head>`:
  not shown, but still there for identity checks (IndieAuth, rel=me
  verification).

## Consequences

- Parsers read the same h-card: `p-name` "Jonathan Chen", `p-nickname` "JC",
  `u-url` and `u-uid` the site. The GitHub URL is no longer an h-card
  `u-url`; it stays a page-level `rel=me`.
- The harness's mf2 parser now reads `p-*`/`u-*` from a `<data>` element's
  `value`, as the microformats2 parsing spec says.
- `spec/output-contract.md` now says the home `<h1>` is the site title; the
  harness checks that and that the site brand stays outside `<nav>`.

## Alternatives considered

- **Keep the author as the `<h1>` and just make the header title bigger.**
  Two competing large names, and the outline would still start with the
  author.
- **A second "Dijon Kitchen" `<h1>` inside `<main>`.** Puts the title twice on
  one screen.
- **Hide the self-link (`u-url u-uid`) from view.** Hidden links confuse
  keyboard and screen-reader users; the author name is a natural link text.
