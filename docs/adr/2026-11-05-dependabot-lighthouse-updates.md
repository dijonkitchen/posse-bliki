# 2026-11-05 — Dependabot proposes Lighthouse updates

- Date: 2026-11-05
- Status: Accepted (JC chose "Skip Nix, add Lighthouse to Dependabot", 2026-11-05)
- Decided by: JC, asking whether Nix should manage every dependency;
  Claude recommended closing this gap instead
  ([ADR 2026-11-05-nix-still-not-worth-it](2026-11-05-nix-still-not-worth-it.md)).

## Context

Lighthouse is CI's only npm package. It and its transitive packages are
locked in `scripts/lighthouse/package-lock.json` and installed with
`npm ci` ([ADR 2026-10-14-mobile-first](2026-10-14-mobile-first-readable-output-lighthouse.md),
spec/performance.md).

`.github/dependabot.yml` lists only GitHub Actions and the Rust
toolchain, so the only Lighthouse PRs so far came from Dependabot
security alerts. That's how 12.8.2 became 13.6.0 (#54). Without an
alert, Lighthouse stays at whatever version it was last bumped to, and
newer audits and Chrome support never arrive. [ADR
2026-10-23-dependabot-rust-toolchain](2026-10-23-dependabot-rust-toolchain.md)
expected Lighthouse to be bumped by hand in the Makefile, but it moved
to a lockfile that Dependabot reads without custom rules.

## Decision

Add an `npm` entry for `/scripts/lighthouse` to `.github/dependabot.yml`,
set up like the other entries: monthly, seven-day cooldown, `chore(deps)`
prefix.

## Consequences

- A new Lighthouse release shows up as a PR about a week after release,
  at most once a month. Security alerts still open PRs right away, as
  before.
- CI's Lighthouse jobs run the new version on the fixed pages, so the PR
  shows whether a release changes scores before it merges. The script
  only reads category scores, so a major release is safe to merge when
  that job is green.
- The weekly Dependabot review merges these PRs along with the others.
- The Nu checker (`VNU_VERSION` and its SHA-512 in the Makefile) is still
  bumped by hand.

## Alternatives considered

- **Leave it to security alerts.** Lighthouse would stay current only when
  one of its packages had a known vulnerability.
- **Weekly instead of monthly.** More PRs to review for a CI-only tool,
  and out of step with the other entries.
