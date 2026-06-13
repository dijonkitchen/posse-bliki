# 0006 — Dependency exit policy

- Date: 2026-06-13
- Status: Accepted

## Context

Enshittification is the default trajectory of hosted platforms: first they
are good to users, then they squeeze users to serve business customers,
then they squeeze everyone. This site exists partly as a refusal of that
cycle — but it still *uses* hosted services (GitHub, webmention.io,
Bridgy Fed). Using a service is fine; being unable to leave it is not.

The existing specs guarantee open *output* (microformats2, RSS, JSON
Feed). Nothing yet guarantees that every operational dependency can be
walked away from with data, identity, and audience intact.

## Decision

Every external dependency must satisfy three exit criteria:

1. **Open-standard interface** — the service is spoken to via a protocol
   or format anyone can implement, never a proprietary API we depend on.
2. **At least one drop-in alternative** — a known replacement exists today.
3. **Data-export path** — everything the service holds for us can be
   retrieved, or the service holds nothing canonical in the first place.

| Dependency | Open-standard interface | Drop-in alternatives | Export path |
|---|---|---|---|
| GitHub (repo hosting) | git protocol | Codeberg, sourcehut, self-hosted Forgejo, any git remote | `git clone` *is* the full export — git is distributed |
| GitHub Pages | static HTTP | Netlify, Cloudflare Pages, any web server | `make build`, copy `public/` anywhere |
| GitHub Actions | none needed — the CI contract is `make ci` | Woodpecker, Forgejo Actions, local cron | the workflow is a one-step `make ci` wrapper (test-enforced) |
| webmention.io | Webmention + Pingback (W3C) | self-hosted receiver (e.g. webmentiond), other hosted receivers | export API (`/api/mentions.jf2`); endpoint is one config value (test-enforced) |
| Bridgy Fed | ActivityPub + Webmention (W3C) | other feed-to-fediverse bridges, self-hosted bridge, plain feed-based POSSE | the site is the source of truth; `u-syndication` links live in front-matter; followers are re-acquirable |
| DNS / registrar | DNS | any registrar (domain transfer) | the owned domain is *the* portability primitive; `base_url` is one config value |
| uv / Python | PEP 621 `pyproject.toml` | pip, pdm, poetry — or rewrite the build in any language (ADR 0001/0002: spec + tests define behaviour) | `uv.lock` is regenerable; the build is one replaceable script |

Adding a new external service requires a new ADR adding a row to this
table. A dependency that cannot fill all three columns is rejected.

Machine-testable consequences of this policy live in
[`spec/portability.md`](../../spec/portability.md); the per-dependency
migration runbook lives in [`docs/exit-plan.md`](../exit-plan.md).

## Consequences

- Hosted services stay safe to use: the cost of leaving any of them is
  one config edit, one rebuild, one deploy — not a migration project.
- The placeholder `base_url` is provably harmless: the harness builds
  with an alternate config and asserts zero residue of the old provider.
- Some friction is accepted: we forgo provider-specific features (GitHub
  Pages analytics, webmention.io widgets) that would deepen coupling.

## Alternatives considered

- **Trust the providers.** Rejected — enshittification is not a risk to
  hedge but a lifecycle to expect. Stability since 2014 (webmention.io)
  is evidence, not a guarantee.
- **Self-host everything now.** Rejected — running a VPS, a webmention
  receiver, and an ActivityPub server is real cost with no current
  benefit. Documented, tested exit paths make hosted services safe
  enough; we pay the self-hosting cost only if an exit is forced.
