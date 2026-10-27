# 2026-10-21 — Porkbun registers the domains and serves DNS; the zone lives in git

- Date: 2026-10-21
- Status: Accepted
- Decided by: JC, after comparing registrars and DNS hosts ahead of the
  Gandi renewal (all three domains were due 2027-02-03). Moved
  dijonkitchen.com and .net on 2026-10-22 and dijonkitchen.org on
  2026-10-27.

## Context

dijonkitchen.org, .com and .net were registered at Gandi, with DNS on
Gandi LiveDNS. Since its 2023 sale Gandi has raised prices and dropped the
free mailbox, and its "no bullshit" stance no longer sets it apart. The
domain carries the site (GitHub Pages) and email (Fastmail), so whoever
controls the registrar or DNS account controls both. For a personal domain
the realistic threat is account takeover, not a flaw in a DNS server.

## Decision

- **Porkbun** is the registrar for all three domains and serves their DNS
  on its default nameservers. One account to secure, with a passkey, and
  logged in with an email that isn't on this domain, so a hijacked domain
  can't reset its own account.
- **dijonkitchen.org** carries the site and Fastmail. The full zone is in
  [`docs/dns/dijonkitchen.org.zone`](../dns/dijonkitchen.org.zone).
  New with the move: a CAA record allowing only Let's Encrypt, and a
  DMARC record in report-only mode (`p=none`), to be tightened once the
  reports are clean. Gandi's catch-all `*` redirect and its webmail and
  blog records were dropped.
- **dijonkitchen.com and .net** are Porkbun URL forwards (301, path kept,
  www included) to https://www.dijonkitchen.org, with no-mail records
  (null MX, `v=spf1 -all`, DMARC `p=reject`) so nobody can send mail as
  them.
- **No DNSSEC.** Fastmail doesn't support it on custom domains
  (https://www.fastmail.help/hc/en-us/articles/7882212586511-DNSSEC).
  DKIM, SPF, DMARC and CAA cover the risks that matter here.
- The zone file in git is the exit plan: any DNS host can load it, so
  moving DNS is a nameserver change, independent of the registrar.

## Consequences

- Renewals are about $11–13 a year per domain, with WHOIS privacy included.
  The transfers added a year: all three now expire 2028-02-03.
- Porkbun's nameservers run on Cloudflare's network, so DNS shares
  Cloudflare's outages. Accepted for now; moving DNS elsewhere is a
  nameserver change plus loading the zone file.
- An edit at Porkbun should land in the zone file in the same change, or
  the file stops being a usable backup.
- The uptime monitor (dijonkitchen/statup) checks the .com and .net
  forwards as well as the site.

## Alternatives considered

- **Stay at Gandi.** Higher renewals and a direction since the sale that
  no longer matches why it was chosen.
- **Cloudflare Registrar.** At-cost pricing, but it requires Cloudflare
  DNS, so the registration and DNS can't be split later.
- **deSEC for DNS, Porkbun for the registration.** Open source, free,
  nonprofit and DNSSEC by default. Turned down because it rests on one
  main maintainer and one main sponsor, and the domain carries email.
  Still the first choice if DNS moves off Porkbun.
- **Bunny DNS or Hetzner DNS, split from the registrar.** Independent of
  Cloudflare, but a second account to secure for little gain today.
- **Fastmail's own DNS.** Basic, and it would tie DNS to the email
  provider.
