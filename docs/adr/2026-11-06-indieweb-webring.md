# 2026-11-06 — Join the IndieWeb Webring, linked from every footer

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, signing up to the IndieWeb Webring (xn--sr8hvo.ws, 🕸💍)
  and sending the links from its dashboard.

## Context

The [IndieWeb Webring](https://xn--sr8hvo.ws) links personal sites in a
ring: each member shows "previous" and "next" links that send a reader
to a neighbouring member. Members sign in with their domain (IndieAuth
or RelMeAuth). The ring rescans each member's home page and marks the
site inactive if the links are missing; inactive sites are skipped.

A webring is not a blogroll. The [blogroll](2026-10-25-blogroll.md) is
JC's own list of sites JC reads; the webring is a ring JC has joined,
whose members JC doesn't choose. They stay separate: the blogroll page
keeps its name and list, and the webring is a line in the footer.

## Decision

- Every page's footer gets a second line with the ring's links, as the
  dashboard gave them: "← An IndieWeb Webring 🕸💍 →".
- The arrows get `aria-label`s ("Previous/Next site in the IndieWeb
  Webring") so screen readers announce more than an arrow, and the
  emoji are `aria-hidden`.
- Every footer, not only the home page: the ring needs the home page,
  but a reader finishing any note can hop on. The ring works out which
  site the reader came from by domain, so any page works.
- The harness checks every footer, including the home page's, has the
  three links, so a template change can't silently drop the site from
  the ring.

## Consequences

- The site is active in the ring once the deploy is live and the ring
  next scans the home page (or JC presses re-scan in the dashboard).
- Three more external links per page, no images, no script, no
  third-party requests until a reader clicks.
- Leaving the ring means deleting the constant and this test.

## Alternatives considered

- **Home page only.** Meets the ring's rule, but a reader on a note has
  to go home first to find the ring.
- **On the blogroll page, or renaming it "Webring".** The two are
  different things: one is curated, one is membership.
- **The ring's image badge.** It would be a third-party or extra image
  request on every page, against the performance budget.
