"""Pages follow spec/design.md: one self-hosted stylesheet built on the a11y design system."""
from __future__ import annotations

import re
from pathlib import Path

import pytest

URL_RE = re.compile(r"url\(\s*['\"]?([^'\")]+)['\"]?\s*\)")
LINK_RE = re.compile(r"<link\b[^>]*>")
HREF_RE = re.compile(r'href="([^"]*)"')
BODY_START_RE = re.compile(r"<body>\s*(<[^>]+>)")
DARK_RE = re.compile(r"@media\s*\(prefers-color-scheme:\s*dark\)\s*\{(.*?)\n\}", re.S)

SEMANTIC_TOKENS = [
    "--text-1",
    "--text-2",
    "--text-link",
    "--surface-page",
    "--surface-sunken",
    "--border-strong",
    "--border-subtle",
    "--focus-halo",
    "--focus-ring",
]


@pytest.fixture(scope="module")
def css(site: Path) -> str:
    return (site / "style.css").read_text(encoding="utf-8")


def test_every_page_links_the_stylesheet(html_files: list[Path]) -> None:
    for f in html_files:
        assert '<link rel="stylesheet" href="/style.css">' in f.read_text(encoding="utf-8"), f


def test_no_inline_styles(html_files: list[Path]) -> None:
    for f in html_files:
        text = f.read_text(encoding="utf-8").lower()
        assert "<style" not in text, f"{f}: <style> block"
        assert " style=" not in text, f"{f}: inline style attribute"


def test_no_third_party_requests_from_pages(html_files: list[Path]) -> None:
    """<link> tags load nothing from another origin (rel=me, alternate and webmention aside)."""
    for f in html_files:
        for tag in LINK_RE.findall(f.read_text(encoding="utf-8")):
            if not re.search(r'rel="(stylesheet|preload|icon|modulepreload)"', tag):
                continue
            href = HREF_RE.search(tag)
            assert href and href.group(1).startswith("/") and not href.group(1).startswith("//"), (
                f"{f}: {tag}"
            )


def test_stylesheet_is_self_contained(css: str) -> None:
    assert "@import" not in css
    for url in URL_RE.findall(css):
        assert "//" not in url and not url.startswith(("/", "data:")), f"non-relative url({url})"


def test_stylesheet_urls_resolve(css: str, site: Path) -> None:
    urls = URL_RE.findall(css)
    assert urls, "expected self-hosted font urls"
    for url in urls:
        assert (site / url).is_file(), f"url({url}) does not resolve next to style.css"


def test_fonts_ship_with_licences(site: Path) -> None:
    fonts = site / "fonts"
    assert sorted(p.name for p in fonts.glob("*.woff2")), "no fonts emitted"
    for woff2 in fonts.glob("*.woff2"):
        assert woff2.read_bytes()[:4] == b"wOF2", f"{woff2.name} is not WOFF2"
    assert (fonts / "OFL-atkinson-hyperlegible-next.txt").is_file()
    assert (fonts / "OFL-jetbrains-mono.txt").is_file()


def test_type_uses_the_design_system_fonts(css: str) -> None:
    assert 'font-family: "Atkinson Hyperlegible Next"' in css
    assert 'font-family: "JetBrains Mono"' in css
    assert "--font-sans:" in css and "--font-code:" in css


def test_semantic_tokens_light_and_dark(css: str) -> None:
    dark = DARK_RE.search(css)
    assert dark, "no prefers-color-scheme: dark block"
    for token in SEMANTIC_TOKENS:
        assert f"{token}:" in css[: dark.start()], f"{token} missing in light"
        assert f"{token}:" in dark.group(1), f"{token} missing in dark"


@pytest.mark.parametrize(
    "query",
    ["prefers-color-scheme: dark", "prefers-reduced-motion: reduce", "prefers-contrast: more", "forced-colors: active"],
)
def test_user_preferences_honoured(css: str, query: str) -> None:
    assert f"@media ({query})" in css


def test_focus_is_always_visible(css: str) -> None:
    assert re.search(r":focus-visible\s*\{[^}]*box-shadow:\s*var\(--focus-outline\)", css)
    assert re.search(r"forced-colors: active\)\s*\{\s*:focus-visible\s*\{[^}]*outline:\s*3px", css)


def test_no_text_below_16px(css: str) -> None:
    for size in re.findall(r"font-size:\s*([0-9.]+)(px|rem|em)", css):
        value, unit = float(size[0]), size[1]
        px = value if unit == "px" else value * 16
        assert px >= 16, f"font-size {size[0]}{unit} is below 16px"


def test_skip_link_first_in_body(html_files: list[Path]) -> None:
    for f in html_files:
        text = f.read_text(encoding="utf-8")
        first = BODY_START_RE.search(text)
        if first is None:
            continue  # alias redirect stubs carry no body content
        assert first.group(1) == '<a class="skip-link" href="#main">', f"{f}: {first.group(1)}"
        assert '<main id="main">' in text, f"{f}: no <main id=\"main\">"
