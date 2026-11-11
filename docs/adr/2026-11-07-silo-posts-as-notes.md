# 2026-11-07 — Bring the best silo posts over as notes

- Date: 2026-11-07
- Status: Accepted
- Decided by: JC, who asked whether 𝕏 content is worth bringing over,
  chose "Some: threads and best posts"
  over all of it in a separate archive or none,
  then asked to consider Threads and Bluesky too.

## Context

The About page already names the aim:
"[Slowly] wrangling legacy social media (Twitter, …)
into a controlled, centralized network of thoughts".
It also says the bliki is for "random musings longer than 140 characters".

What each site holds, as of 2026-11-07:

- **𝕏.**
  Unknown until JC downloads the archive
  (Settings, Your account, Download an archive of your data).
  Its `data/tweets.js` holds every post as JSON,
  including retweets and replies,
  and is what archive tools read
  ([tweetback](https://github.com/tweetback/tweetback),
  [twitter-archive-parser](https://github.com/timhutton/twitter-archive-parser)).
  Links in posts are `t.co` short links,
  and the archive has the full address for only some of them
  ([inkdroid](https://inkdroid.org/2022/11/20/t-dot-co/)).
  The archive also holds direct messages.
- **Bluesky** (`@dijonkitchen.org`).
  12 posts, 2026-02-18 to 2026-05-07,
  read from the public `app.bsky.feed.getAuthorFeed` API.
  Each one is only a link to a Substack post
  that is already a note here.
- **Threads** (`@dijonkitchen`).
  2 posts, by the public profile.
  Threads exports from its phone app
  ([Review and export your Threads information](https://help.instagram.com/259803026523198/)).

How the site would take them:

- Every post is a dated note.
  The home page lists the 10 newest
  (ADR 2026-10-16-home-shows-recent-posts),
  and 2026 alone has 17,
  so a note dated in an earlier year never reaches it.
- The RSS feed carries the 50 newest posts (spec/feeds.md),
  which reach back to about 2017,
  so a note from after then joins the feed
  and readers see it as a new item with its old date.
  The JSON Feed carries every post.
- The deploy asks Bridgy to post every note that is new to the feed,
  whatever its date,
  at most 5 per deploy
  (ADR 2026-10-11-webmentions-at-deploy-time).
  Bridgy Fed doesn't bridge posts more than two weeks old
  ([Bridgy Fed docs](https://fed.brid.gy/docs)),
  but nothing stopped Bridgy Publish
  from posting a 2019 thread to Mastodon and Bluesky as if it were new.
  Sending is off for now:
  the `SEND_WEBMENTIONS` variable isn't set,
  and the `syndicate` job was skipped on the last deploy.
- Bridgy can't post to 𝕏 or read from it:
  its access was suspended on 2023-04-04
  ([Bridgy about](https://brid.gy/about)).

The IndieWeb name for copying silo posts to your own site is
[PESOS](https://indieweb.org/PESOS).
That page says to give the copy a `syndication` link to the original
([u-syndication](https://indieweb.org/u-syndication)).

## Decision

- **Some, not all.**
  JC's own threads and the posts that still hold up
  come over as ordinary notes, picked by hand.
  Reposts and replies to other people stay where they are.
  spec/content-schema.md, "Posts first published elsewhere",
  says how a note is made:
  a title, the original's date,
  the original's URL in `syndication`,
  short links replaced by their full address,
  and no more than 5 in one deploy.
- **𝕏.**
  JC downloads the archive and keeps it private,
  never in this repo, since it holds direct messages.
  From `data/tweets.js`,
  Claude leaves out retweets and replies to other people,
  joins self-reply threads,
  and ranks the rest by likes and reposts
  so JC picks from a shortlist.
- **Bluesky.**
  Nothing to bring over.
  Each of the 12 posts becomes a `syndication` link
  on the note it links to.
- **Threads.**
  JC reads the 2 posts;
  any that holds up comes over like an 𝕏 post.
- **No webmentions for back-dated notes.**
  `--pending-webmentions` leaves out a new note
  dated more than 14 days before the build's today,
  the same two weeks Bridgy Fed uses (spec/indieweb.md).

## Consequences

- No new kind of post, field or page:
  a brought-over note is a note.
- The home page stays about new writing.
  Feed readers may see a few old-dated items at a time.
- Turning on `SEND_WEBMENTIONS` later won't re-post old threads.
  It also won't send a note JC writes now
  but dates more than two weeks back;
  that one needs a webmention sent by hand.
- The archive and its direct messages never become public.
- Each note's original stays on 𝕏 for as long as 𝕏 keeps it;
  the note no longer depends on it.

## Alternatives considered

- **All of it, in a separate `/x/` archive,**
  as Zach Leatherman does with tweetback
  ([zachleat.com/twitter](https://www.zachleat.com/twitter/)).
  Needs a new kind of untitled post kept out of the home page and feeds.
  Replies carry other people's words without their side of the conversation,
  and 𝕏 exports don't mark Circle posts as private,
  so a full import can publish what was meant for a few
  ([indieweb.org/Twitter](https://indieweb.org/Twitter)).
- **None, a private backup only.**
  Cheapest,
  but leaves the threads worth keeping on a site JC doesn't control,
  against the About page's own aim.
- **Skip notes that already list a `syndication` link,**
  instead of skipping by date.
  A new note can list a copy made by hand before it's published,
  and should still go to Bridgy.
  The date says "published elsewhere first" more directly.
