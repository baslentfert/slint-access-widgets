# slint-access-widgets

Accessible widgets for [Slint](https://slint.dev): built so that people who
use a screen reader or only a keyboard can use them as well as anyone else
(WCAG 2.2 AA / EN 301 549). They follow the active Slint style (fluent,
cupertino, material, cosmic), light and dark.

**Status: early work, not on crates.io yet.**

## AccessTable

A drop-in for `StandardTableView` (same `columns` and `rows`):

- **Only what is on screen exists.** 2,000 or 200,000 rows cost the same.
- **A screen reader always knows where you are**, also far down the list:
  every row says "report.pdf, Size 2 MB, Modified yesterday, 12 of 2000".
- **One Tab stop.** Arrows, Page Up/Down, Home/End; Enter opens; the Menu
  key or Shift+F10 opens your context menu.
- **Type to jump:** "inv" goes to the first row starting with "inv"; the
  same letter again walks through all rows with that letter.
- **Left/Right reads one cell:** "Size, 2 MB".
- **Sortable, resizable columns,** also by keyboard (Enter sorts, Ctrl+Left/
  Right resizes); the sort is announced.
- **Several rows at once**, as in File Explorer: Shift extends, Ctrl moves
  only the focus, Space or Ctrl+click adds or removes, Ctrl+A selects all;
  "3 selected" is announced. The selection lives in `Selection` (Rust), so
  it stays right while rows are reused.
- **Compact, standard or comfortable rows** (24 / 32 / 40 px, growing with
  the text size).
- **Empty and loading states** that are shown and announced.
- **Sticky header** and visible focus.

## AccessFilter

A quick filter field that announces "12 of 2000 shown" once you stop
typing; Escape clears it. Your app does the filtering.

For anything else in a row (check boxes, combo boxes, …) use the building
blocks `AccessTableView`, `AccessRow` and `AccessTableHeader`.

```rust
// build.rs
let config = slint_build::CompilerConfiguration::new()
    .with_library_paths(slint_access_widgets::library_paths());
slint_build::compile_with_config("ui/app.slint", config).unwrap();
```

```slint
import { AccessTable } from "@access-widgets";
```

See `examples/gallery` (`cargo run -- 200000` for a long table).

## Translations

The widgets' own words ("sorted ascending", …) use `@tr` with the context
`access-widgets`; everything else (column titles, the table's name) comes
from your app.

## Testing what a screen reader gets

`scripts/headless.sh` runs an app in its own invisible X display with its
own accessibility bus, and `scripts/atspi-probe.py` reads the tree, the
focus after each key and what would be announced. This is becoming the
`slint-a11y-check` crate.

```
scripts/headless.sh sh -c 'examples/gallery/target/debug/gallery & sleep 3; \
  python3 -I scripts/atspi-probe.py gallery "Tab Return Tab Down End b Right" 0 $!'
```

## Credits and licensing

Behaviour follows the W3C WAI-ARIA Authoring Practices; ideas are borrowed
from well-known web tables (see `docs/research-html-tables.md`). No code is
taken from them. MIT OR Apache-2.0.
