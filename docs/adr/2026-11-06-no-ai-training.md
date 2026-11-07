# 2026-11-06 — Ask AI training crawlers to stay out and keep search crawlers in

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC: "Don't let them train." JC also asked whether search
  crawling should stay. It stays here, on Claude's recommendation, for JC
  to confirm on the PR.

## Context

`robots.txt` came with the first scaffold (PR #1) as `User-agent: *` /
`Allow: /`, which lets every crawler in, AI training crawlers included.
Nobody chose that. When JC saw it among the scaffold's undecided choices,
JC said not to let AI crawlers train on the site, and asked whether to
keep search crawling or block every crawler.

Blocking every crawler would take the site out of search engines. People
would only find it by a link they already had, and the small-web
directories JC means to join (Kagi Small Web, nownownow.com) send readers
to sites that search can already reach. The site has no search of its
own (no JS, no server) to make up for that.

The large AI companies now split training from search, each with its
own robots.txt token, so the two can be told apart:

| Token | Operator | What its operator says it's for |
|---|---|---|
| `GPTBot` | OpenAI | "Disallowing GPTBot indicates a site's content should not be used in training generative AI foundation models." Search is `OAI-SearchBot` ([docs](https://developers.openai.com/api/docs/bots)) |
| `ClaudeBot` | Anthropic | The crawler that collects public web content that may train its models. Search is `Claude-SearchBot` ([docs](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler)) |
| `Google-Extended` | Google | Controls use in training Gemini models and grounding. It "does not impact a site's inclusion in Google Search nor is it used as a ranking signal" ([docs](https://developers.google.com/search/docs/crawling-indexing/google-common-crawlers)) |
| `Applebot-Extended` | Apple | Opts content out of training Apple's foundation models. "Webpages that disallow Applebot-Extended can still be included in search results" ([docs](https://support.apple.com/en-us/119829)) |
| `meta-externalagent` | Meta | Crawls for "training foundation AI models or improving products by indexing content directly" ([docs](https://developers.facebook.com/docs/sharing/webmasters/web-crawlers)) |
| `Amazonbot` | Amazon | Its data "may be used to train Amazon AI models". Search is `Amzn-SearchBot` ([docs](https://developer.amazon.com/amazonbot)) |
| `CCBot` | Common Crawl | The open web archive that many AI training sets are built from ([docs](https://commoncrawl.org/ccbot)) |

## Decision

Keep search crawling and ask the training crawlers to stay out:

- `User-agent: *` keeps `Allow: /`, so search engines and every other
  crawler still read everything.
- One group names the seven tokens above, each by its operator's
  documented spelling, with `Disallow: /`. One group of several
  `User-agent` lines is valid
  [RFC 9309](https://www.rfc-editor.org/rfc/rfc9309) (section 2.1).
- Search agents (Googlebot, Applebot, OAI-SearchBot, Claude-SearchBot,
  Amzn-SearchBot) are never named, so they stay in the `*` group.
- The list lives in `AI_TRAINING_AGENTS` in `build/site.rs`, and
  `tests/harness/robots.rs` checks both groups and the sitemap line.

## Consequences

- The site stays findable in Google, Bing, Apple, Kagi and the search
  features of ChatGPT and Claude, while asking their training crawlers
  not to use it.
- robots.txt is a request, not a lock. Crawlers that ignore it, or new
  ones not on the list, can still read the site. GitHub Pages can't set
  headers or block by IP, so this is as far as a static site can go.
- The Markdown source is public on GitHub, which has its own terms. The
  site's robots.txt doesn't cover the repo.
- The list will go stale as new crawlers appear. Adding one is a line in
  the build and a line in the test.

## Alternatives considered

- **Block every crawler.** That stops training, but it also takes the
  site out of search, and the only way to find it would be a link.
- **Keep allowing everything.** That's the scaffold default JC rejected.
- **Also block the AI search and user agents** (`OAI-SearchBot`,
  `Claude-SearchBot`, `ChatGPT-User`, `Claude-User`). Those cite and link
  to the page rather than train on it, and JC wanted search kept.
  Blocking them is a one-line change later if that changes.
