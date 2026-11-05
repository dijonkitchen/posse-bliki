# 2026-11-05 — Tests of optional pages build a small vault

- Date: 2026-11-05
- Status: Proposed
- Decided by: JC, asking why the no-now-page test had to rewrite links
  to the now page.

## Context

The now, follow and blogroll pages are optional: the build links them from
the header or footer only when the vault has them
([now in the header](2026-11-05-now-in-the-header.md),
[follow with a feed reader](2026-11-05-follow-with-a-feed-reader.md)).
The harness checks that by building a vault without the page.

Those tests copied the whole of `content/`, deleted the page, then
rewrote every link to it, because a link to a missing page fails the
build ([relative Markdown links](2026-10-16-relative-markdown-links.md)).
Each new link to an optional page meant editing a test, as the bio's link
to the now page did.

## Decision

- A test about a page the vault may leave out builds `small_vault`: a
  home page plus only the pages that test needs. Nothing in it links to
  the missing page, so there is nothing to rewrite.
- Tests about the real content keep building `content/` or a copy of it.

## Consequences

- Linking an optional page from real content never needs a test edit.
- These tests no longer show the real vault builds without the page, but
  that isn't a case the site needs: its pages link the optional ones on
  purpose.

## Alternatives considered

- **Keep rewriting links in a copy of the vault.** Works, but grows with
  every cross-link.
- **Make the now page required and always linked.** Removes one test but
  not the follow and blogroll ones, and stops a fork from leaving it out.
