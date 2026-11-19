# A modal dialog in Slint: research notes

What `AccessDialog` is built on, and where Slint 1.19 sets the limits.

## What the pattern asks

WAI-ARIA Authoring Practices, Dialog (Modal) pattern,
<https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/>:

- **Focus in:** on open, to the first focusable element; to a static
  element at the start when the content is long or structured; "consider"
  the least destructive action when the last step can't be undone; the most
  used button (OK, Continue) for dialogs that only inform.
- **Focus stays in:** Tab and Shift+Tab move through the dialog only, and
  wrap from the last element to the first and back.
- **Escape** closes the dialog.
- **Focus back:** to the element that opened it (unless it is gone, or the
  task naturally goes on somewhere else).
- **A visible close or Cancel button** in the Tab order is strongly
  recommended.
- **Roles:** `role="dialog"`, `aria-modal="true"`, a name (`aria-labelledby`
  the visible title) and, for simple content, a description
  (`aria-describedby`).
- **Inert:** the content behind cannot be used; mark a dialog modal only
  when that is really so and the content behind is visually obscured.

## What Slint 1.19 offers

- **`PopupWindow`** with `close-policy: no-auto-close`
  (`i-slint-compiler/builtin_elements.rs`). Seen in
  `i-slint-core/window.rs`:
  - `show_popup` takes the focus from the window and keeps it
    (`focus_item_in_parent`); closing gives it back to that element
    (`close_popup_impl`). That is the APG's "focus back", for free.
  - While a non-menu popup is open, mouse events outside it go nowhere
    (`process_mouse_input`: the loop over active popups stops at the first
    one that is not a menu). A popup that covers the whole window therefore
    makes the rest unreachable for the mouse, and draws the dimmed cover.
  - Escape closes a popup by itself only with the close-on-click policies;
    with `no-auto-close` the dialog decides.
  - `@children` can be placed inside a popup, but elements there can't be
    read from outside it ("Access to property … which is inlined into a
    PopupWindow via @children is forbidden"; slint#4438): content inside
    the dialog hands its values out itself (`edited(t) => { value = t; }`).
- **Focus:** `focus()` on an element; no "focus next/previous" function.
  But asking a `FocusScope` with `enabled: false` for the focus makes Slint
  walk on to the next element that takes it (`WindowInner::move_focus`):
  that is how the dialog finds its first focusable element without knowing
  your content. Two 1 px `FocusScope`s at the start and the end catch Tab
  and Shift+Tab leaving and send the focus round (the "focus guards" of web
  focus traps).
- **The `Dialog` element** is a window of its own, with its button order per
  platform (`organize_dialog_button_layout` in `i-slint-core/layout.rs`:
  macOS Cancel then OK, Windows and Linux (KDE and GNOME alike) OK then
  Cancel). It is a separate top-level window, not modal to another one
  (no API for that in 1.19), so it is not used here; its order is.
- **Roles:** `AccessibleRole` (i-slint-common `enums.rs`) has no `dialog`
  and no way to say "modal" (the AccessKit adapter in
  `i-slint-backend-winit/accesskit.rs` never sets `Role::Dialog` or
  `modal`). The dialog is a `groupbox` (AT-SPI "panel") with the title as
  its name and the description as its description: a screen reader says
  both when the focus enters it.

## Choices

- **Button order:** as Slint's own `Dialog` (platform order from
  `Platform.os`): macOS Cancel, primary; elsewhere primary, Cancel; right
  aligned. (GNOME's HIG puts the affirmative button on the right, as macOS
  does; Slint does not, and following Slint keeps both kinds of dialog in
  one app the same.)
- **Initial focus:** the first element by default; `initial-focus: cancel`
  for things that can't be undone. (Not "your own element": content can't
  be reached from outside the popup, and an element that takes the focus
  in its `init` does so before Slint takes the focus for the popup, which
  would then give the focus "back" to it on close.)
- **Enter** is not caught by the dialog: a focused button or field handles
  it, so a multi-line field keeps its Enter.

## Not possible yet

- No `dialog` role or `aria-modal`: screen readers hear a group, and their
  browse mode can still read (and AT-SPI actions can still press) what is
  behind the dialog. Needs a dialog role and modal flag in Slint's
  `AccessibleRole` and AccessKit adapter.
