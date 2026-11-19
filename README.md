# slint-access-widgets

Accessible widgets for [Slint](https://slint.dev): built so that people who
use a screen reader or only a keyboard can use them as well as anyone else
(WCAG 2.2 AA / EN 301 549). They follow the active Slint style (fluent,
cupertino, material, cosmic), light and dark.

```
cargo add slint-access-widgets
cargo add --build slint-access-widgets
```

**Status: young (0.2).** Tested automatically with the Linux accessibility
bus (AT-SPI, what Orca reads); Windows (NVDA, Narrator) and macOS
(VoiceOver) still to be tested.

![The gallery example](docs/gallery.png)

## AccessTable

A drop-in for `StandardTableView` (same `columns` and `rows`):

- **Only what is on screen exists.** 2,000 or 200,000 rows cost the same.
- **Lazy loading:** the table says which rows are on screen; fetch only
  those (from a server, another computer). Rows on their way show and read
  "Loading…". `LazyRows` (Rust) keeps track.
- **A screen reader always knows where you are**, also far down the list:
  every row says "report.pdf, Size 2 MB, Modified yesterday" (up to 24
  columns) with its position.
- **One Tab stop.** Arrows, Page Up/Down, Home/End; Enter opens; the Menu
  key or Shift+F10 opens your context menu.
- **Type to jump:** "inv" goes to the first row starting with "inv"; the
  same letter again walks through all rows with that letter.
- **Left/Right reads one cell:** "Size, 2 MB".
- **Sorting** by click or Enter on a header; **Shift+click or Shift+Enter
  adds a level** (up to three: ▲1 ▼2). Announced: "Type, sorted ascending,
  then Size, sorted descending". Your app sorts (`sort-changed`).
- **Columns:** drag the edge or Ctrl+Left/Right to resize; double click
  the edge or Ctrl and + to fit the contents. When they don't fit, the
  table **scrolls sideways** and the **first column stays in place**.
- **A column menu** (right click, Menu key or Shift+F10 on a header): sort,
  add to sort, size to fit, hide, move left/right, and every column with a
  tick to show or hide it. Each action also has a key on the header
  (Delete hides, Ctrl+Shift+Left/Right moves), and every change is
  announced ("Size, hidden", "Name, column 2 of 6"). Your app moves the
  data (`move-column`).
- **A row menu** (`row-menu`: your items; right click, Menu key or
  Shift+F10 on a row), followed item by item by screen readers; the focus
  comes back to the row.
- **Several rows at once**, as in File Explorer: Shift extends, Ctrl moves
  only the focus, Space or Ctrl+click adds or removes, Ctrl+A selects all;
  "3 selected" is announced. `Selection` (Rust) keeps the **same items
  selected after sorting or filtering**.
- **Ctrl+C copies** the selected rows as tab-separated text (pastes as
  columns into a spreadsheet); "3 rows copied" is announced, and "Could not
  copy" when the clipboard refuses (feature `clipboard`).
- **Cut-off text gets a tooltip** with all of it, for the mouse and for the
  keyboard; it stays while you hover it, Escape closes it (WCAG 1.4.13).
- **Focus ring only when you use the keyboard** (like `:focus-visible`);
  the selection always shows, also when the focus is elsewhere: then in a
  grey with at least 3:1 contrast against the background, and readable text
  on it (WCAG 1.4.11, 1.4.3), worked out from the style's own colours
  (`AccessColors`), so in light, dark and high-contrast palettes alike.
- **Compact, standard or comfortable rows** (24 / 32 / 40 px, growing with
  the text size); **empty and loading states** that are shown and announced;
  **sticky header**.

For anything else in a row (check boxes, progress bars, …) use the building
blocks `AccessTableView`, `AccessRow` and `AccessTableHeader`, with cells
that never take focus (`AccessCheckCell`, `AccessProgressCell`,
`AccessTextCell`); see `examples/transfer`.

## AccessAutocomplete

A text field with suggestions underneath (the editable combobox of the
WAI-ARIA Authoring Practices). Your app gives the suggestions for what was
typed (`edited` → `suggestions`), from a list, a database or another
computer; `loading` shows "Searching…".

- After a short pause "3 suggestions" is announced (not on every key).
- Down (or Alt+Down without typing) goes into the list; every suggestion has
  focus of its own and reads "test-vm-02, 2 of 3"; Up from the first goes
  back to the field.
- Enter or a click chooses; Escape closes the list, a second Escape empties
  the field; typing while in the list goes on in the field.
- `suggestions-only` allows only a suggestion to be chosen.

## AccessInfo

An ⓘ button with an explanation (a "toggletip"): opens on click, Enter or
Space (not only on hover), so it works with a keyboard, a touch screen and a
screen reader. The explanation is announced when it opens and stays the
button's description while open; the button reads as "More about …",
expanded or collapsed. WCAG 1.4.13: Escape or a click elsewhere closes it,
it stays while hovered, and until closed. (On Linux "expanded" doesn't
reach screen readers yet: AccessKit #824.)

Give the widgets the window size, so the explanation opens under the ⓘ (or
above it near the bottom), 8 px from the window edges, with a pointer to the
ⓘ; without it Slint only keeps it inside the window:

```slint
import { AccessWindow } from "@access-widgets";
// in your window:
init => { AccessWindow.width = self.width; AccessWindow.height = self.height; }
changed width => { AccessWindow.width = self.width; }
changed height => { AccessWindow.height = self.height; }
```

## AccessSegmented

One choice out of a few, as joined buttons (a radio group): one Tab stop on
the chosen option, the arrows move and choose (round), Home/End, options
that are not available are skipped (`unavailable`). Reads as "Sound,
group · Off, radio button, checked, 3 of 3". The choice shows with a tick
and bold text, not colour alone (WCAG 1.4.1).

## AccessMenu

A menu that screen readers can follow (Slint 1.18's own context menu shows
its items to assistive technology as plain labels, without focus): every
item has focus of its own and reads with its state ("Owner, checked",
"Add to sort, unavailable"). Up/Down/Home/End, a letter to jump, Enter or
Space to choose, Escape or Tab to close.

## AccessTree

A tree (folders, groups), built on the table's building blocks, so only the
rows on screen exist. One Tab stop; Up/Down/Home/End; Right opens a node or
goes to its first child, Left closes it or goes to its parent; Enter
activates; letters jump. Rows read as "Projects, expanded, level 2"; on
Linux, where "expanded" doesn't reach screen readers yet (AccessKit #824),
it is said in the description and announced ("expanded, 3 items"). A
folder that turns out empty says "Empty". `TreeRows` (Rust) keeps the
visible rows and loads children the first time a node opens, so they can
come from a disk or another computer.

Rows can have an icon before the text: a built-in folder (open while
expanded) or file, drawn by the tree (`icon: AccessTreeIcon.folder`), or
your own `image` (16 × 16 px). `TreeRowInfo::branch` says a node was a
folder, also when it turned out empty. Icons are decoration: screen readers
don't hear them (the text and "expanded"/"collapsed" already say what a row
is). `indent-guides` draws a thin line per level, from a folder down along
its contents: `none` (the default, as in File Explorer, Finder and GTK),
`on-hover` (while the mouse is on the tree or it has the focus, as the VS
Code explorer does by default) or `always`. They use the text colour, faint
(`AccessColors.guide`), so they follow light, dark and high-contrast
palettes. See `docs/research-tree-icons.md`.

## AccessTransferList

Two lists, items move between them ("Available" and "Chosen"; a dual
listbox). Each list is one Tab stop with Shift/Ctrl selection, Space,
Ctrl+A, a "Select all" check box and type-ahead. Enter or a double click
moves the selected items to the other list, Delete moves them back; with
nothing selected, the current item moves. The focus stays where it was, on
the item after the moved ones, so Enter, Enter, Enter moves them one by one;
when a list empties, the focus goes along to the other one. Every move is
announced ("Moved to Chosen: 2 items"), and the moved items are selected in
the other list, where Tab lands on them. The buttons say where items go
("Add to Chosen"), not only with arrows. With `orderable`, Alt+Up/Alt+Down
put the second list in order ("position 4 of 5"). `TransferLists` (Rust)
keeps both lists and their selections.

## AccessAlert and AccessSnackbar

Messages for the user (WCAG 4.1.3 status messages). Both are announced
without moving the focus: errors and warnings at once (an assertive live
region, like the WAI-ARIA APG
[alert pattern](https://www.w3.org/WAI/ARIA/apg/patterns/alert/)), the rest
when the screen reader is done with what it is saying (a polite one, like
the [`status` role](https://www.w3.org/TR/wai-aria-1.2/#status)). The kind
shows by an icon with its own shape and is said as a word ("Error: …"), not
only by colour (WCAG 1.4.1).

- **`AccessAlert`** stays in the window until closed ("Settings saved",
  "3 files could not be sent"). Set `shown: true`; it is announced then and
  whenever its text changes: "Error: 3 files could not be sent. The other
  computer has no room left. Try again is available". It reads as a group
  with that name, with an optional button (`action-text`) and a close
  button ("Close message: 3 files could not be sent") that Tab reaches in
  the normal order.
- **`AccessSnackbar`** is a short message at the edge of the window:
  `snack.show("File deleted", "Undo")`. It never takes the focus, but Tab
  reaches its buttons and Escape inside closes it. Without a button it goes
  away by itself (`duration`, 6 s), **but not while the mouse is on it or
  the focus is in it** (WCAG 2.2.1); with a button it stays until closed, so
  there is always time to reach it. When it closes with the focus inside,
  `had-focus` is true: put the focus back where the user was (`closed`).

## AccessFilter

A quick filter field that announces "12 of 2000 shown" once you stop
typing; Escape clears it. Your app (or server) does the filtering.

## Use it

```rust
// build.rs
let config = slint_build::CompilerConfiguration::new()
    .with_library_paths(slint_access_widgets::library_paths());
slint_build::compile_with_config("ui/app.slint", config).unwrap();
```

```slint
import { AccessTable } from "@access-widgets";
```

Examples (`cargo run` in each):

| | |
|---|---|
| `examples/basic` | the smallest table: swap it in for `StandardTableView` |
| `examples/gallery` | everything: sort, filter, select, copy, density (`-- 200000` for a long table, `--lang nl` in Dutch) |
| `examples/transfer` | a list with tick boxes and progress per row, pause and cancel |
| `examples/remote` | lazy loading: 100,000 files on a slow server, searched and sorted there |
| `examples/autocomplete` | typing with suggestions that arrive from a "server" |
| `examples/controls` | smaller controls: the ⓘ button, segmented choices |
| `examples/tree` | a folder tree whose folders load when opened |
| `examples/transfer-list` | choosing columns: from "Available" to "Shown", in order |
| `examples/notices` | alerts that stay, and a snackbar with "Undo" |

## Translations

The widgets' own words ("sorted ascending", "3 selected", …) use `@tr`
with the context `access-widgets`, and Dutch and German come with the
crate (`translations/`). With [po-bundle](https://github.com/baslentfert/po-bundle)
they are picked up and filled in by themselves; everything else (column
titles, the table's name) comes from your app.

## Testing what a screen reader gets

`scripts/a11y-test.py` starts each example in its own invisible X display
with its own accessibility bus (`scripts/headless.sh`), presses keys and
checks after every key where the focus is and what is announced, also
what lands on the clipboard; and that no empty or "unknown" nodes reach a
screen reader (decoration that should be silent):

```
scripts/headless.sh python3 -I scripts/a11y-test.py
```

`scripts/shot.sh` makes screenshots the same way; `scripts/scroll-cost.sh`
measures what scrolling costs.

A tip for your own components: `accessible-role: none` on a component's
root does *not* keep it away from screen readers in Slint 1.19; every place
the component is used becomes an empty "unknown" node. Leave the role out
on a root that is a `Rectangle`, `Path` or layout (they have no role of
their own), and set `accessible-role: none` where a `Text` or `Image` is
used. This is becoming the
`slint-a11y-check` crate.

## Not yet

Table semantics (rows, cells, column headers) for screen readers need Slint
and AccessKit ([slint#14965](https://github.com/slint-ui/slint/issues/14965));
until then rows are read as list items with their column titles. Still to
come: tree rows inside the table, groups, a card layout for
narrow windows (also the transfer list's two lists under each other), editable cells, rows of different heights, drag and drop,
right-to-left.

## Credits and licensing

Behaviour follows the W3C WAI-ARIA Authoring Practices; ideas are borrowed
from well-known web tables (see `docs/research-html-tables.md`). No code is
taken from them. MIT OR Apache-2.0.
