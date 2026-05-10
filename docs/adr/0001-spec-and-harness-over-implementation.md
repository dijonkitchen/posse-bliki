# 0001 — Spec + harness as the durable artifact, not the build code

- Date: 2026-05-10
- Status: Accepted

## Context

This site is meant to last 10+ years and to be maintained partly (eventually
mostly) by AI agents. Traditional notions of "maintainable code" — small,
stable language, few deps — optimise for human comprehension of the
implementation. Under agentic maintenance, the implementation becomes cheap
and rewritable; what's expensive is **knowing whether a rewrite is correct**.

## Decision

Treat outcome specs and the test harness as the long-lived artifacts. The
build code is replaceable.

- `spec/` declares what "correct" means in prose and machine-readable form
  (JSON Schema for content, prose contracts for output and feeds).
- `tests/` enforces those specs as automated checks: HTML5 validity,
  microformats2 conformance, RSS/JSON Feed validity, link integrity,
  build idempotency.
- `build/` is the smallest implementation that turns the harness green.
  Rewriting it in a different language, framework, or library is acceptable
  and expected — as long as the harness stays green.

The contract for any agent (or human) editing this repo:

> Read `spec/`. Run `tests/`. Edit `build/` only. Never edit a snapshot
> without also editing the spec it pins.

## Consequences

- New features arrive as: spec change → failing test → build change.
- Migrations between languages or frameworks are mechanical — the harness
  is the migration acceptance criterion.
- The repo is self-describing for agents: a fresh agent can read
  `AGENTS.md` and `spec/` and produce correct work without prior context.
- Cost: writing tests for things that "obviously work" feels redundant
  short-term. We accept this — the harness is the ratchet that prevents
  regression as agents iterate.

## Alternatives considered

- **Rely on a popular SSG (Quartz, Hugo, 11ty).** Rejected — see
  [ADR 0002](0002-no-quartz-no-vendored-ssg.md). Their behaviour is not
  specified; "correct" means "what the SSG happens to do this version."
- **Hand-write a small SSG without specs/tests.** Rejected — without a
  harness, an agent has no way to verify a refactor preserved behaviour,
  and no way to know when a feature is "done."
