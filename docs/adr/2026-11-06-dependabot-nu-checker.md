# 2026-11-06 — Dependabot proposes Nu HTML Checker updates

- Date: 2026-11-06
- Status: Accepted (JC chose "Merge #73, then track Nu checker", 2026-11-06)
- Decided by: JC, asking whether any other dependency was missing from
  Dependabot after [ADR 2026-11-05-dependabot-lighthouse-updates](2026-11-05-dependabot-lighthouse-updates.md).
- Supersedes: how [2026-10-11-nu-html-checker-in-ci](2026-10-11-nu-html-checker-in-ci.md)
  pins and downloads vnu, and the "Nu checker stays hand-bumped" bullet of
  [2026-10-23-dependabot-rust-toolchain](2026-10-23-dependabot-rust-toolchain.md).

## Context

The Nu HTML Checker (vnu) was the last CI tool Dependabot couldn't update.
The Makefile pinned it with `VNU_VERSION` and `VNU_SHA512`, downloaded the
`vnu-jar` npm package with curl, and checked the hash. Dependabot doesn't
read Makefiles, so a new release only arrived when someone remembered to
look.

The rest of what's left can't be bumped by Dependabot in any form: the
Node and Java majors in the workflows, the runner image and its Chrome,
the Unicode data version for `make tables`, and the fonts checked into
the repo ([ADR 2026-11-05-nix-still-not-worth-it](2026-11-05-nix-still-not-worth-it.md)).

## Decision

- Lock vnu the same way as Lighthouse: `scripts/vnu/package.json` names
  `vnu-jar` at the exact version, and `scripts/vnu/package-lock.json` holds
  its registry URL and SHA-512 integrity hash. This is the same release
  (26.9.16) and the same tarball: the lockfile's hash is the Makefile's
  old `VNU_SHA512`, base64-encoded.
- `make validate` installs it with `npm ci --ignore-scripts` and runs
  `scripts/vnu/node_modules/vnu-jar/build/dist/vnu.jar`. npm checks the
  download against the lockfile, and the Makefile reinstalls only when the
  lockfile changes. `--ignore-scripts` matters here: the package's
  postinstall downloads a Java runtime, and CI installs Java with
  `actions/setup-java` instead.
- The `html` job adds `actions/setup-node` (Node 22, pinned by SHA like the
  Lighthouse job).
- `.github/dependabot.yml` gets an `npm` entry for `/scripts/vnu`: monthly,
  seven-day cooldown, `chore(deps)` prefix, the same as the others.
- `tests/harness/supply_chain.rs` fails if any `scripts/*/package-lock.json`
  has no Dependabot npm entry, so a CI tool added later can't be forgotten.

## Consequences

- A new vnu release shows up as a PR about a week after release, at most
  once a month. Its `html` job shows whether new checks flag existing
  pages before it merges, which ADR 2026-10-11 called "a reason to look,
  not to pin forever".
- The weekly Dependabot review merges these PRs along with the others when
  they're green.
- Running `make validate` locally now needs Node as well as Java.
- The download is the same size, about 30 MB per CI run.

## Alternatives considered

- **Keep curl, read the version and hash from the lockfile.** Keeps Node
  out of `make validate`, but the Makefile would parse JSON with sed and
  convert the base64 hash for `shasum`. That's more fragile than letting
  npm do the check it already does for Lighthouse.
- **A Maven `pom.xml` for `nu.validator:validator`.** Dependabot reads
  Maven too, but that means a build file for a tool we only download.
- **Keep it hand-bumped.** It works, but it's the one CI tool that would
  silently age.
