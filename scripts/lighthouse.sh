#!/bin/sh
# Lighthouse budget for the built site (spec/performance.md, ADR 2026-10-14-mobile-first).
#
# CI-only, like the HTML checker: it needs Node and Chrome, which the build
# and `make ci` never do. Builds and serves the site on 127.0.0.1, audits a
# few representative pages with Lighthouse's default mobile profile, and
# fails if any category scores below its budget.
#
# usage: scripts/lighthouse.sh [PORT]
set -eu

PORT="${1:-8090}"
OUT="${OUT:-_lighthouse/site}"
REPORTS="${REPORTS:-_lighthouse}"
# Lighthouse and everything it pulls in are locked in
# scripts/lighthouse/package-lock.json; `npm ci` installs exactly that.
TOOLS="scripts/lighthouse"

# Minimum score per category, 0-100. Performance gets headroom because
# shared CI runners are noisy; the rest are pass/fail checks with no noise.
BUDGET_PERFORMANCE=90
BUDGET_ACCESSIBILITY=100
BUDGET_BEST_PRACTICES=100
BUDGET_SEO=100

# Lighthouse runs per page. Each category is scored by the median run, so one
# slow moment on a shared runner can't fail the budget (ADR 2026-10-16-lighthouse-median-of-three).
RUNS="${RUNS:-3}"

# Home, a post with code, a post with a large photo, a post with a video
# embed, the longest list page, and a plain page.
PAGES="/ /notes/data/ /notes/governance/ /notes/security/ /notes/ /about/"

mkdir -p "$REPORTS"
npm ci --prefix "$TOOLS" --ignore-scripts --no-audit --no-fund
LIGHTHOUSE="$TOOLS/node_modules/.bin/lighthouse"
cargo build --release --locked --quiet
target/release/bliki --out "$OUT" --serve --port "$PORT" &
server=$!
trap 'kill $server 2>/dev/null || true' EXIT INT TERM

# Wait for the server to answer.
tries=0
until curl -sf -o /dev/null "http://127.0.0.1:$PORT/"; do
  tries=$((tries + 1))
  [ "$tries" -lt 60 ] || { echo "server did not start" >&2; exit 1; }
  sleep 1
done

failed=0
for page in $PAGES; do
  name=$(printf '%s' "$page" | tr '/' '_')
  reports=""
  i=1
  while [ "$i" -le "$RUNS" ]; do
    report="$REPORTS/report$name-$i.json"
    "$LIGHTHOUSE" "http://127.0.0.1:$PORT$page" \
      --quiet \
      --chrome-flags="--headless=new --no-sandbox" \
      --output=json --output-path="$report"
    reports="$reports $report"
    i=$((i + 1))
  done
  node -e '
    const [page, runs, ...rest] = process.argv.slice(1);
    const budget = rest.slice(0, 4);
    const rs = rest.slice(4).map((f) => require(require("path").resolve(f)));
    const want = {
      performance: +budget[0],
      accessibility: +budget[1],
      "best-practices": +budget[2],
      seo: +budget[3],
    };
    let ok = true;
    const line = [];
    for (const [id, min] of Object.entries(want)) {
      // The median run for this category; its audits explain a miss.
      const sorted = rs.slice().sort((a, b) => a.categories[id].score - b.categories[id].score);
      const r = sorted[Math.floor(sorted.length / 2)];
      const score = Math.round(r.categories[id].score * 100);
      const all = rs.map((x) => Math.round(x.categories[id].score * 100));
      line.push(id + " " + score + (rs.length > 1 ? " (" + all.join("/") + ")" : ""));
      if (score < min) {
        ok = false;
        console.log("::error::" + page + ": median " + id + " " + score + " of " + runs + " runs is below " + min);
        for (const ref of r.categories[id].auditRefs) {
          const a = r.audits[ref.id];
          if (ref.weight > 0 && a.score !== null && a.score < 1) {
            console.log("  " + a.id + ": " + a.title + (a.displayValue ? " (" + a.displayValue + ")" : ""));
          }
        }
      }
    }
    console.log(page + ": " + line.join(", "));
    process.exit(ok ? 0 : 1);
  ' "$page" "$RUNS" \
    "$BUDGET_PERFORMANCE" "$BUDGET_ACCESSIBILITY" "$BUDGET_BEST_PRACTICES" "$BUDGET_SEO" \
    $reports \
    || failed=1
done
exit $failed
