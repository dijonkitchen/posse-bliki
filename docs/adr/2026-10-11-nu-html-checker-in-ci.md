# 2026-10-11 — Nu HTML Checker in CI

- Date: 2026-10-11
- Status: Accepted
- Decided by: JC, asking for an independent validator in its own PR.

## Context

The harness checks the HTML the build emits with a strict std-only parser
written for this repo (`tests/harness/support/html.rs`, ADR 0010). Both the build and that checker come from the same reading of the
spec, so a rule misread in one can be misread the same way in the other.

The W3C Nu HTML Checker (vnu) is the reference conformance checker behind
validator.w3.org. It implements the whole HTML standard and also flags
things a parser doesn't, such as skipped heading levels and sections
without headings. Running it over the built site once found three posts
that jumped from `<h1>` to `<h3>` and a home page with no `<h1>`; this PR
fixes them.

## Decision

Run vnu over every built page in CI, as its own `html` job:

- `make validate` builds the site, downloads a pinned `vnu.jar` from the
  npm package `vnu-jar`, checks it against a SHA-512 in the Makefile, and
  runs it with `--Werror`, so warnings fail like errors.
- One message is filtered: "Trailing slash on void elements". It is
  `info`, not an error. The Markdown renderer writes `<img … />` because
  it mirrors markdown-it's output, and the slash is harmless with quoted
  attributes.
- The job installs Java with `actions/setup-java` (Temurin 21).
- It is CI-only. It is not part of `make ci`, the build, or `tests/`. The
  build stays std-only Rust (ADR 0007) and offline, and `make ci` still
  needs nothing but the repo's own toolchain.

### Feeds

vnu checks HTML only, so the RSS feed and JSON Feed need their own check.
The W3C Feed Validation Service is the RSS equivalent, and one request per
push would be a fair load. But it sits behind a Cloudflare bot challenge
that answers automated requests with a 429 and a JavaScript page, so CI
can't call it. Running it ourselves would bring Python back into CI, which
ADR 0010 removed. Instead the Rust harness (`tests/harness/rss.rs`) checks
the RSS 2.0 rules that service enforces: required channel elements, absolute
URLs, a single matching `atom:link rel="self"`, unique permalink guids,
real RFC 822 dates with the right weekday, and no unknown elements. JSON
Feed has no official validator; `tests/harness/jsonfeed.rs` checks it
against the spec.

## Consequences

- Real conformance and outline errors fail a PR, independent of our own
  parser.
- Contributors need Java to run `make validate` locally. Everything else
  still works without it.
- Updating vnu means changing `VNU_VERSION` and `VNU_SHA512` together. A
  new release can add checks and turn CI red on unchanged content; that is
  a reason to look, not to pin forever.
- The job downloads about 30 MB from npm per run.

## Alternatives considered

- **Post pages to validator.w3.org/nu.** No Java, but it sends every page
  to a shared public service on every push and is rate-limited.
- **A third-party GitHub Action wrapping vnu.** Less Makefile, but CI
  would run something `make validate` can't reproduce locally.
- **Run it inside `make ci`.** Every contributor and cloud session would
  need Java and network access for the core loop. Keeping it a separate job
  keeps `make ci` fast and dependency-free.
- **Write the missing checks (heading order) into our own harness.** We may
  still add the ones that matter most, but that repeats the shared-reading
  problem this check exists to catch.
