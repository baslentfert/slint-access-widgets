# Icons and lines in trees

What common trees do (checked November 2026), and what `AccessTree` does.

## An icon per row

- **WinUI TreeView** (and File Explorer's navigation pane): "The tree view
  uses a combination of indentation and icons"; for a file system "you could
  use folder icons for the parent nodes and file icons for the leaf nodes"
  (data templates per kind).
  <https://learn.microsoft.com/en-us/windows/apps/design/controls/tree-view>
- **Fluent 2 Tree**: a leading icon shows what kind of item it is; children
  are indented 24 px (12 px small), leaves another 24 px to leave room for
  the chevron.
  <https://fluent2.microsoft.design/components/web/react/core/tree/usage>
- **WAI-ARIA APG, "File Directory" treeview example**: a closed folder,
  an open folder (while `aria-expanded="true"`) and a document icon, added
  with CSS `::before`, so they are not part of the markup.
  <https://www.w3.org/WAI/ARIA/apg/patterns/treeview/examples/treeview-1a/>,
  CSS: <https://github.com/w3c/aria-practices/blob/main/content/patterns/treeview/examples/css/tree.css>
- **GTK4 TreeExpander** (Nautilus' tree, file choosers): `indent-for-icon`
  (default true) indents a row without children by the expander's width, so
  icons and text line up.
  <https://docs.gtk.org/gtk4/property.TreeExpander.indent-for-icon.html>
- macOS Finder (list view) and VS Code's explorer show a folder or file icon
  per row too.

So: an icon is usual, closed and open folders differ, and the space of the
triangle stays on rows without children. The icon is decoration: what a row
is follows from its name and from "expanded"/"collapsed", so screen readers
should not read it. In Slint that is `accessible-role: none`, set where the
icon is used (on the component's own root alone it still came through to
AT-SPI as an empty node; `scripts/a11y-test.py` checks for that).

## Lines

- Classic connecting lines (Win32 `TVS_HASLINES`, GTK3 `enable-tree-lines`)
  are off by default (GTK: `FALSE`) and gone from modern File Explorer,
  Finder, WinUI and GTK4.
  <https://docs.gtk.org/gtk4/property.TreeView.enable-tree-lines.html>
- **VS Code** has thin indent guides: `workbench.tree.renderIndentGuides`,
  `none` / `onHover` / `always`, default `onHover`; indent 8 px.
  <https://github.com/microsoft/vscode/blob/main/src/vs/platform/list/browser/listService.ts>
- The APG examples have no lines.

So `AccessTree` has `indent-guides: none | on-hover | always`, default
`none` like most file trees. `on-hover` also shows them while the tree has
the focus, so keyboard users get them too. The guide is the text colour,
faint (`AccessColors.guide`); on the selected row the row's text colour.
