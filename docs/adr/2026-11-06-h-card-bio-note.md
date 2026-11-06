# 2026-11-06 — A one-line bio in the home h-card

- Date: 2026-11-06
- Status: Accepted
- Raised by: JC, passing on indiewebify.me's suggestion:
  "Got a brief bio like a Twitter/Instagram bio?
  Add it to your own h-card as a note property!"

## Context

The home page's representative h-card
(ADR 2026-10-16-home-shows-recent-posts)
has a name, a nickname and a URL, but no bio.
IndieWeb tools that show a profile,
such as social readers and the people who receive a webmention,
read the h-card's `p-note` for a short description of the person
([h-card](https://microformats.org/wiki/h-card)).
[indiewebify.me](https://indiewebify.me/) flags it when it's missing.

The bio page (ADR 2026-10-16-bio-page) already opens with a description
of JC in JC's own words:
"a full-stack, entrepreneurial engineer that has built businesses
from zero to one, scaled up new product lines to millions of dollars, …".

The home h-card shows only "By JC" in the footer,
at JC's request (ADR 2026-10-16-site-title-heads-home),
and keeps the full name in a `<data>` element so parsers see it
without it showing on the page.

## Decision

- The home h-card gets
  `<data class="p-note" value="Full-stack, entrepreneurial engineer that has built businesses from zero to one.">`,
  the first clause of the bio page's opening, unchanged.
  Microformats parsers take a `<data>` element's `value` as the property
  ([mf2 parsing](https://microformats.org/wiki/microformats2-parsing)).
- It's a `<data>` value like `p-name`, so the byline still reads "By JC"
  and no style's footer changes.
- The text lives in the build config as `author_note`,
  next to the name and nickname, and the config's JSON carries it as
  `author.note`.
- A harness test checks the home h-card's `p-note` equals the configured
  bio.

## Consequences

- indiewebify.me's h-card check finds a note,
  and profile cards built from the h-card show a one-line bio.
- Changing the bio means editing `author_note` in `build/site.rs`,
  not a content file, like the name and tagline.
  If the bio page's opening changes, this line doesn't follow it.
- The note is invisible on the page, as the full name already is.
  Readers see the same text at the top of /bio/.

## Alternatives considered

- **Show the bio in the footer.** Visible data is the microformats ideal,
  but JC asked for a bare "By JC" byline,
  and a sentence there would change the footer in every style.
- **JC's GitHub bio, "Mini-gamifying utopia".** Shorter, and JC's own,
  but it doesn't say what JC does to someone meeting them through a
  webmention. It's a one-line swap if JC prefers it.
- **Read it from the bio page's first paragraph.** Keeps one source,
  but the paragraph is several sentences of work history,
  too long for a profile line, and cutting it automatically could end
  mid-thought.
