# 2026-10-16 — A résumé page replaces LinkedIn

- Date: 2026-10-16
- Status: Proposed
- Raised by: JC, who asked for a professional bio on the site as a
  LinkedIn and résumé replacement, so people can find them for work.

## Context

JC's work history lived in a private résumé and on LinkedIn. Following
POSSE, the canonical copy should live on JC's own site, where it outlasts
any one platform and links to the writing behind it.

## Decision

- A page at `/resume/` (`content/resume.md`, alias `/cv/`) holds a short
  summary, what JC is looking for, how to get in touch, experience, skills
  and education. The experience comes from JC's own résumé and the
  current role from the now page; anything not found in JC's own
  materials was asked of JC rather than invented (JC filled in dates,
  what work they want, and education on 2026-11-05).
- It is a plain Markdown page. The file is `resume.md`, not `work.md`,
  because the 2017 post `notes/work.md` already owns that stem and `/work/`.
- It is not in the header nav, which JC kept to three links so it doesn't
  wrap on phones (ADR 2026-11-05-now-in-the-header). Once it's published,
  the now page and the About page link to it.
- The visible name stays "JC", as on the rest of the site
  (ADR 2026-10-14-site-and-author-naming). Email is written out the same
  way as on About, not as a `mailto:` link, to keep scrapers off it.

## Consequences

- Recruiters and collaborators reach the work history from Now or About,
  and search engines index it on JC's domain.
- Updating the résumé is editing one Markdown file. LinkedIn can link here.

## Alternatives considered

- **Extend the About page.** About is about the bliki; mixing in a work
  history buries both.
- **A header nav link.** One tap from every page, but a fourth link wraps
  on small phones in several styles.
- **An h-resume microformat.** The draft h-resume vocabulary would need raw
  HTML or a custom template, and no common tool consumes it. The home
  page's h-card already identifies JC.
- **A PDF résumé.** Harder to read on a phone, not indexed as well, and
  drifts from the page.
