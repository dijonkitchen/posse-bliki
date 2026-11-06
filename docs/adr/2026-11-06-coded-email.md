# 2026-11-06 — Email in HTML character codes, as rel=me and contact links

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, who asked whether rel=me should be GitHub
  and whether it could be something neutral,
  then chose to add email next to GitHub ("ok, use html character code
  email") and to use it wherever the site gives contact details.

## Context

The home page's only `rel=me` was GitHub
(ADR 2026-10-16-site-title-heads-home).
[IndieLogin](https://indielogin.com/) signs people in to IndieWeb sites
through any `rel=me` provider it supports, email among them,
so an email `rel=me` lets JC log in without going through GitHub
([rel-me](https://microformats.org/wiki/rel-me)).

The About and Bio pages gave the address as a riddle,
"`${my-initials}` [at] MY_DOMAIN_NAME", to keep it from spammers,
at the cost of readers working it out and typing it.

Spencer Mortensen's honeypot
([Email address obfuscation: What works in 2026?](https://spencermortensen.com/articles/email-obfuscation/),
updated 2026-08-08) puts a different address behind each technique and
counts which spammers find which.
All 742 spammers harvested a plain address;
a `mailto:` link written in HTML character codes stopped 99% of the
spammers that harvest links, and plain text in character codes 95%.
Browsers and real HTML parsers decode character codes,
so readers and IndieLogin see a normal address.

The address is already the author email on this repo's commits,
so hiding it on the site lowers spam rather than keeping it secret.

## Decision

- The build config holds only the part before the `@`,
  `author_email_user: "jc"`; the domain is the site's, without `www.`.
  The full address isn't written out anywhere in the repo.
- The home page gets a second `rel=me`,
  `<link rel="me" href="&#109;&#97;&#105;&#108;…">`:
  `mailto:` and the address, every character as a numeric character
  reference. GitHub stays.
- In content, a link to `mailto:` with no address,
  `[email me](mailto:)`, becomes a link to the author in the same codes.
  Content stays Markdown with no raw HTML (spec/content-schema.md),
  and the address doesn't appear in it.
- The About and Bio pages swap the riddle for an "email me" link.
  The link text doesn't show the address, so it isn't in the page's text
  either.
- Harness tests check the home `rel=me` decodes to the address next to
  GitHub, About and Bio link to it in codes, and no built file
  (HTML, feeds, sitemap, OPML, text) contains the address in plain text.

## Consequences

- Readers click "email me" instead of decoding a riddle,
  and JC can sign in with IndieLogin by email.
- Harvesters that decode character codes still get the address.
  Fastmail's spam filter is the backstop,
  and moving to a separate alias is a one-line change to
  `author_email_user`.
- Readers who want the address itself see it on hover or by copying the
  link. Screen readers announce the link text, "email me".

## Alternatives considered

- **Keep GitHub as the only rel=me.** Nothing to hide, but GitHub stays
  the only way to sign in with the site.
- **Email only, dropping GitHub.** Neutral, but GitHub is also how
  people find JC's code, and it links back to the site.
- **Stronger hiding** (JavaScript or CSS tricks, 100% in the same
  honeypot). The site's only script is the style picker,
  and rel=me must be a plain `href` that IndieLogin can read.
- **Show the address as the link text, also coded.** More convenient,
  but the address would be in the visible text, where the 5% of
  plain-text harvesters that decode codes would find it.
