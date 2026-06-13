# Portability and enshittification resistance

The site must always be exit-able with data, identity, and audience
intact. Providers — hosts, CI, webmention receivers, fediverse bridges —
are commodities behind open standards, swappable with a config edit and
a rebuild. The policy and per-dependency exit criteria are in
[ADR 0006](../docs/adr/0006-dependency-exit-policy.md); the migration
runbook is [`docs/exit-plan.md`](../docs/exit-plan.md).

## Machine-enforced invariants

Enforced by `tests/test_portability.py` unless noted otherwise.

| # | Invariant |
|---|---|
| P1 | The output tree is pure static files: only `.html`, `.xml`, `.json`, `.txt`, `.css`. Anything a plain web server can't serve as-is is a build error. |
| P2 | No JavaScript in output. (Enforced by `tests/test_html_valid.py::test_no_script_tags`.) |
| P3 | No third-party active content: no `<iframe>`, `<embed>`, `<object>`; every stylesheet is root-relative; no `preconnect`/`dns-prefetch` hints. Passive external images in authored content are permitted — they degrade, not surveil. |
| P4 | Provider swap leaves zero residue: building with an alternate config (different `base_url`, webmention endpoints, author) yields output with no occurrence of the default providers' hostnames. Service URLs enter output only via config, never templates. |
| P5 | Every absolute self-URL — canonical links, RSS `link`/`guid`/self, JSON Feed `home_page_url`/`feed_url`/`id`, sitemap `<loc>`, robots `Sitemap:` — derives solely from `base_url`. (Canonical also enforced by `tests/test_indieweb_links.py::test_canonical_matches_url`.) |
| P6 | Feeds liberate full content: every RSS item carries the full rendered body in `<content:encoded>`; every JSON Feed item carries full `content_html` (the latter enforced by `tests/test_jsonfeed.py`). Readers never need to visit the site of record to get the words. |
| P7 | Content is portable: every file under `content/` (outside dot-directories) is UTF-8 markdown. The vault opens in any editor, forever. |
| P8 | CI is a thin wrapper: `.github/workflows/ci.yml` does nothing but invoke `make ci`. The CI provider knows nothing the Makefile doesn't. |

## Review-enforced invariants

Not machine-testable; enforced by the change process (ADR 0006).

- Every external dependency has a row in the ADR 0006 exit table and a
  section in `docs/exit-plan.md`. No new service without a new ADR.
- The domain is a single config value (`base_url`). Nothing else in the
  repo may assume a particular host or registrar.
- Syndication is POSSE, never the reverse: canonical content lives here;
  copies elsewhere point back via `u-syndication` front-matter.
