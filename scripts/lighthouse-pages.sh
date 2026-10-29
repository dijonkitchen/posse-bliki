#!/bin/sh
# Pages for Lighthouse to audit, one per line (ADR 2026-10-29-lighthouse-changed-pages).
#
# With no argument, prints the fixed representative pages. With a git ref,
# also builds the site at that ref and at HEAD and adds the pages whose built
# HTML differs, so a PR's new or edited posts get audited too. Diffing the
# output, not the changed files, catches pages a template or shared note
# change touches. Past MAX_CHANGED pages the change is site-wide, and the
# fixed pages already cover every kind of page, so it prints only those.
#
# usage: scripts/lighthouse-pages.sh [BASE_REF]
set -eu

MAX_CHANGED="${MAX_CHANGED:-10}"

# Home, a post with code, a post with a large photo, a post with a video
# embed, the longest list page, and a plain page.
FIXED="/ /notes/data/ /notes/governance/ /notes/security/ /notes/ /about/"

printf '%s\n' $FIXED
[ $# -gt 0 ] || exit 0

base="$1"
# Both builds publish the same notes, whatever day each runs.
today=$(date -u +%Y-%m-%d)
tmp=$(mktemp -d)
trap 'git worktree remove --force "$tmp/src" >/dev/null 2>&1 || true; rm -rf "$tmp"' EXIT INT TERM

cargo build --release --locked --quiet
target/release/bliki --out "$tmp/head" --today "$today" >/dev/null

git worktree add --quiet --detach "$tmp/src" "$base"
(
  cd "$tmp/src"
  CARGO_TARGET_DIR="$tmp/target" cargo build --release --locked --quiet
  "$tmp/target/release/bliki" --out "$tmp/base" --today "$today" >/dev/null
)

# Built HTML pages, new or changed, as the paths they're served at.
changed=$(
  cd "$tmp/head"
  find . -name '*.html' | sort | while read -r f; do
    cmp -s "$f" "$tmp/base/$f" && continue
    p=${f#.}
    case "$p" in
      */index.html) p=${p%index.html} ;;
    esac
    case " $FIXED " in
      *" $p "*) ;;
      *) printf '%s\n' "$p" ;;
    esac
  done
)

[ -n "$changed" ] || exit 0
count=$(printf '%s\n' "$changed" | wc -l)
if [ "$count" -gt "$MAX_CHANGED" ]; then
  echo "$count pages changed, more than $MAX_CHANGED: auditing only the fixed pages" >&2
  exit 0
fi
printf '%s\n' "$changed"
