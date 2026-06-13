# Exit plan

Per-dependency migration runbook. Policy and exit criteria:
[ADR 0006](adr/0006-dependency-exit-policy.md). Machine-enforced
invariants: [`spec/portability.md`](../spec/portability.md).

The common shape of every exit: edit one config value in
`default_config()` (`build/build.py`), run `make ci`, deploy `public/`.
The harness proves the swap is clean (`tests/test_portability.py`).

## GitHub — repository hosting

**Warning signs**: paid tiers gating core git features, AI training
opt-outs removed, Actions minutes squeezed, account lockouts.

**Exit**: `git clone` already on every machine that has touched the
repo is the full export. Add a new remote (Codeberg, sourcehut,
self-hosted Forgejo), `git push --mirror`, done. Issues/PR history is
the only GitHub-only data; this repo's durable records are ADRs and
commits, deliberately not issues.

**Lost**: GitHub stars, PR history. Nothing canonical.

## GitHub Pages — hosting

**Warning signs**: bandwidth caps, forced github.io branding, TLS or
custom-domain restrictions.

**Exit**: `make build`, copy `public/` to any static host (Netlify,
Cloudflare Pages, a $4 VPS with nginx, IPFS). Repoint DNS. The output
is pure static files (P1) with no host assumptions (P4/P5).

**Lost**: nothing.

## GitHub Actions — CI

**Warning signs**: free-tier minutes cut, lock-in features pushed.

**Exit**: the workflow is a one-step `make ci` wrapper (P8). Recreate
that single step on Woodpecker, Forgejo Actions, or a cron job. Local
`make ci` is always the reference implementation.

**Lost**: nothing.

## webmention.io — webmention/pingback receiver

**Warning signs**: signup closed, export API removed, ads in the
dashboard, mentions held hostage.

**Exit**:

1. Export everything: `https://webmention.io/api/mentions.jf2?domain=<domain>&token=<token>&per-page=...` (paginate); commit the JSON to this repo for safekeeping.
2. Stand up or sign up for a replacement receiver (e.g. self-hosted [webmentiond](https://github.com/zerok/webmentiond)).
3. Change `webmention.endpoint` and `webmention.pingback` in `default_config()`.
4. `make ci` — the provider-swap test (P4) guarantees no residue — and deploy.

**Lost**: nothing, if the export runs before shutdown. Mentions are
display-only enrichment; the posts themselves never lived there.

## Bridgy Fed — ActivityPub bridge

**Warning signs**: service shutdown notice, paid gating of follows,
protocol drift.

**Exit**: the site is the POSSE source of truth; Bridgy Fed only
mirrors it. Options, in order of effort: point a different feed-based
bridge at `/index.xml`; self-host a bridge; or drop fediverse presence
and keep RSS/JSON Feed. `u-syndication` URLs already recorded in
front-matter remain valid history even if the copies die.

**Lost**: fediverse followers (they follow the bridge identity).
Announce the move in a final bridged post; followers re-follow the new
identity. This is the one genuinely lossy exit — acceptable because
the audience contract is "follow the feed", not "follow the platform".

## DNS / registrar

**Warning signs**: renewal price hikes, transfer-lock dark patterns.

**Exit**: standard domain transfer (get auth code, unlock, transfer).
The domain itself is the portability primitive — keep it renewed, with
auto-renew and an up-to-date payment method. `base_url` is one config
value; nothing else in the repo knows the host (P4/P5).

**Lost**: nothing, as long as the domain never lapses. Losing the
domain is the only unrecoverable failure in this design; treat renewal
as the single most important operational task.

## uv / Python toolchain

**Warning signs**: license change, abandonment.

**Exit**: `pyproject.toml` is standard PEP 621 — pip, pdm, or poetry
can install it today. Or rewrite the build entirely (ADR 0001/0002):
`spec/` + `tests/` define correctness; the build is a disposable
implementation detail.

**Lost**: nothing.

## Fire drill

Once a year (or after any major change): clone fresh on a clean
machine, `make ci`, serve `public/` with `python -m http.server`, click
around. If that works, every exit above is still real.
