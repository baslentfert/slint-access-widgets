# Changelog

## 0.3.0 (not published yet)

Tested with Slint 1.19 (needs 1.18 or later).

### New

- **AccessTree**: a tree (folders, groups) on the table's building blocks:
  only the rows on screen exist, one Tab stop, Right/Left open, close, go
  in and up, the level and "expanded" for screen readers, "Empty" for an
  empty folder. `TreeRows` (Rust) loads children the first time a node
  opens. Optional icons (built-in folder, open folder and file, or your own
  image; silent for screen readers) and indent guides (`none`, `on-hover`,
  `always`).
- **AccessTransferList**: two lists, items move between them (a dual
  listbox), with selection, "Select all", type-ahead, moves announced and
  Alt+Up/Down to put the second list in order. `TransferLists` (Rust).
- **AccessAlert** and **AccessSnackbar**: messages announced without moving
  the focus (errors and warnings at once); the kind by icon shape and a
  word; the snackbar never takes the focus and doesn't go away while
  hovered or focused (WCAG 2.2.1).
- **AccessDialog**: a modal dialog (APG dialog pattern): the focus goes in,
  stays in (Tab and Shift+Tab go round) and comes back; Escape cancels; the
  rest of the window is out of reach; buttons in the platform's order.
- **AccessBreadcrumb**: a breadcrumb trail as a navigation landmark, the
  current page said as such.
- **AccessSplitter**: a bar between two panes (APG window splitter): arrows,
  Home/End, Enter to fold away and back, mouse drag.
- `AccessColors.selection-*` and `inactive-selection-*`, worked out from the
  style's palette, and `AccessColors.contrast()` / `luminance()`.
- `AccessTableView.row-of(k)` for the row of slot `k` (use it instead of
  `first + k` in your own `for k in view.slots` loops: rows keep their slot
  while on screen).
- `AccessRow` can be expandable (`expandable`, `expanded`, `description`).
- Test tools: `scripts/a11y-test.py` checks that no empty or "unknown"
  nodes reach screen readers and can click, hover and wait;
  `scripts/scroll-cost.sh` and `scripts/scroll-profile.sh` measure
  scrolling.

### Fixed

- Selected rows in a list without the focus were almost invisible
  (1.07–1.29:1 against the list); now 3:1 or more in every style, light and
  dark, with readable text on them. The focused selection of cupertino and
  material (see-through, under 3:1) uses the accent colour instead.
- Scrolling a table, tree or transfer list did all the work for every cell
  on every animation frame (Slint animates each wheel step): rows are now
  only touched when they change.
- `AccessCheckCell`, `AccessProgressCell`, `AccessTextCell` (and the icons
  of the notices and the tree) showed up as empty "unknown" nodes for
  screen readers wherever they were used.

### Upgrading

- `AccessTree` and `AccessTreeRow` are new in this release. If you used
  them from git before it: `AccessTreeRow` got the fields `icon` and
  `image`, so Rust code that spells out the struct needs
  `..Default::default()`:
  ```rust
  AccessTreeRow { text: r.text.clone().into(), level: r.level, has_children: r.has_children, expanded: r.expanded, ..Default::default() }
  ```
- Selected rows look different without the focus (a darker grey, sometimes
  with white text); `AccessRow.text-color` follows it, so cells that use it
  stay readable. Use it in your own cells too.

## 0.2.0

AccessAutocomplete, AccessInfo, AccessSegmented, AccessMenu (column and row
menus), lazy loading (`LazyRows`), copying rows, sorting on several columns,
sideways scrolling with a frozen first column, tooltips for cut-off text,
selection by id.

## 0.1.0

AccessTable (a drop-in for `StandardTableView` that screen readers can
follow), AccessFilter.
