# 2026-10-23 — Dependabot bumps the Rust toolchain; Renovate stays out

- Date: 2026-10-23
- Status: Accepted
- Decided by: JC, comparing Dependabot, Renovate and other update bots,
  then asking to verify Dependabot's toolchain support and close the
  Renovate onboarding PR (#3).

## Context

Dependabot already keeps the SHA-pinned actions current
(ADR 2026-10-14-supply-chain-pinned-actions). The build has no crates
(ADR 2026-10-06-rust-build-zero-dependencies), so the only other moving
version is the compiler, pinned in `rust-toolchain.toml`
(ADR 2026-10-11-rust-only-toolchain). Nothing proposed bumping it, so it
would only move when someone remembered to.

Since August 2025 Dependabot has a `rust-toolchain` ecosystem that bumps
`rust-toolchain.toml` (version updates only, not security alerts):
https://github.blog/changelog/2025-08-19-dependabot-now-supports-rust-toolchain-updates/

## Decision

- Add a `rust-toolchain` entry to `.github/dependabot.yml`: monthly,
  seven-day cooldown, `chore(deps)` prefix, the same as the actions entry.
- Keep Dependabot as the only update bot. Renovate's onboarding PR (#3)
  is closed unmerged.
- The Nu checker (`VNU_VERSION` and its SHA-512 in the Makefile) and
  Lighthouse (`make lighthouse`) stay hand-bumped. Neither bot reads a
  Makefile without custom regex rules.

## Consequences

- A new stable Rust shows up as a PR about a week after release, at most
  once a month. CI on that PR (`make ci`, clippy, rustfmt) shows whether
  the new compiler's lints need code changes before merging.
- Dependabot opens the toolchain PR separately from the actions group,
  since a group cannot span ecosystems.
- Docs that name "1.97" describe the version a decision was made with;
  they are history, not the current pin.

## Alternatives considered

- **Renovate.** More flexible: custom regex managers could also bump the
  Makefile pins. It needs a third-party app with write access to the
  repo and a larger config, to save a few hand edits a year. That runs
  against the reason actions are pinned in the first place.
- **Self-hosted Renovate in Actions.** No third-party app, but another
  workflow and token to maintain.
- **pinact or ratchet.** They pin SHAs, which the harness already
  enforces; they don't propose updates on a schedule.
- **Snyk or Socket.** Vulnerability scanners rather than updaters, aimed at
  package ecosystems this repo doesn't use.
- **Leave the toolchain manual.** Fine, but it silently ages; a monthly PR
  costs one review.
