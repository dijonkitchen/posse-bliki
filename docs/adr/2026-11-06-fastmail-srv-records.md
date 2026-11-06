# 2026-11-06 — Publish Fastmail's SRV records for app discovery

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, after comparing SRV records with Fastmail's Apple
  setup profiles and `.well-known` redirects.

## Context

Fastmail's manual DNS guide lists SRV records that let mail, calendar
and contacts apps find its servers from an address alone
(https://www.fastmail.help/hc/en-us/articles/360060591153-Manual-DNS-configuration).
The labels come from standards: RFC 6186 for IMAP, POP3 and submission
(https://www.rfc-editor.org/rfc/rfc6186#section-3), RFC 8314 for
`_submissions` (https://www.rfc-editor.org/rfc/rfc8314#section-7.3),
RFC 6764 for CalDAV and CardDAV
(https://www.rfc-editor.org/rfc/rfc6764#section-3), RFC 8620 for JMAP
(https://www.rfc-editor.org/rfc/rfc8620#section-2.2), and Microsoft's
Autodiscover for `_autodiscover`
(https://learn.microsoft.com/en-us/exchange/architecture/client-access/autodiscover).
Mail delivery doesn't need them; the Fastmail apps and Apple devices set
up with Fastmail's profile don't either. Third-party apps (Thunderbird,
DAVx5, some Outlook versions) use them, or ask for server names by hand.

## Decision

Publish all twelve SRV records Fastmail lists, at Porkbun, mirrored in
`docs/dns/dijonkitchen.org.zone`. Plain-text services get the target
`.`, which RFC 2782 defines as "decidedly not available"
(https://www.rfc-editor.org/rfc/rfc2782), so apps go straight to TLS.
Porkbun's form takes weight, port and target together in its Target
field, with priority separate
(https://kb.porkbun.com/article/109-how-to-create-an-srv-record).

## Consequences

- Adding the account to a new third-party app needs only the address and
  an app password.
- The records are public pointers to Fastmail's shared servers. They
  reveal nothing the MX records don't, and logins still need an app
  password, which Fastmail requires for IMAP, SMTP and DAV and lets you
  scope and revoke per device
  (https://www.fastmail.help/hc/en-us/articles/360058752854-App-passwords).
- If email moves off Fastmail, these records move or go with the MX
  records.

## Alternatives considered

- **Skip them.** Fine for Fastmail's apps and Apple's profile, but each
  third-party app needs servers typed in.
- **`.well-known/caldav` and `/carddav` redirects on the site.** RFC 6764
  says the server "MUST redirect HTTP requests for that resource"
  (https://www.rfc-editor.org/rfc/rfc6764#section-5), so clients expect
  a real HTTP redirect to Fastmail. GitHub Pages only serves files, and
  the bliki's aliases are meta-refresh pages that answer 200
  (spec/url-policy.md), which these clients don't follow.
