# 0013 — www as the canonical host

- Date: 2026-10-14
- Status: Accepted
- Decided by: JC, choosing www over the bare domain "for flexibility in future".

## Context

The site moved from Substack to GitHub Pages on 2026-10-11. DNS is at Gandi;
email (Fastmail MX, SPF, DKIM) shares the domain and must not change.
The build declared `https://dijonkitchen.org` as canonical while Pages served
`www.dijonkitchen.org` and redirected the bare domain to it, so every
canonical link pointed at an address that redirects away. One of the two had
to change, and the canonical host is costly to change later: links,
webmentions and the IndieWeb identity (`rel=me`, webmention.io, Bridgy) all
accrue to it.

## Decision

`https://www.dijonkitchen.org` is canonical (`base_url`, `author_url`, and the
webmention.io endpoints in `default_config`). DNS:

- `www` is a CNAME to the host (`dijonkitchen.github.io.`).
- The bare domain has GitHub's four A and four AAAA records, and Pages
  redirects it to www.

## Consequences

- Moving hosts means changing the `www` CNAME. The bare domain needs either
  the new host's addresses or a redirect-only service that never changes
  (as it had under Substack: Gandi ALIAS to a redirector).
- Hosts that only hand out a hostname (Netlify, Vercel, Cloudflare, Substack)
  work without apex tricks, because a CNAME is not allowed at the apex
  alongside MX.
- webmention.io and Bridgy accounts are signed in as `www.dijonkitchen.org`.
- Mentions targeting the bare domain still match: targets are compared
  ignoring a leading `www.` (spec/indieweb.md).

## Alternatives considered

- **Bare domain.** Shorter, and what the build used first. Rejected: a host
  move touches two sets of records, and hostname-only hosts need an
  ALIAS/flattening feature at the DNS provider.
