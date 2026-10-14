#!/bin/sh
# Lighthouse budget for the built site (spec/performance.md, ADR 0013).
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
LIGHTHOUSE="lighthouse@12.8.2"

# Minimum score per category, 0-100. Performance gets headroom because
# shared CI runners are noisy; the rest are pass/fail checks with no noise.
BUDGET_PERFORMANCE=90
BUDGET_ACCESSIBILITY=100
BUDGET_BEST_PRACTICES=100
BUDGET_SEO=100

# Home, a post with code, a post with a large photo, a post with a video
# embed, the longest list page, and a plain page.
PAGES="/ /notes/data/ /notes/governance/ /notes/security/ /notes/ /about/"

mkdir -p "$REPORTS"
cargo build --release --quiet
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
  report="$REPORTS/report$name.json"
  npx --yes "$LIGHTHOUSE" "http://127.0.0.1:$PORT$page" \
    --quiet \
    --chrome-flags="--headless=new --no-sandbox" \
    --output=json --output-path="$report"
  node -e '
    const [report, page, ...budget] = process.argv.slice(1);
    const r = require(require("path").resolve(report));
    const want = {
      performance: +budget[0],
      accessibility: +budget[1],
      "best-practices": +budget[2],
      seo: +budget[3],
    };
    let ok = true;
    const line = [];
    for (const [id, min] of Object.entries(want)) {
      const score = Math.round(r.categories[id].score * 100);
      line.push(id + " " + score);
      if (score < min) {
        ok = false;
        console.log("::error::" + page + ": " + id + " " + score + " is below " + min);
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
  ' "$report" "$page" \
    "$BUDGET_PERFORMANCE" "$BUDGET_ACCESSIBILITY" "$BUDGET_BEST_PRACTICES" "$BUDGET_SEO" \
    || failed=1
done
exit $failed
