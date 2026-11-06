# 2026-11-06 — A photo in the home h-card

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, who sent the photo to use
  after indiewebify.me's h-card check said "Add a photo!",
  and asked for it on the bio page too.
  Claude had recommended the spork icon; JC chose a photo.

## Context

The home page's representative h-card
(ADR 2026-10-16-home-shows-recent-posts)
has a name, nickname, URL and bio (ADR 2026-11-06-h-card-bio-note),
but no photo.
IndieWeb tools use the h-card's `u-photo` as the author's avatar
([h-card](https://microformats.org/wiki/h-card)):
social readers, the webmention displays on other people's sites,
and [Bridgy Fed](https://fed.brid.gy/docs), which uses it as the
fediverse profile picture.
[indiewebify.me](https://indiewebify.me/validate-h-card/) flags it when
it's missing.

JC's photo is a 1024×1024 JPEG (338 KB).
Avatars are shown small and cropped square:
Mastodon stores them at 400×400.

## Decision

- The photo ships as `content/images/jc.jpg`, 400×400, JPEG quality 80,
  progressive, about 40 KB, with all metadata (EXIF, ICC) stripped,
  so no camera or location data goes public.
  It sits with the other images and their budget
  (spec/performance.md).
- The home h-card gets
  `<data class="u-photo" value="https://www.dijonkitchen.org/images/jc.jpg">`.
  Microformats parsers take a `<data>` element's `value`
  ([mf2 parsing](https://microformats.org/wiki/microformats2-parsing)),
  and an absolute URL needs no resolving.
- Like the full name and the bio, it's not shown on the page.
  A hidden `<data>` isn't an image, so browsers never download it:
  the home page's weight and the "By JC" footer stay the same in every
  style.
- The bio page (/bio/) opens with the same photo, under its heading,
  10rem (160px) wide so the text still starts on a phone's first screen,
  with alt text "JC".
  Content can't hold raw HTML (spec/content-schema.md),
  so the build adds it ahead of `content/bio.md`'s text,
  and it goes through the same image step as any post image:
  width and height for no layout shift, eager loading, and a build error
  if the file is missing.
- The path lives in the build config as `author_photo`,
  next to the name and bio, and the config's JSON carries it as
  `author.photo`.
- Harness tests check the h-card's `u-photo` is that URL,
  the file is in the built site and square,
  and the bio page opens with it, sized, with alt text.

## Consequences

- indiewebify.me's h-card check passes,
  and readers and Bridgy Fed show JC's face next to posts and replies.
  Bridgy Fed keeps its old picture until JC presses "Update profile"
  there.
- Changing the photo means replacing `content/images/jc.jpg`
  (square, under the image budget) and, for a new name, `author_photo`.
- /bio/ loads the 40 KB photo; no other page does.
  To drop it from /bio/, remove the `BIO` step in `build/site.rs`.

## Alternatives considered

- **The spork icon.** Already the site's icon and JC's old Substack
  avatar, so nothing new to host; JC preferred a photo.
- **A visible `<img class="u-photo">` in the footer.** The microformats
  ideal is visible data, but it would add ~40 KB to the home page and
  change the footer in every style, where JC asked for a bare
  "By JC" (ADR 2026-10-16-site-title-heads-home).
- **Keep the original 1024px file.** Sharper on large screens nobody
  shows avatars on, at eight times the size.
