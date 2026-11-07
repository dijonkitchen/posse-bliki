# 2026-11-07 — The fediverse handle on the bliki's own domain

- Date: 2026-11-07
- Status: Proposed, awaiting JC's choice
- Raised by: JC, passing on Bridgy Fed's check of the site:
  "Current vs expected:
  https://www.dijonkitchen.org/.well-known/webfinger?resource=acct:dijonkitchen.org@dijonkitchen.org
  vs
  https://fed.brid.gy/.well-known/webfinger?resource=acct:dijonkitchen.org@dijonkitchen.org"

## Context

Bridgy Fed bridges the bliki into the fediverse
(ADR 2026-10-11-webmentions-at-deploy-time).
Its account for the site is the bare domain, `dijonkitchen.org`,
because Bridgy Fed drops a leading `www.` when the bare domain serves the same site
([`Web.verify`](https://github.com/snarfed/bridgy-fed/blob/main/web.py)).

By default the handle is `@dijonkitchen.org@web.brid.gy`.
To make it `@dijonkitchen.org@dijonkitchen.org`,
the domain has to redirect `/.well-known/webfinger` and `/.well-known/host-meta`,
query string and all, to the same paths on `https://fed.brid.gy/`
([Bridgy Fed docs](https://fed.brid.gy/docs#fediverse-enhanced)).
Bridgy Fed checks this by requesting
`https://dijonkitchen.org/.well-known/webfinger?resource=acct:dijonkitchen.org@dijonkitchen.org`,
following redirects, and comparing where it lands with the fed.brid.gy URL.
Only then does it use the domain in the handle
([`Web.handle_as`](https://github.com/snarfed/bridgy-fed/blob/main/web.py)).

Today the bare domain's A and AAAA records point at GitHub Pages,
which redirects every path to `www` and keeps the query string
([GitHub docs](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/about-custom-domains-and-github-pages)).
`www` has no webfinger file, so the check ends at a 404 on `www`.
GitHub Pages serves files and has no redirect rules,
and a static webfinger file wouldn't pass, since the check wants a redirect.

Bridgy Fed's docs say the handle's server is "purely cosmetic":
followers, replies and likes work the same either way.
The difference that lasts is that a handle on the bliki's own domain
stays the same if the site ever leaves Bridgy Fed,
the reason the site sits on its own domain at all
(ADR 2026-10-14-www-as-canonical-host).
The bridged account has no followers yet, so changing it now costs nothing.

ADR 2026-10-14-www-as-canonical-host already expected the bare domain
might one day be "a redirect-only service".

## Decision

If JC chooses it:

- The bare domain moves to a redirect-only site on Netlify,
  the host Bridgy Fed gives instructions for.
  `apex/_redirects` holds its three rules:
  host-meta and webfinger to fed.brid.gy (302, as Bridgy Fed's docs give them),
  and every other path to `https://www.dijonkitchen.org/` (301, as Pages did).
  Netlify forwards query strings on these rules
  ([Netlify docs](https://docs.netlify.com/manage/routing/redirects/redirect-options/)).
- JC deploys the folder by hand (Netlify Drop),
  not by linking the repo,
  so Netlify gets no access to the repo and adds nothing to its pull requests.
  The folder changes rarely, and each deploy is one drag.
- At Porkbun, the bare domain's four A and four AAAA records for GitHub
  become one A record, `75.2.60.5`, Netlify's load balancer
  ([Netlify docs](https://docs.netlify.com/manage/domains/configure-domains/configure-external-dns/)).
  `www`, email and every other record stay as they are.
  The CAA record already allows Let's Encrypt, which Netlify uses
  ([Netlify docs](https://docs.netlify.com/manage/domains/secure-domains-with-https/https-ssl/)).
- After that, JC presses "Update profile" on Bridgy Fed,
  which re-runs the check and switches the handle.
- A harness test keeps the redirect rules in order
  and keeps the catch-all pointed at `base_url`.

## Consequences

- The fediverse handle becomes `@dijonkitchen.org@dijonkitchen.org`.
- The bare domain depends on Netlify.
  If Netlify drops it, the bare domain stops redirecting,
  but `www`, where every link points, keeps working,
  and the uptime check on the bare domain (statup) notices.
- Netlify lists `www.dijonkitchen.org` next to the bare domain,
  since it adds both for any domain
  ([Netlify docs](https://docs.netlify.com/manage/domains/manage-domains/manage-multiple-domains/)).
  Its DNS keeps pointing at GitHub, so Netlify never serves it.
- Changing a redirect means editing `apex/_redirects` and dragging the folder again.
- The handle's name part is the domain.
  Bridgy Fed would use a custom name, such as `jc`,
  if the h-card had an `acct:` URL
  ([`Web.username`](https://github.com/snarfed/bridgy-fed/blob/main/web.py)),
  but `@jc@dijonkitchen.org` would spell out the email address
  the site keeps in character codes (ADR 2026-11-06-coded-email),
  so this ADR leaves the name alone.

## Alternatives considered

- **Keep `@dijonkitchen.org@web.brid.gy`.**
  It works today, costs nothing and needs no new account.
  Bridgy Fed's page keeps showing the redirect check as failed.
  It is the choice if JC doesn't want the bare domain on another host.
- **Move DNS to Cloudflare** and use its redirect rules.
  Free and robust, but it moves every record off Porkbun
  (ADR 2026-10-21-porkbun-registrar-and-dns) to redirect two paths.
- **Move the whole site to Netlify.**
  It could serve both the site and the redirects,
  but it reverses the GitHub Pages deploy for one feature.
- **A static webfinger file on Pages.**
  Pages ignores the query string,
  and Bridgy Fed's check wants a redirect, not a copy, so it wouldn't pass.
