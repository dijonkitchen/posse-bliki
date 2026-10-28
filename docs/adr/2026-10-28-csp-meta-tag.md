# 2026-10-28 — A strict Content Security Policy in a meta tag

- Date: 2026-10-28
- Status: Accepted
- Decided by: JC, after a Cloudflare Radar URL scan of
  www.dijonkitchen.org flagged missing security headers. JC chose this
  fix plus a `security.txt` (its own change).

## Context

Security scanners such as Cloudflare Radar check for response headers:
Content-Security-Policy, Strict-Transport-Security, X-Frame-Options,
X-Content-Type-Options, Referrer-Policy and Permissions-Policy. GitHub
Pages doesn't let a site set response headers, so none of them can be
sent from here.

Of those, only a Content Security Policy can also be given in the page,
as `<meta http-equiv="Content-Security-Policy">`. The rest either need a
real header (HSTS, X-Frame-Options, X-Content-Type-Options, and a CSP's
`frame-ancestors`) or add little here: http already redirects to https,
and with no logins or forms there's nothing to clickjack. The browser's
default Referrer-Policy (`strict-origin-when-cross-origin`) is already
sensible.

The site loads only its own files: `style.css`, the theme stylesheets,
the self-hosted fonts, the icons, the manifest and `/style-picker.js`.
Markdown's raw HTML is escaped, so notes can't add scripts, styles or
embeds.

## Decision

Every page's `<head>` carries, right after the charset so it covers
everything that follows:

```
default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self';
font-src 'self'; manifest-src 'self'; base-uri 'none'; form-action 'none'
```

Anything not named is blocked. `base-uri` and `form-action` don't fall
back to `default-src`, so they're set too.

A harness test (`tests/harness/csp.rs`) checks every page has the policy
first, and that nothing the build emits needs more: no inline `style=`,
`<style>`, `on*=` handlers, embeds, or scripts, images, stylesheets, fonts
or icons from other hosts, in the HTML or in any stylesheet's `url()`.

## Consequences

- If anything ever injected markup into a page, browsers would refuse to
  run its scripts or load its styles and images from elsewhere.
- Embedding something from another site (a video, an image hotlinked from
  another host, an inline style) now fails the build. Allowing it means
  widening the policy for that host in a new ADR, and updating the test.
- Scanners that only read headers will still report a missing CSP. A
  meta policy can't set `frame-ancestors`, `report-uri` or `sandbox`.
- Some browser extensions that inject scripts or styles may be blocked
  on this site; that's the policy working.

## Alternatives

- **Cloudflare (or another proxy) in front of Pages** to add real headers.
  It would add a third party between readers and the site, and put the
  Pages TLS certificate and the www/apex redirects behind it. Not worth it
  for a static site with no logins.
- **Leave it.** A static site with no third-party code is already low
  risk. But the meta tag costs one line per page and no performance, and
  the test keeps it honest.
