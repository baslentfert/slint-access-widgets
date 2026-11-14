# Research: accessible tables and data grids on the web

Goal: borrow ideas and behaviour (not code) from well-regarded web tables and
data grids for an accessible, virtualized Slint table (MIT/Apache, WCAG 2.2 AA /
EN 301 549). Sources are numbered inline as [n]; the list is at the end.
Anything not confirmed in a source is marked **[unverified]**.

Licence note: MUI X Pro/Premium, AG Grid Enterprise and Handsontable carry
commercial or non-commercial licences. Only their documented *behaviour* is
described here; no code was read or copied, except for default numbers read from
MIT-licensed sources (Fluent UI, MUI X core, Material Components Web).

---

## 0. First decision: table or grid?

- **Table** (`role=table`): for reading and comparing. It can still be sorted and
  filtered and can contain links or buttons. Screen readers use their own table
  navigation (NVDA: Ctrl+Alt+arrows) [9].
- **Grid** (`role=grid`): a composite widget with **one tab stop**; the author
  handles arrow keys, Home/End and PageUp/Down [1]. Use it when the main purpose
  is *interacting* (selecting, editing, acting on rows) and the context is
  app-like. Grids "have a steeper learning curve than tables and more support
  quirks across assistive tech" [8].
- Higley rejects a fixed threshold (such as a percentage of interactive cells)
  for choosing between them [8].
- For a desktop toolkit, the app context and virtualization (rows that do not
  exist cannot be reached by screen-reader table navigation) both point to
  **grid semantics with author-managed focus** as the default. A non-interactive,
  fully rendered "simple table" mode can come later.

---

## 1. Layout and visual

| Feature | What it is | Accessibility notes / pitfalls |
|---|---|---|
| Sticky header | Column headers stay visible while the body scrolls | Header needs an opaque background or text overlaps [6]. **WCAG 2.4.11**: a focused row must not be *entirely* hidden behind the sticky header. Scroll the focused row into the area *below* the header (the CSS equivalent is `scroll-padding`, technique C43) [28]. |
| Frozen first column | Row-header column stays visible while scrolling sideways | Expose it as `rowheader` so each cell is announced with its row name [1][14]. MUI keeps row-header cells mounted even when column virtualization scrolls them away [14]. Frozen areas become unusable at high zoom if they take most of the viewport [6]. Cap the frozen width (e.g. at most about 1/3 of the viewport [unverified heuristic]) or unfreeze it when narrow. |
| Density | Compact / standard / comfortable row heights | MUI: standard, compact and comfortable, user-switchable from a toolbar [14]. Primer: condensed, normal and spacious [23]. Spectrum: compact, regular and spacious [unverified, from its design-data listing]. Numbers are in section 3. |
| Cell padding | Horizontal padding inside cells | Material: 16px leading and trailing padding [21]. Carbon: 16px [17]. Fluent v9: `spacingHorizontalS` (8px) [19]. Fluent v8 DetailsList: 12px left and 8px right [20]. |
| Zebra striping | Alternate row background | Carbon offers it as a token (`$layer-accent`) [17]. It disappears in Windows High Contrast / forced colours, so never use it to carry meaning [7]. Atlassian: "Never rely on highlighted rows to convey important meaning, selection, or focus" [24]. |
| Row hover | Highlight under the pointer | Decoration only; selection and focus need their own non-colour indicator (a checkbox, an outline) with 3:1 non-text contrast (WCAG 1.4.11) [unverified for tables specifically]. |
| Gridlines | Row separators and optional column lines | Fluent v9 rows use a thin bottom border (`colorNeutralStroke2`) [19]. |
| Column alignment | Text follows reading direction; numbers align right | GOV.UK: "align the numbers to the right" when comparing figures, with numeric headers aligned the same way [22]. Primer: left by default, numeric `align: 'end'` [23]. Use *start/end* rather than left/right so RTL works. |
| Vertical alignment | Content centred in the row | Carbon: header and row text always centred, except in its XL rows [17]. |
| Truncation | Ellipsis plus tooltip, or wrapping | The full text must reach assistive technology (the accessible name of a cell is its full value, not the clipped one). A tooltip must also open on keyboard focus, be dismissable with Esc and stay hoverable (WCAG 1.4.13) [unverified as grid-specific guidance]. Allow an optional "wrap" mode. |
| Empty state | Message when there are no rows | React Aria `renderEmptyState` ("No results found") [10]. It must sit inside the grid (a row or cell) or be announced as a status message [30]. |
| Loading state | Spinner, skeleton or "load more" row | React Aria `TableLoadMoreItem` with an `aria-label="Loading more..."` spinner [10]. Announce "Loading…" and then "N rows loaded" politely (WCAG 4.1.3 covers "waiting state") [30]. Skeleton rows should be hidden from assistive technology. |
| Caption and name | Visible title for the table | The grid needs an accessible name via `aria-labelledby` (visible label) or `aria-label`, and optionally `aria-describedby` [1]. Atlassian: use a caption *or* a label, not both [24]. Carbon requires a label or title [18]. |

## 2. Columns

| Feature | Behaviour in good grids | Accessibility |
|---|---|---|
| Resize | Drag the header edge, with min/max/default width [10] | AG Grid: **Alt+Left/Right** on a focused header resizes [13]. React Spectrum: a keyboard resize mode, left with **Esc or Enter**, which returns focus to the header [33]. React Aria's `ColumnResizer` is focusable and accepts its own `aria-label` [10][33]. Expose the handle as a separator/slider with the current width as its value [unverified best practice]. |
| Auto-fit | Double-click the header edge to fit the content | Excel and most grids do this [unverified per library]. Offer a keyboard path too (a header menu item "Fit to content"). |
| Reorder | Drag a header | AG Grid: **Shift+Left/Right** on a focused header moves the column [13]. Announce the new position ("Moved to column 3 of 6") [unverified]. |
| Hide/show | Column chooser | Keep `aria-colcount` as the *total* and `aria-colindex` as the real position, so users notice that columns are missing [2]. AG Grid counts hidden columns in colcount [12]. |
| Pin | Freeze left or right | Same as the frozen column. Keep DOM/accessibility order equal to the visual order; AG Grid offers `ensureDomOrder` for this [12]. |
| Header menu | Sort, filter, hide, pin | AG Grid: **Alt+Down** or Ctrl+Enter on the header [13]. MUI and Handsontable: **Ctrl+Enter** on the header [14][25]. |
| Persistence | Remember widths, order, visibility, sort and density | An app concern; expose a serializable "column state" [unverified as a documented feature in the surveyed open-source grids]. |

## 3. Concrete default numbers

| Source | Row heights | Header | Padding |
|---|---|---|---|
| Carbon [17] | xs 24, sm 32, **md 40** (base), lg 48, xl 64 | "should always match the row size" | 16 (`$spacing-05`) |
| Fluent v9 Table [19] | extra-small 24, small 34, **medium 44** | (not checked) | 0 8px (`spacingHorizontalS`) |
| Fluent v8 DetailsList [20] | **42**, compact 32 | (not checked) | left 12, right 8 (extra 24) |
| Material 2 / MDC [21] | **52** default, density down to a minimum of **36** (scales −4…0, 4px steps) | row + 4 = **56** | 16 leading, 16 trailing |
| MUI X [15][16] | **52** default; compact ×0.7 (≈36), comfortable ×1.3 (≈68) | (factor also applies to the header) | (not checked) |
| Carbon typography [17] | row text 14px Regular | column header 14px SemiBold | |
| WCAG 2.5.8 [31] | targets at least **24×24 CSS px** (or 24px spacing) | | |

**Proposal for Slint** (logical px, all scale with the font size):
- Compact **32**, Standard **40**, Comfortable **48**, with the header the same
  height as the rows (Carbon). Do not offer a 24px row as a default: checkboxes and
  buttons in it barely meet 2.5.8 [inference].
- Horizontal cell padding 12–16 (Carbon and Material 16, Fluent 8–12); vertical
  padding is implied by the row height.
- Rows must grow with the text size. A fixed pixel height breaks at 200% text
  size (WCAG 1.4.4) [inference]. Virtualization then needs
  `row_height = f(font metrics, density)`, not a constant.
- The selection checkbox gets a 24×24 minimum hit area, even in compact rows.

## 4. Sorting

- Put a **button inside the column header** (native keyboard support) [3][5].
  In a grid the header cell itself can be focusable and Enter sorts it
  (AG Grid, MUI, Handsontable) [13][14][25]. Carbon: Tab to the header, then
  Space/Enter [18].
- Put `aria-sort` (AccessKit: `sort_direction` [32]) **on one header only**:
  "You should not use `aria-sort` on more than one column header at a time" [5].
  On unsorted columns **remove** the attribute rather than setting `none` [5]
  (the APG examples do set `none` [2]; Roselli's advice is the safer choice).
- **Multi-sort**: Shift+Enter on the header (AG Grid, MUI, Handsontable) or
  Shift/Ctrl+click [13][14][25]. Only the primary column gets aria-sort. Announce
  the secondary sorts through a live region or the header description
  [inference from [5]].
- **Tri-state** (asc, desc, none): common, but Primer says that if a table is
  sortable, one column should always be sorted by default [23]. Tip: let
  "none" return to the original data order.
- **Icons**: ▲/▼, chosen because they survive high-contrast inversion. Unsorted
  sortable columns get a distinct shape (♢), not just another colour or size,
  and the icons are `aria-hidden` [3][5].
- **Announcement**: support differs between screen readers. JAWS, Narrator and
  iOS VoiceOver announce sort changes; NVDA, TalkBack and macOS VoiceOver do not.
  Hence a polite live region ("Sorted by Name, ascending") that clears itself
  after about a second, with the text taken from the translations [5].
- **Pitfall**: some screen readers do not notice changes to the *focused* header,
  so the new state is only heard after moving away and back [12]. That is
  another reason for the live region.
- Put the hint "column headers with buttons are sortable" in the caption or
  description, *once*, not on every header [3][5].

## 5. Filtering and search

- Quick filter (one search box above the grid) and per-column filters (in the
  header menu or a filter row below the header).
- AG Grid floating filters: Left/Right between header cells, Enter goes into the
  filter, Esc returns to the header [13].
- **Announce the result count** as a status message ("12 of 340 rows"), the
  example WCAG 4.1.3 gives for search results [30].
- Update `row_count` after filtering; a stale `aria-rowcount` is a known
  problem [34].
- The search box and toolbar **must reflow** at 320px / 400% zoom; only the
  table itself is exempt [29].
- Ctrl+Shift+L toggles filters in Excel (web/Mac) [27]. That is a familiar
  shortcut, but optional.

## 6. Selection

- **Modes**: none, single, multiple. React Aria distinguishes **toggle**
  (Space/Enter toggles, arrows only move focus, usually with a checkbox column)
  from **replace** (arrows move focus *and* selection, modifiers add to it;
  explorer-like) [11]. Touch is always toggle, and a long press enters selection
  mode [11].
- **Keys** [1][11][14][25]:
  - Space toggles the row (AG Grid) [13], Shift+Space selects the row [1][14].
  - Shift+Up/Down extends the range [14], Shift+click selects a range [14].
  - Ctrl+Space toggles without losing the rest (React Aria) [11]. Note: in APG and
    Excel, Ctrl+Space selects the *column* [1][27].
  - Ctrl+Arrow moves focus without selecting (React Aria) [11].
  - Ctrl+A selects all [1][14][25]; Esc clears the selection (React Aria
    default) [10].
- **Checkbox column**: a header checkbox for select-all (tri-state: none, some,
  all) [10]. Each row checkbox is named after the row ("Select Invoice 123").
  Atlassian asks for unique row-specific labels for row actions too [24].
- **ARIA**: `aria-selected` on rows or cells, and `aria-multiselectable` on the grid
  (AccessKit: `selected` [32]). Selected rows need a non-colour indicator [24].
- **Announcement**: "3 selected" through a live region after changes (most
  libraries leave this to the app) [unverified per library]. Recommendation:
  always do it, politely, after a short debounce.
- Disabled rows: `isDisabled` / `disabledKeys` [10].

## 7. Keyboard navigation and focus

Base (APG grid) [1][2]:

| Key | Action |
|---|---|
| Arrows | One cell. At the edge nothing happens (no wrap) |
| Home / End | First / last cell of the row |
| Ctrl+Home / Ctrl+End | First cell of the first row / last cell of the last row |
| PageUp / PageDown | An author-defined number of rows (APG example: 5); better is "one visible page" (MUI, Handsontable) [14][25] |
| Enter / F2 | Into the cell: edit field or first widget. F2 again leaves [1] |
| Esc | Back to grid navigation, optionally undoing the edit [1] |
| Tab / Shift+Tab | Inside a cell: between its widgets. Otherwise out of the grid (one tab stop) [1] |
| Typing a character | Starts editing in an editable cell [1]; otherwise type-ahead on the first column (React Aria `textValue`) [10] |

Additions from spreadsheets (Excel/Handsontable) [25][27]: Ctrl+Arrow jumps to
the edge of the data, Ctrl+Shift+Arrow extends to the edge, Alt+PageUp/Down
pages sideways, and Shift+F10 / the Menu key opens the context menu.

Focus rules:
- **One tab stop** with roving focus. The APG examples use roving tabindex;
  the alternative is `aria-activedescendant` [1][2]. In AccessKit this is the
  `focus` on the active cell's node.
- **Row versus cell focus**: React Aria and the APG treegrid support both; Left
  from the first cell can go to the row [4][10]. For a "list-like" table with no
  editing, **row focus** (the row reads all its cells) is easier for screen-reader
  users; for editable grids use **cell focus**. Make it configurable.
- **Headers**: focusable only when they *do* something (sort, filter, menu) [1][2].
  AG Grid starts tabbing at the first header [13]; Handsontable can turn header
  navigation off (`navigableHeaders`) [26]. Up from the first row goes to the
  header [13].
- A cell with a single widget (link, checkbox) passes focus to that widget; a text
  cell takes focus itself [1].
- **Focus after deleting a row**: move to the next row [24].
- **Restore focus** after sorting or filtering to the same *record* (by key, not by
  index). If it is gone, use the nearest row [inference].
- **Visible focus**: an outline of at least 2px with 3:1 contrast that is not
  covered by sticky headers or frozen columns (2.4.11) [28]. Frozen columns cover
  horizontally scrolling cells, so scroll the focused cell into view next to the
  frozen area.

## 8. Editing

- Enter or F2 opens, Esc cancels, Enter commits and moves down, Tab commits and
  moves right (Excel, Handsontable) [25][27]. Typing a character starts editing
  with that character [1][25].
- Ctrl+Enter fills a selected range with the value (AG Grid, Handsontable)
  [13][25]. Alt+Enter inserts a line break in the cell [25].
- Ctrl+Z / Ctrl+Y for undo and redo [25].
- **Validation**: keep the editor open, set `aria-invalid` and link the message
  with `aria-describedby`/`aria-errormessage`, and announce the error
  (WCAG 3.3.1 / 4.1.3) [unverified for specific grids]. Read-only cells or grids get
  `aria-readonly` [1].

## 9. Grouping and tree rows (treegrid)

- `role=treegrid`, with `aria-expanded` **only** on rows that actually have
  children [4]. AG Grid switches to treegrid as soon as there are groups or tree
  data and uses `aria-level` [12].
- **Right** on a collapsed row expands it, then goes to the first cell. **Left**
  on an expanded row collapses it, then goes to the parent [4]. MUI: Space on a
  group cell toggles [14].
- Level, set size and position: AccessKit has `level` (zero-based!),
  `size_of_set` and `position_in_set` [32].
- Pitfall: sticky group rows produce out-of-order row indexes (AG Grid offers
  `suppressGroupRowsSticky` for screen-reader users) [12].

## 10. Virtualization (only visible rows exist)

What the web world does:
- `aria-rowcount`/`aria-colcount` = **total**, and `aria-rowindex`/`aria-colindex` =
  real position, 1-based, header row included [1][2]. Without them a screen reader
  announces the visible window as the whole table [9].
- Higley: these four attributes "alone don't make on-demand grids accessible" [9].
- AG Grid recommends that screen-reader users turn virtualization off or use
  pagination, keep DOM order equal to visual order, and render all columns [12].
- Handsontable likewise recommends turning off DOM virtualization, and sets
  `aria-rowcount=-1` on purpose because screen readers handle it badly [26].
- With server-side loading the total is unknown, so it is not announced [12].

Translated to Slint/AccessKit:
- AccessKit has `row_count`, `column_count`, `row_index`, `column_index`,
  `row_index_text`, `column_index_text`, `row_span`, `column_span` and
  `sort_direction` [32]. These are the direct equivalents.
- Since the user's own focus is the only way to "travel", keyboard navigation
  must be done by **the component**, not by the screen reader. The component
  moves focus to a row that is created just in time, and the screen reader hears
  "row 1234 of 5000". Screen-reader table commands (which walk only the existing
  nodes) only work within the visible window. That is a known and accepted
  limitation of virtualization [9][12].
- **Overscan**: keep a few rows above and below the window in the tree, so that
  arrow keys never focus a node that does not exist yet [inference].
- Native platform APIs: UIA has `ItemContainerPattern`/`VirtualizedItemPattern`
  for virtualized lists. Whether AccessKit supports them is **[unverified]**. Check
  this before relying on it, because it determines whether NVDA/Narrator can
  "realize" off-screen rows.
- Offer a fallback for small datasets (for example under 500 rows):
  **render everything**, so screen-reader table navigation works completely
  (the AG Grid and Handsontable advice) [12][26].

## 11. Responsiveness, zoom and reflow

- Data tables are **exempt** from 1.4.10 Reflow (a two-dimensional layout), but
  the toolbar, title and search box above them are not [29].
- Web approaches: a horizontally scrolling region that can take focus and has a
  name (`role=region` with a label, from the caption) [6][7], or a stacked "card"
  layout at narrow widths with the column name before each value [7].
- Pitfall: changing the display or layout of table elements loses the table
  semantics, so the roles must be set again explicitly [7][9].
- Slint: offer (a) horizontal scrolling with a frozen first column that unfreezes
  when narrow, and (b) an optional card/list mode where each row becomes a group
  of "Column: value" lines, with the same row object and the same selection.
- GOV.UK: on small screens, smaller text only for large datasets, and split
  datasets where possible [22].

## 12. Context menu, clipboard, export, printing

- Context menu: **Shift+F10** and the Menu key (Excel, Handsontable), and also
  Ctrl+Shift+\ in Handsontable. Arrows inside the menu, Esc closes it and returns
  focus to the cell [25][27].
- Copy: Ctrl+C copies the selected rows (MUI) or range (Handsontable) [14][25].
  Use tab-separated text with a header row, so it pastes into Excel/Sheets
  [unverified as a documented format]. Announce "3 rows copied".
- Export (CSV) and print: an app concern; the component can offer "all rows as
  text". For printing, do not use the stacked layout (it wastes paper) [7].

## 13. i18n and RTL

- Every built-in string (sort and selection announcements, "Loading…", resizer
  label, empty state) comes from translations. AG Grid has localisation variables
  for its ARIA labels [12]. Roselli: take announcement texts from the
  translations [5].
- RTL: column order mirrors, frozen columns go to the right, Left/Right arrows
  follow the *visual* direction, and number alignment becomes "end" [inference].
- Number and date formatting per locale. Sorting with a locale-aware collator,
  not byte order [unverified per library].

---

## 14. Known pitfalls in brief

1. aria-sort on more than one column [5].
2. Sort icons as the only cue, distinguished by colour [3][5].
3. Sticky header or frozen column covering the focused row (2.4.11) [28].
4. Content outside cells (a checkbox directly in a row) cannot be reached [9].
5. Accessibility order differs from visual order (pinning, DOM reuse) [9][12].
6. Stale row count after filtering [34]; unknown total in server-side mode [12].
7. Focus on a header that changes: its new state is not announced [12].
8. Grids for plain reading content: a learning curve for screen-reader users [8].
9. Zebra striping, hover or selection by colour alone: gone in high contrast
   [7][24].
10. Fixed row heights that do not scale with text size [inference].

## 15. Prioritized recommendation

**v0.1: must-have (basic, accessible, virtualized)**
- Grid semantics: AccessKit Grid/Row/Cell/ColumnHeader/RowHeader, with total
  `row_count`/`column_count` and 1-based `row_index`/`column_index` per row/cell.
- One tab stop. Arrows, Home/End, Ctrl+Home/End and PageUp/Down (one page), with
  overscan so focus always lands on an existing node.
- Row focus (default) and cell focus (option); a visible focus indicator that is
  never covered by the sticky header.
- Sticky header; columns with a fixed, min and max width; number alignment
  "end".
- Single and multiple selection (toggle behaviour): Space, Shift+Space,
  Shift+Arrow, Ctrl+A, Esc; a checkbox column with tri-state select-all;
  `selected` on the row; the "N selected" announcement.
- Single-column sort through a focusable header (Enter/Space), sort_direction on
  one column, ▲/▼ plus a distinct "sortable" shape, and a live-region
  announcement.
- Densities 32/40/48 that scale with the font; a 24px minimum target for
  checkboxes.
- Empty state and loading state inside the grid, with a status message.
- Accessible name (caption) required; all strings translatable.

**v0.2**
- Column resizing with mouse and keyboard (Alt+Left/Right, or a resize mode with
  Esc/Enter) and double-click auto-fit.
- Quick filter with a result-count announcement; filter API per column.
- Replace selection behaviour (explorer-like) and Ctrl+Arrow focus without
  selecting.
- Frozen first column (row header) that unfreezes when narrow.
- Type-ahead on the first column; Shift+F10 / Menu key context menu; Ctrl+C as
  TSV.
- Hide/show columns with a correct column_count/column_index; persistence of
  column state (serializable).
- Restoring focus to the same record after sorting and filtering.

**Later**
- Inline editing (Enter/F2/Esc, Tab/Enter commit, validation with
  invalid/describedby), undo.
- Treegrid/grouping (Right/Left expand, `level`, `size_of_set`,
  `position_in_set`).
- Multi-sort (Shift+Enter), column reordering (Shift+Left/Right), pinning on the
  right.
- Card/stacked mode for narrow widths; RTL polish.
- "Render everything" mode for small datasets; research into UIA
  ItemContainerPattern/VirtualizedItemPattern through AccessKit.
- Cell range selection (spreadsheet mode), Ctrl+Enter fill, export and print.

---

## Sources

1. W3C WAI-ARIA APG, Grid pattern. https://www.w3.org/WAI/ARIA/apg/patterns/grid/
2. W3C WAI-ARIA APG, Data grid examples. https://www.w3.org/WAI/ARIA/apg/patterns/grid/examples/data-grids/
3. W3C WAI-ARIA APG, Sortable table example. https://www.w3.org/WAI/ARIA/apg/patterns/table/examples/sortable-table/
4. W3C WAI-ARIA APG, Treegrid pattern. https://www.w3.org/WAI/ARIA/apg/patterns/treegrid/
5. Adrian Roselli, "Sortable Table Columns". https://adrianroselli.com/2021/04/sortable-table-columns.html
6. Adrian Roselli, "Fixed Table Headers". https://adrianroselli.com/2020/01/fixed-table-headers.html
7. Adrian Roselli, "A Responsive Accessible Table". https://adrianroselli.com/2017/11/a-responsive-accessible-table.html
8. Sarah Higley, "Grids Part 1: To grid or not to grid". https://sarahmhigley.com/writing/grids-part1/
9. Sarah Higley, "Grids Part 2: Semantics". https://sarahmhigley.com/writing/grids-part2/
10. React Aria, Table. https://react-aria.adobe.com/Table
11. React Aria, Selection. https://react-aria.adobe.com/selection
12. AG Grid, Accessibility. https://www.ag-grid.com/javascript-data-grid/accessibility/
13. AG Grid, Keyboard navigation. https://www.ag-grid.com/javascript-data-grid/keyboard-navigation/
14. MUI X Data Grid, Accessibility. https://mui.com/x/react-data-grid/accessibility/
15. MUI X Data Grid, Row height. https://mui.com/x/react-data-grid/row-height/
16. MUI X source (MIT), density factors. https://github.com/mui/mui-x/blob/master/packages/x-data-grid/src/hooks/features/density/densitySelector.ts
17. Carbon, Data table style. https://carbondesignsystem.com/components/data-table/style/
18. Carbon, Data table accessibility. https://carbondesignsystem.com/components/data-table/accessibility/
19. Fluent UI v9 source (MIT), TableCell styles. https://github.com/microsoft/fluentui/blob/master/packages/react-components/react-table/library/src/components/TableCell/useTableCellStyles.styles.ts
20. Fluent UI v8 source (MIT), DetailsRow styles. https://github.com/microsoft/fluentui/blob/master/packages/react/src/components/DetailsList/DetailsRow.styles.ts
21. Material Components Web (MIT), data table theme. https://github.com/material-components/material-components-web/blob/master/packages/mdc-data-table/_data-table-theme.scss
22. GOV.UK Design System, Table. https://design-system.service.gov.uk/components/table/
23. GitHub Primer, DataTable. https://primer.style/product/components/data-table/
24. Atlassian Design System, Dynamic table usage. https://atlassian.design/components/dynamic-table/usage
25. Handsontable, Keyboard shortcuts (behaviour only). https://handsontable.com/docs/javascript-data-grid/keyboard-shortcuts/
26. Handsontable, Accessibility (behaviour only). https://handsontable.com/docs/javascript-data-grid/accessibility/
27. Microsoft Support, Keyboard shortcuts in Excel. https://support.microsoft.com/en-us/office/keyboard-shortcuts-in-excel-1798d9d5-842a-42b8-9c99-9b7213f0040f
28. W3C, Understanding WCAG 2.4.11 Focus Not Obscured (Minimum). https://www.w3.org/WAI/WCAG22/Understanding/focus-not-obscured-minimum.html
29. W3C, Understanding WCAG 1.4.10 Reflow. https://www.w3.org/WAI/WCAG22/Understanding/reflow.html
30. W3C, Understanding WCAG 4.1.3 Status Messages. https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html
31. W3C, Understanding WCAG 2.5.8 Target Size (Minimum). https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html (not reopened; well-known 24×24 rule)
32. AccessKit, `Node` API (docs.rs). https://docs.rs/accesskit/latest/accesskit/struct.Node.html
33. React Spectrum pull request on keyboard column resizing (Esc/Enter returns to the header), found via search; exact PR number **[unverified]**. https://github.com/adobe/react-spectrum/pulls?q=column+resize+keyboard
34. "Making React Window Accessible for Screen Reader Users" (stale aria-rowcount after filtering). https://accessible-data-interfaces.pages.dev/virtualization-charts-dynamic-data-displays/accessible-virtualized-list-patterns/making-react-window-accessible-for-screen-reader-users/

Not covered from a primary source: TanStack Table (headless; ARIA and keys are left
to the app [unverified]), Tabulator (the docs only say "Aria tags are used on all
elements", with no details), Google Sheets, Deque/WebAIM table articles.
