# 2026-10-28 — A security.txt that points at GitHub's private reporting

- Date: 2026-10-28
- Status: Accepted
- Decided by: JC, after a Cloudflare Radar URL scan of
  www.dijonkitchen.org, choosing a `security.txt` alongside the CSP meta
  tag (ADR 2026-10-28-csp-meta-tag).

## Context

RFC 9116 defines `/.well-known/security.txt`, the standard place a
security researcher looks to learn how to report a problem with a site.
It needs a `Contact` and an `Expires` (recommended less than a year
ahead, so a stale file reads as stale), and may name its `Canonical` URL
and `Preferred-Languages`. The site served a 404 there.

The site publishes no email address. This repository already has
GitHub's private vulnerability reporting turned on, which gives a
reporter a private thread with JC and needs no address on the page.

## Decision

- The build writes `/.well-known/security.txt` with:
  - `Contact:` this repository's private vulnerability reporting form,
    `https://github.com/dijonkitchen/posse-bliki/security/advisories/new`
  - `Expires:` 180 days after the build's today (`--today`, else the UTC
    date), at midnight UTC
  - `Preferred-Languages: en`
  - `Canonical:` the file's URL on `site.base_url`
- Deploying `public/` with `actions/upload-pages-artifact` keeps the
  dot-directory, so Pages serves the file.

## Consequences

- The daily 06:17 UTC rebuild (ADR 2026-10-20-scheduled-posts) rolls
  `Expires` forward, so the file never needs hand edits. If the repo goes
  quiet long enough for GitHub to pause scheduled workflows (60 days
  without activity), the file still has months left; any push renews it.
- The file changes daily, like anything else that depends on `--today`.
  Two builds on the same day are still byte-identical.
- If private vulnerability reporting is turned off in the repository's
  settings, the Contact link stops working. Switching to a `mailto:` is a
  one-line change to `SECURITY_CONTACT` in `build/site.rs`.
- The file isn't signed (OpenPGP is optional in RFC 9116); it's served
  over HTTPS from the canonical host, which is what readers can check.

## Alternatives

- **A `mailto:` contact.** It would put an address on the site for
  scrapers, and there's no `security@` alias yet.
- **A hand-written static file with a fixed `Expires`.** It would quietly
  expire within a year.
