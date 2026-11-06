# 2026-11-06 — A standing invitation to email

- Date: 2026-11-06
- Status: Accepted
- Raised by: JC, asking to note, the way patio11 and others do,
  why readers should email.

## Context

About's Feedback section and the Bio's contact line say "email me"
(ADR 2026-11-06-coded-email), but not why a stranger should,
and many readers hold back for fear of being a bother.

Two writers this site already follows answer that on their own sites:

- Patrick McKenzie (patio11),
  [Standing Invitation](https://www.kalzumeus.com/standing-invitation/):
  "I was once a shy engineer with a pathological fear of annoying people
  by contacting them", "I like getting email",
  and "The worst thing that has ever happened from someone sending me an
  email is me being a bit busy that day and not replying."
  His [About page](https://www.kalzumeus.com/about/) points there.
- Derek Sivers, [contact](https://sive.rs/contact):
  "I reply to every email, and I enjoy it."
  The now page already credits him (ADR 2026-11-01-now-page).

## Decision

- About's Feedback section follows "Email me" with a short
  standing invitation, crediting both pages by link:
  you won't bother me; keep it short, the worst case is a slow reply;
  email beats DMs because it's on the site's own domain.
- It makes no promise about replies.
  patio11 has since stepped back from his implicit one,
  and Sivers's "every email" is his to keep, not this site's.
- The Bio's contact line adds "You won't bother me" and points to About
  for the invitation, so the reasons live in one place.
- A harness test checks About links both sources and Bio links About.

## Consequences

- Readers get a reason to write and a sense of how little it takes.
- The wording is JC's to tune; the test only checks the credits and the
  link, not the text.

## Alternatives considered

- **The full note on both pages.** Repeats itself;
  Bio is for work and stays short.
- **Promise a reply, like Sivers.** A promise JC would have to keep for
  every email; easy to add later if JC wants it.
