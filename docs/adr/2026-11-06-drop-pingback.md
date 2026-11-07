# 2026-11-06 — Drop Pingback and keep Webmention

- Date: 2026-11-06
- Status: Accepted
- Decided by: JC, answering "Remove ping back" when asked about the
  scaffold choices nobody had decided.

## Context

Every page advertised two ways to be told about links to it:
`<link rel="webmention">` and `<link rel="pingback">`, both pointing at
webmention.io. Pingback came with the first scaffold (PR #1, ADR
2026-05-10-indieweb-standards-supported) because webmention.io accepts
it. Nobody chose it.

- [Pingback 1.0](https://www.hixie.ch/specs/pingback/pingback) (2002)
  sends the source and target in an XML-RPC call.
- [Webmention](https://www.w3.org/TR/webmention/), a W3C Recommendation
  since 12 January 2017, "began as a simplified version of the Pingback
  spec". It swaps the XML-RPC payload for a plain form post, so it "was
  not vulnerable to accidentally exposing other parts of a system's code
  via XML-RPC" (section 1.2).
- The IndieWeb wiki calls Pingback "a legacy XML-RPC based protocol" and
  tells developers to "implement receiving webmention instead". It also
  notes that "WordPress is perhaps the top target of Pingback spam"
  ([indieweb.org/pingback](https://indieweb.org/pingback)).
- Bridgy and Bridgy Fed, the services this site relies on for replies and
  syndication (ADR 2026-10-11-webmentions-at-deploy-time), send
  webmentions, not pingbacks.

## Decision

Stop advertising Pingback. Pages keep `<link rel="webmention">` only.
The build no longer has a `pingback` setting, and the harness fails if
any page advertises `rel=pingback`. spec/indieweb.md lists Pingback as
out of scope.

## Consequences

- Each page's `<head>` loses a line, and the config one setting.
- A site that only sends pingbacks (mostly older WordPress installs) can
  no longer notify this one. Modern senders discover the webmention
  endpoint instead.
- webmention.io still has its XML-RPC endpoint, but nothing points
  anyone to it.

## Alternatives considered

- **Keep both.** It costs one line per page, but it advertises a legacy
  protocol that draws spam, for senders this site doesn't need.
