# 2026-11-07 — A random link in the webring footer

- Date: 2026-11-07
- Status: Proposed
- Raised by: JC, about the webring line in the footer:
  "Check if footer should include random".

## Context

Every footer carries the IndieWeb Webring's line as the ring's dashboard gives it,
"← An IndieWeb Webring 🕸💍 →"
([ADR 2026-11-06-indieweb-webring](2026-11-06-indieweb-webring.md);
the dashboard's snippet is in
[`views/dashboard.php`](https://git.schmarty.net/schmarty/gem-diamond/src/commit/5333d2bd3f61d744ccb1f0feb562d9203d4c4be6/views/dashboard.php#L46-L49)).

The arrows don't change from click to click.
The ring gives each active site a sorting number,
and "next" goes to the site with the next higher number, "previous" to the next lower
(Marty McGuire, [Ordering an IndieWeb Webring](https://martymcgui.re/2024/06/23/ordering-an-indieweb-webring/), 2024-06-23).
The ring's source works the number out from when each site joined, offset by the month,
so the order reshuffles once a month
([`Site::updateSorting`](https://git.schmarty.net/schmarty/gem-diamond/src/commit/5333d2bd3f61d744ccb1f0feb562d9203d4c4be6/app/Model/Site.php#L99-L113)).
It knows which member the reader came from by the `Referer` header, matched exactly
([`Controller::next`](https://git.schmarty.net/schmarty/gem-diamond/src/commit/5333d2bd3f61d744ccb1f0feb562d9203d4c4be6/app/Controller.php#L103-L118)).
This site sets no referrer policy,
and the browser default, `strict-origin-when-cross-origin`, sends another site only the origin
([Referrer Policy](https://w3c.github.io/webappsec-referrer-policy/#default-referrer-policy)),
so the arrows on every page lead to the same two neighbours for the month.
Checked on 2026-11-07:
with this site as the referrer, ← went to issledova.tel and → to blog.darylsun.page every time;
with an unknown referrer, each went to a different random member every time.

The ring also has a random link, `/random`, to any active member.
Its home page offers it:
"Feeling lucky? Visit a Random site on the webring!"
([xn--sr8hvo.ws](https://xn--sr8hvo.ws/)).
Its steward added the route back for readers who liked hopping at random
(Marty McGuire, [Feeling lucky?](https://martymcgui.re/2024/07/31/103341/), 2024-07-31).

The dashboard's snippet leaves it out.
The ring checks a member's home page for links to itself,
counts the site active when `/next` and `/previous` are both there,
and lists any other ring link except its home page as "Found unknown link"
([`LinksCheck`](https://git.schmarty.net/schmarty/gem-diamond/src/commit/5333d2bd3f61d744ccb1f0feb562d9203d4c4be6/app/Util/LinksCheck.php#L65-L103)).
A member who added `/random` reported that note in a comment on the steward's post,
and the source still lists it today.

Few members link it.
Of 40 sites from the ring's [directory](https://xn--sr8hvo.ws/directory) sampled on 2026-11-07,
22 had their ring links in the page's HTML, and 2 of those also linked `/random`.

## Decision

- The footer's webring line gets a fourth link, after →:
  a die, 🎲, to `https://xn--sr8hvo.ws/random`.
  The ring marks the feature with 🎲 in its own list of changes.
- It gets an `aria-label`, "Random site in the IndieWeb Webring", like the arrows,
  so a screen reader announces more than "game die".
- Every footer, the same as the arrows,
  and the harness checks every footer has it.
- The blogroll's line about the ring mentions the die.

## Consequences

- A reader can reach a different member with each click,
  not only this month's two neighbours.
- The ring's dashboard will list
  "Found unknown link: https://xn--sr8hvo.ws/random"
  each time it checks the home page.
  That's a note, not a fault: the site stays active.
  If the ring ever counts unknown links against a site, drop the die.
- One more link per page, about 100 bytes, with no image, script or third-party request until a reader clicks.

## Alternatives considered

- **Leave the line as the dashboard gives it.**
  No note on the dashboard,
  but a reader who comes back only ever reaches the same two members in a month.
- **The word "random" instead of a die.**
  Plainer to a first-time reader,
  but longer on a phone's footer line, which is arrows and emoji already.
- **Only on the blogroll page**, beside the ring's entry.
  The ring checks only the home page, so the dashboard wouldn't note it,
  but a reader finishing a note wouldn't see it.
- **Hiding it from the checker**, with a protocol-relative or Unicode-host URL
  its prefix match skips.
  Works today, but it games the ring's own check.
