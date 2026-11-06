# 2026-11-06 — Publish Fastmail's SRV records for app discovery

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, after comparing SRV records with Fastmail's Apple
  setup profiles and `.well-known` redirects.

## Context

Fastmail's manual DNS guide lists SRV records that let mail, calendar
and contacts apps find its servers from an address alone
(https://www.fastmail.help/hc/en-us/articles/360060591153-Manual-DNS-configuration).
Mail delivery doesn't need them; the Fastmail apps and Apple devices set
up with Fastmail's profile don't either. Third-party apps (Thunderbird,
DAVx5, some Outlook versions) use them, or ask for server names by hand.

## Decision

Publish all twelve SRV records Fastmail lists, at Porkbun, mirrored in
`docs/dns/dijonkitchen.org.zone`. Plain-text services get the "not
offered" target (`0 0 0 .`) so apps go straight to TLS.

## Consequences

- Adding the account to a new third-party app needs only the address and
  an app password.
- The records are public pointers to Fastmail's shared servers. They
  reveal nothing the MX records don't, and logins still need an app
  password.
- If email moves off Fastmail, these records move or go with the MX
  records.

## Alternatives considered

- **Skip them.** Fine for Fastmail's apps and Apple's profile, but each
  third-party app needs servers typed in.
- **`.well-known/caldav` and `/carddav` redirects on the site.** CalDAV
  and CardDAV clients need a real HTTP 301 to Fastmail. GitHub Pages
  only serves files, and the bliki's aliases are meta-refresh pages
  that answer 200, which these clients don't follow.
