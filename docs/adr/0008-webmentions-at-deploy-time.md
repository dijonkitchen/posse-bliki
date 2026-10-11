# 0008 — Webmentions at deploy time

- Date: 2026-10-11
- Status: Accepted
- Decided by: JC, asking for the config, a send hook and a deploy-time fetch.

## Context

[ADR 0004](0004-indieweb-standards-supported.md) puts Webmention (via
webmention.io) and ActivityPub (via Bridgy Fed) in scope as external
services, with only `<link>` tags in our HTML. That receives mentions but
shows none, and nothing tells Bridgy when a post is published, so nothing
reaches Mastodon, Bluesky or the fediverse.

[ADR 0007](0007-rust-build-zero-dependencies.md) keeps the build std-only
Rust, and std has no HTTPS client. The spec also forbids `<script>`, so
mentions can't be loaded in the browser.

## Decision

Keep all network I/O in the deploy workflow; the build stays offline and
deterministic over its inputs.

- **Receive and show.** Before building, the workflow fetches webmention.io's
  jf2 export (secret `WEBMENTION_IO_TOKEN`). `bliki --webmentions FILE`
  renders each post's likes, reposts, bookmarks, replies and mentions as
  static `h-cite` responses inside its `h-entry`. Text only: mention content
  is escaped, never emitted as HTML, and only http(s) URLs become links. A
  daily scheduled deploy picks up new mentions.
- **Send.** Every post links (hidden, no text) to each target in
  `Config::syndicate_to`, as Bridgy requires. Before building, the workflow
  saves the live `feed.json`; after building, `bliki --pending-webmentions
  FEED` lists the posts that are new. After the deploy, a `syndicate` job
  posts one webmention per new post and target with `curl`. It runs only
  when the repository variable `SEND_WEBMENTIONS` is `true`.
- **No floods.** More than 5 new posts in one deploy, or a previous feed
  that is missing or isn't JSON, sends nothing. That covers the domain
  cutover, when every URL changes at once.

## Consequences

- Output now depends on one more input file, but the same file gives the
  same bytes, so build-invariants still hold.
- Mentions appear on the next deploy, at most a day late. Nothing here
  writes the Bridgy copies back into `syndication:` front matter; that
  stays manual.
- Mentions only match posts on the configured domain, so the github.io
  preview shows none.
- The default targets publish to JC's own Mastodon and Bluesky accounts
  (Bridgy Publish) and to the site's own fediverse account (Bridgy Fed).
  Bridgy Fed can also bridge that account to Bluesky, which would make two
  Bluesky copies; drop a target from `syndicate_to` to stop one.

## Alternatives considered

- **HTTPS in the build.** Needs a TLS crate, which ADR 0007 rules out.
- **Client-side widget** (webmention.js). Needs JS, which the spec forbids.
- **Bridgy Fed feed polling only.** Covers the fediverse but not Bridgy
  Publish, and gives no control over which posts go out.
