# 2026-11-11 — Verify the domain in Google Search Console by DNS TXT record

- Date: 2026-11-11
- Status: Accepted
- Decided by: JC, who asked whether to verify at all, what the risks are
  and which method is best, then picked "DNS TXT at Porkbun" over the
  HTML file, the HTML tag and skipping Search Console.

## Context

Google Search Console reports how a site does in Google Search:
which searches show it, which pages Google could not index,
alerts about indexing and spam problems, and which sites link to it.
It is optional.
Google says "You don't have to sign up for Search Console to be included in Google Search results,
but Search Console helps you understand and improve how Google sees your site"
([About Search Console](https://support.google.com/webmasters/answer/9128668)).
The site is already crawlable and names its sitemap in `robots.txt`,
so nothing here depends on it.

One tool is for owners only, the Removals tool,
which hides a page from results while it is fixed or taken down:
"You must own the property in Search Console to use this tool",
and "A successful request lasts only about six months"
([Removals and SafeSearch reports tool](https://support.google.com/webmasters/answer/9689846)).
That is worth having in case something is ever published by mistake.

To see any of this, a Google account has to prove it owns the site, because
"Site owners in Search Console have access to sensitive Google Search information about a site.
Site owners can also perform actions that can affect a site's presence and behavior on Google Search and other Google services"
([Verify your site ownership](https://support.google.com/webmasters/answer/9008080)).

Search Console has two kinds of property
([Add a website or platform property to Search Console](https://support.google.com/webmasters/answer/34592)):

- A URL-prefix property "Includes only URLs with the specified prefix,
  including the protocol (http/https)".
- A Domain property "Includes all subdomains (m, www, and so on)
  and multiple protocols (http, https, ftp)".
  It "requires you to verify ownership using DNS record verification".

The verification methods are an HTML file on the site, an HTML tag on the home page,
a Google Analytics tracking code, a Google Tag Manager snippet,
and a DNS record at the domain name provider.
Google says of the DNS record that it "is the only way to verify a Domain property"
([Verify your site ownership](https://support.google.com/webmasters/answer/9008080)).

The site is served at `www.dijonkitchen.org`, with the bare domain redirecting to it
(ADR 2026-10-14-www-as-canonical-host).
Its DNS is at Porkbun and mirrored in `docs/dns/`
(ADR 2026-10-21-porkbun-registrar-and-dns).
Its pages load only the site's own files
(ADR 2026-10-28-csp-meta-tag).

The bare domain already carries one `google-site-verification` TXT record.
It predates the 2026-10-11 move to GitHub Pages and came across with the rest of the zone.
Which Google account made it is not known.

## Decision

Verify `dijonkitchen.org` as a Domain property, with a TXT record on the bare domain.

- JC adds the `google-site-verification=…` TXT record that Search Console gives
  at Porkbun and presses Verify.
  Google's instruction for the host field is to
  "either leave this blank, or set as "@", as described in the documentation for your DNS provider".
- TXT, not CNAME.
  Google's rule is "If no CNAME record is found, then follow the TXT record instructions",
  and a bare domain with address records can't have one:
  "If a CNAME RR is present at a node, no other data should be present"
  ([RFC 1034 section 3.6.2](https://www.rfc-editor.org/rfc/rfc1034#section-3.6.2)).
- The record is mirrored in `docs/dns/dijonkitchen.org.zone`, next to the older one.
- The older token stays for now.
  Google says "If you use the same verification method,
  just be sure that you don't overwrite the verification tokens of any other owners".
  Once verified, JC checks the owners under Settings, Users and permissions.
  If the older token belongs to an account JC no longer uses,
  JC removes that owner and deletes its record, because otherwise
  "the removed owner will be able to re-verify ownership if they choose to"
  ([Managing owners, users, and permissions](https://support.google.com/webmasters/answer/7687615)).

## Consequences

- One property covers the bare domain and `www`, over http and https.
- Nothing is added to the built site: no file, no tag, no script.
  The pages and the Content Security Policy are unchanged,
  and a reader's browser sends Google nothing it did not send before.
- The record has to stay.
  "To stay verified, don't remove the DNS record from your provider,
  even after verification succeeds."
  Search Console "periodically checks if your verification token is still present and valid",
  and if it is gone "your permissions on that property will expire after a certain grace period".
  The zone file keeps the record from being dropped in a move to another DNS host.
- Verification does not depend on the web host,
  so it survives a move off GitHub Pages without a new token.
- The record is public, as every DNS record is.
  It shows that a Google account has claimed the domain. It does not name the account.
- The Google account is now worth protecting.
  Anyone signed in to it can read the search reports and hide pages with the Removals tool.
- Until the owners list has been checked, an unknown account is a verified owner too.

## Alternatives considered

- **HTML file on the site.**
  Verifies only the `https://www.dijonkitchen.org/` prefix.
  It puts a Google file among the site's own, and
  "Removing this verification file from your site will cause you to lose verification for the site",
  so verification would ride on every deploy and on the web host.
- **HTML tag on the home page.**
  The same single prefix, and a Google token in the source of the page every reader loads.
  "Search Console will periodically check for the presence of this tag."
- **Google Analytics or Tag Manager.**
  Both need Google's script on the home page.
  The site runs no third-party script and its policy forbids one
  (ADR 2026-10-28-csp-meta-tag).
- **Skip Search Console.**
  Google indexes the site either way.
  JC would go without the search and indexing reports and the Removals tool.
