#!/usr/bin/env python3
"""Builds the Claude Design bundle: every `src/*.html` fragment becomes a
self-contained preview in `dist/`, with the colour schemes, the shared styles
and the icons inlined.

The schemes in `tokens/schemes.json` are what `adaptive-colors` makes from the
three applications' own seed colours; regenerate them with the crate, not by
hand.

    python3 -I design/build.py
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "src"
DIST = ROOT / "dist"

# The status colours a scheme has no role for: what `scheme.custom(GREEN)` and
# `scheme.custom(YELLOW)` come out near, close enough for a mock-up.
STATUS = {
    "light": {"success": "#3b6939", "on-success": "#ffffff", "success-container": "#bcf0b4",
              "warning": "#7a5900", "on-warning": "#ffffff", "warning-container": "#ffdea2"},
    "dark": {"success": "#a1d39a", "on-success": "#0a390e", "success-container": "#235024",
             "warning": "#f6be48", "on-warning": "#402d00", "warning-container": "#5c4200"},
}

ICONS = {
    "plus": '<path d="M12 5v14M5 12h14"/>',
    "back": '<path d="M19 12H5M12 19l-7-7 7-7"/>',
    "more-v": '<circle cx="12" cy="5" r="1.2"/><circle cx="12" cy="12" r="1.2"/><circle cx="12" cy="19" r="1.2"/>',
    "more-h": '<circle cx="5" cy="12" r="1.2"/><circle cx="12" cy="12" r="1.2"/><circle cx="19" cy="12" r="1.2"/>',
    "search": '<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>',
    "inbox": '<path d="M22 12h-6l-2 3h-4l-2-3H2"/><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>',
    "folder": '<path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2z"/>',
    "settings": '<path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6"/>',
    "chart": '<path d="M3 3v18h18"/><path d="M7 16v-5M12 16V8M17 16v-8"/>',
    "wallet": '<rect x="2" y="6" width="20" height="14" rx="2"/><path d="M2 10h20M16 15h2"/>',
    "list": '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
    "tag": '<path d="M12.6 2.6 21.4 11.4a2 2 0 0 1 0 2.8l-7.2 7.2a2 2 0 0 1-2.8 0L2.6 12.6A2 2 0 0 1 2 11.2V4a2 2 0 0 1 2-2h7.2a2 2 0 0 1 1.4.6z"/><circle cx="7.5" cy="7.5" r="1"/>',
    "hash": '<path d="M4 9h16M4 15h16M10 3 8 21M16 3l-2 18"/>',
    "send": '<path d="M22 2 11 13"/><path d="M22 2l-7 20-4-9-9-4 20-7z"/>',
    "clip": '<path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>',
    "check": '<path d="M20 6 9 17l-5-5"/>',
    "x": '<path d="M18 6 6 18M6 6l12 12"/>',
    "calendar": '<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>',
    "down": '<path d="M6 9l6 6 6-6"/>',
    "right": '<path d="M9 18l6-6-6-6"/>',
    "up": '<path d="M18 15l-6-6-6 6"/>',
    "reply": '<path d="M9 17l-5-5 5-5"/><path d="M20 18v-2a4 4 0 0 0-4-4H4"/>',
    "smile": '<circle cx="12" cy="12" r="9"/><path d="M8 14s1.5 2 4 2 4-2 4-2M9 9h.01M15 9h.01"/>',
    "trash": '<path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
    "archive": '<rect x="2" y="3" width="20" height="5" rx="1"/><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8M10 12h4"/>',
    "bell": '<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9"/><path d="M10.3 21a1.94 1.94 0 0 0 3.4 0"/>',
    "edit": '<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/>',
    "filter": '<path d="M22 3H2l8 9.46V19l4 2v-8.54L22 3z"/>',
    "diff": '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M12 13V7M9 10h6M9 17h6"/>',
    "bot": '<rect x="3" y="11" width="18" height="10" rx="2"/><path d="M12 7v4"/><circle cx="12" cy="5" r="2"/><path d="M8 16h.01M16 16h.01"/>',
    "copy": '<rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    "pin": '<path d="M12 17v5M9 10.76V6h6v4.76l2 3.24H7z"/><path d="M8 2h8"/>',
    "phone": '<path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.8 19.8 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6A19.8 19.8 0 0 1 2.12 4.18 2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.13.96.36 1.9.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.91.34 1.85.57 2.81.7A2 2 0 0 1 22 16.92z"/>',
    "grip": '<circle cx="9" cy="6" r="1"/><circle cx="15" cy="6" r="1"/><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="9" cy="18" r="1"/><circle cx="15" cy="18" r="1"/>',
    "sync": '<path d="M21 12a9 9 0 0 1-15 6.7L3 16M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M21 3v5h-5M3 21v-5h5"/>',
    "info": '<circle cx="12" cy="12" r="9"/><path d="M12 16v-4M12 8h.01"/>',
    "alert": '<path d="M10.3 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.7 3.86a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/>',
}

ICON_RE = re.compile(r"\{\{i:([a-z-]+)\}\}")
CARD_RE = re.compile(r"^<!--\s*card\s+group=\"([^\"]+)\"\s+title=\"([^\"]+)\"\s+app=\"([^\"]+)\"\s*-->\n")


def icon(match: re.Match) -> str:
    name = match.group(1)
    if name not in ICONS:
        sys.exit(f"unknown icon {name!r}")
    return (
        '<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" '
        f'stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{ICONS[name]}</svg>'
    )


def scheme_css(schemes: dict) -> str:
    rules = []
    for app, modes in schemes.items():
        for mode, roles in modes.items():
            props = ";".join(f"--{role}:{value}" for role, value in {**roles, **STATUS[mode]}.items())
            rules.append(f'[data-app="{app}"][data-mode="{mode}"]{{{props}}}')
    return "\n".join(rules)


TOGGLE = """
<nav class="ds-toggle" aria-label="Scheme">
  <span>Scheme</span>
  <button data-set-app="aiddiction">aiddiction</button>
  <button data-set-app="matterfast">matterfast</button>
  <button data-set-app="money-manager">money-manager</button>
  <span class="sep"></span>
  <button data-set-mode="light">Light</button>
  <button data-set-mode="dark">Dark</button>
</nav>
<script>
  const body = document.body;
  const mark = () => document.querySelectorAll('.ds-toggle button').forEach(b =>
    b.classList.toggle('on', b.dataset.setApp === body.dataset.app || b.dataset.setMode === body.dataset.mode));
  document.querySelectorAll('.ds-toggle button').forEach(b => b.addEventListener('click', () => {
    if (b.dataset.setApp) body.dataset.app = b.dataset.setApp;
    if (b.dataset.setMode) body.dataset.mode = b.dataset.setMode;
    mark();
  }));
  mark();
</script>
"""


def main() -> None:
    schemes = json.loads((ROOT / "tokens" / "schemes.json").read_text())
    style = scheme_css(schemes) + "\n" + (SRC / "ds.css").read_text()
    DIST.mkdir(exist_ok=True)
    for fragment in sorted(SRC.glob("*.html")):
        text = fragment.read_text()
        header = CARD_RE.match(text)
        if not header:
            sys.exit(f"{fragment.name}: first line must be <!-- card group=\"…\" title=\"…\" app=\"…\" -->")
        group, title, app = header.groups()
        body = ICON_RE.sub(icon, text[header.end():])
        page = (
            f'<!-- @dsCard group="{group}" -->\n'
            '<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n'
            f"<title>{title}</title>\n"
            '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
            f"<style>\n{style}\n</style>\n</head>\n"
            f'<body data-app="{app}" data-mode="light">\n{TOGGLE}\n{body}\n</body>\n</html>\n'
        )
        (DIST / fragment.name).write_text(page)
        print(f"dist/{fragment.name}  [{group}] {title}")


if __name__ == "__main__":
    main()
