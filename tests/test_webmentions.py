"""Webmentions: Bridgy links on posts, received mentions rendered at build
time, and the list of webmentions to send for new posts.

See spec/indieweb.md and docs/adr/0008-webmentions-at-deploy-time.md.
"""
from __future__ import annotations

import json
from pathlib import Path
import subprocess

import mf2py
import pytest

from conftest import CONTENT_DIR, REPO_ROOT

POST = "/notes/architecture/"


def _post_entry(path: Path) -> dict:
    parsed = mf2py.parse(doc=path.read_text(encoding="utf-8"))
    return next(it for it in parsed["items"] if "h-entry" in it["type"])


def test_every_post_links_to_each_syndication_target(
    post_html_files: list[Path], config: dict
) -> None:
    targets = [t["target"] for t in config["syndicate_to"]]
    assert targets, "no syndication targets configured"
    for f in post_html_files:
        html = f.read_text(encoding="utf-8")
        for t in targets:
            assert f'<a href="{t}" hidden></a>' in html, f"{f}: no link to {t}"


def test_no_responses_section_without_webmentions(post_html_files: list[Path]) -> None:
    for f in post_html_files:
        assert 'class="webmentions"' not in f.read_text(encoding="utf-8"), f


@pytest.fixture(scope="module")
def mentioned_site(tmp_path_factory: pytest.TempPathFactory, build_site, config: dict) -> Path:
    # Targets use www. and http:// on purpose: they must still match.
    host = config["base_url"].split("://", 1)[1].rstrip("/")
    target = f"http://www.{host}{POST}"
    children = [
        {"wm-property": "like-of", "wm-target": target, "url": "https://m.example/likes/1",
         "author": {"name": "Ada", "url": "https://ada.example/"}},
        {"wm-property": "repost-of", "wm-target": target, "url": "https://m.example/boosts/1",
         "author": {"name": "Bo", "url": "https://bo.example/"}},
        {"wm-property": "in-reply-to", "wm-target": target, "url": "https://m.example/replies/1",
         "author": {"name": "Cy <script>", "url": "javascript:alert(1)"},
         "published": "2026-10-02T10:00:00Z", "content": {"text": "Nice <b>post</b>"}},
        {"wm-property": "in-reply-to", "wm-target": target, "url": "https://m.example/replies/2",
         "wm-private": True, "content": {"text": "secret"}},
        {"wm-property": "like-of", "wm-target": target, "url": "javascript:alert(2)"},
    ]
    tmp = tmp_path_factory.mktemp("wm")
    jf2 = tmp / "webmentions.json"
    jf2.write_text(json.dumps({"type": "feed", "children": children}), encoding="utf-8")
    out = tmp / "public"
    build_site(CONTENT_DIR, out, "--webmentions", str(jf2))
    return out


def test_mentions_render_on_their_post(mentioned_site: Path) -> None:
    html = (mentioned_site / POST.strip("/") / "index.html").read_text(encoding="utf-8")
    assert '<section class="webmentions">' in html
    assert "Liked by 1" in html and "Reposted by 1" in html
    assert "Nice &lt;b&gt;post&lt;/b&gt;" in html
    assert "Cy &lt;script&gt;" in html
    assert "javascript:" not in html
    assert "secret" not in html


def test_mentions_are_mf2_responses_of_the_entry(mentioned_site: Path) -> None:
    props = _post_entry(mentioned_site / POST.strip("/") / "index.html")["properties"]
    assert len(props.get("like", [])) == 1
    assert len(props.get("repost", [])) == 1
    comments = props.get("comment", [])
    assert len(comments) == 1 and "h-cite" in comments[0]["type"]
    assert comments[0]["properties"]["url"] == ["https://m.example/replies/1"]


def test_mentions_only_on_their_target(mentioned_site: Path) -> None:
    for f in (mentioned_site / "notes").rglob("index.html"):
        if f.parent.name != POST.strip("/").split("/")[-1]:
            assert 'class="webmentions"' not in f.read_text(encoding="utf-8"), f


def test_bad_webmentions_file_fails_the_build(tmp_path: Path, build_site) -> None:
    from conftest import BuildFailed

    bad = tmp_path / "wm.json"
    bad.write_text("{not json", encoding="utf-8")
    with pytest.raises(BuildFailed):
        build_site(CONTENT_DIR, tmp_path / "out", "--webmentions", str(bad))


def test_pending_lists_new_posts_per_target(site: Path, config: dict, bliki: Path, tmp_path: Path) -> None:
    feed = json.loads((site / "feed.json").read_text(encoding="utf-8"))
    new = feed["items"][0]["url"]
    previous = dict(feed, items=feed["items"][1:])
    prev = tmp_path / "previous.json"
    prev.write_text(json.dumps(previous), encoding="utf-8")
    proc = subprocess.run(
        [str(bliki), "--out", str(site), "--pending-webmentions", str(prev)],
        cwd=REPO_ROOT, capture_output=True, text=True, check=True,
    )
    expected = [f"{t['endpoint']}\t{new}\t{t['target']}" for t in config["syndicate_to"]]
    assert proc.stdout.splitlines() == expected


def test_pending_is_empty_when_nothing_is_new(site: Path, bliki: Path) -> None:
    proc = subprocess.run(
        [str(bliki), "--out", str(site), "--pending-webmentions", str(site / "feed.json")],
        cwd=REPO_ROOT, capture_output=True, text=True, check=True,
    )
    assert proc.stdout == ""


def test_pending_fails_on_a_feed_that_is_not_json(site: Path, bliki: Path, tmp_path: Path) -> None:
    prev = tmp_path / "previous.json"
    prev.write_text("<html>Not found</html>", encoding="utf-8")
    proc = subprocess.run(
        [str(bliki), "--out", str(site), "--pending-webmentions", str(prev)],
        cwd=REPO_ROOT, capture_output=True, text=True,
    )
    assert proc.returncode != 0 and proc.stdout == ""
