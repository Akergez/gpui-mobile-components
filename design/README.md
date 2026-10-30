# Intent design system: previews

This directory holds the proposal for the intent design system shared by aiddiction, matterfast and
money-manager, as preview cards for Claude Design.

- `src/*.html` contains one card per fragment. Its first line is
  `<!-- card group="…" title="…" app="…" -->`, and `app` sets the scheme the card opens in.
- `src/ds.css` holds the shared styles: the Pointer skin (`d-*`) and the Touch skin (`m-*`).
- `tokens/schemes.json` holds the real M3 schemes that `adaptive-colors` (rev 55e9997) makes from the
  three apps' fallback seeds: aiddiction `#0b57d0`, matterfast `#3b82f6`, money-manager `#2fb380`.
- `build.py` inlines the schemes, the styles and the icons into `dist/*.html`. Each output file starts
  with the `@dsCard` marker the Design System pane reads.

```sh
python3 -I design/build.py
```

To regenerate the schemes, build a small program against `adaptive-colors` that prints
`Scheme::light/dark(&Seed::Color(..))` for each role (`adaptive_colors::for_each_role!`).
