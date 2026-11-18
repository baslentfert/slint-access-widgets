#!/usr/bin/env python3
"""Checks what a screen reader gets from the examples, key by key: where the
focus is (role, name, position) and what is announced. Fails on the first
difference. Run it inside scripts/headless.sh (its own X display and
accessibility bus, so it never touches your desktop):

    scripts/headless.sh python3 -I scripts/a11y-test.py [gallery|transfer ...]

Needs: Xvfb, xdotool, at-spi2-core, python3-pyatspi; the examples built
(`cargo build` in examples/gallery and examples/transfer).
"""
import os, subprocess, sys, time

import pyatspi
from gi.repository import GLib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
heard = []
pyatspi.Registry.registerEventListener(lambda e: heard.append(str(e.any_data).strip()), "object:announcement")


def pump(seconds):
    ctx = GLib.MainContext.default()
    end = time.time() + seconds
    while time.time() < end:
        while ctx.pending():
            ctx.iteration(False)
        time.sleep(0.02)


def describe(a):
    st = a.getState()
    attrs = dict(x.split(":", 1) for x in a.getAttributes() if ":" in x)
    d = f'{a.getRoleName()} "{a.name}"'
    if "posinset" in attrs:
        d += f' #{attrs["posinset"]}'
    if st.contains(pyatspi.STATE_SELECTED):
        d += " selected"
    if a.getRoleName() in ("radio button", "check box", "radio menu item", "check menu item"):
        d += " checked" if st.contains(pyatspi.STATE_CHECKED) else " unchecked"
    if not st.contains(pyatspi.STATE_ENABLED):
        d += " unavailable"
    if st.contains(pyatspi.STATE_EXPANDABLE):
        d += " expanded" if st.contains(pyatspi.STATE_EXPANDED) else " collapsed"
    # (AccessKit doesn't pass "expanded" to AT-SPI yet: AccessKit #824.
    # The description shows what a screen reader reads with the name.)
    if a.description:
        d += f' desc="{a.description[:30]}"'
    else:
        d += " no-desc"
    return d


def focused(a):
    try:
        if a.getState().contains(pyatspi.STATE_FOCUSED):
            return a
        for c in a:
            if c is not None and (f := focused(c)):
                return f
    except Exception:
        pass
    return None


def find(a, text):
    """The first element (not only the focused one) whose description
    contains `text`."""
    try:
        if text in describe(a):
            return a
        for c in a:
            if c is not None and (f := find(c, text)):
                return f
    except Exception:
        pass
    return None


# (key, text the focus must contain, text an announcement must contain).
# Besides keys: "find" (an element with that text must exist, focused or
# not), "gone" (it must not), "hover" (the mouse onto it), "unhover" (the
# mouse into the window's top left corner), "wait" (1 s) or "wait N" (N s).
SCENARIOS = {
    "gallery": [
        ("Tab", 'entry "Filter files"', None),
        ("Tab", 'combo box "Rows"', None),
        ("Tab", 'button "Reload"', None),
        ("Tab", 'button "Name" #1', None),
        ("Return", '"Name, sorted ascending"', "Name, sorted ascending"),
        ("Return", '"Name, sorted descending"', "Name, sorted descending"),
        ("Return", '"Name, sorted ascending"', None),
        # The current row stayed on its file while sorting by name.
        ("Tab", 'list item "report-00000.pdf, Size 1 kB, Modified today, Type PDF document, Folder ', None),
        ("Home", "#1", None),
        ("End", "#2000", None),
        ("Up", "#1999", None),
        ("Down", "#2000", None),
        ("Home", "#1", None),
        ("shift+Tab", 'button "Name, sorted ascending"', None),
        ("Tab", "#1", None),
        ("End", "#2000", None),
        ("Home", "#1", None),
        ("Next", "#", None),
        ("Home", "#1", None),
        ("shift+Down", "#2 selected", "2 selected"),
        ("shift+Down", "#3 selected", "3 selected"),
        # Copy: announced, and really on the clipboard, as columns.
        ("ctrl+c", "#3 selected", "3 rows copied"),
        ("clipboard", "\t", "3"),
        ("ctrl+Down", "#4", None),
        ("space", "#4 selected", "4 selected"),
        ("ctrl+a", "selected", "2000 selected"),
        ("Right", "list item", "Size, "),
        ("Right", "list item", "Modified, "),
        # Further right: the table scrolls sideways (checked on screenshots).
        ("Right", "list item", "Type, "),
        ("Right", "list item", "Folder, "),
        ("Right", "list item", "Owner, "),
        ("Right", "list item", "Owner, "),
        ("Left", "list item", "Folder, "),
        ("i", 'list item "invoice-', None),
        ("shift+Tab", 'button "Name, sorted ascending"', None),
        ("Tab", "list item", None),
        # The selection stays on the same files after sorting.
        ("Home", "#1 selected", None),
        ("shift+Down", "#2 selected", "2 selected"),
        ("ctrl+c", "#2 selected", "2 rows copied"),
        ("clipboard", "backup-00004.zip", "2"),
        ("shift+Tab", 'button "Name, sorted ascending"', None),
        ("Right", 'button "Size"', None),
        ("Return", '"Size, sorted ascending"', "Size, sorted ascending"),
        # Back in the table: still on the same file, and still selected.
        ("Tab", 'list item "backup-00012.zip', None),
        ("ctrl+c", "selected", "2 rows copied"),
        ("clipboard", "backup-00004.zip", "2"),
        # Sort on two columns: Type, then (Shift+Enter) Size.
        ("shift+Tab", 'button "Size, sorted ascending"', None),
        ("Right", 'button "Modified"', None),
        ("Right", 'button "Type"', None),
        ("Return", '"Type, sorted ascending"', "Type, sorted ascending"),
        ("Left", 'button "Modified"', None),
        ("Left", 'button "Size"', None),
        ("shift+Return", '"Size, sorted ascending, level 2"', "Type, sorted ascending, then Size, sorted ascending"),
        ("shift+Return", '"Size, sorted descending, level 2"', "then Size, sorted descending"),
        ("Right", 'button "Modified"', None),
        ("Right", 'button "Type, sorted ascending, level 1"', None),
        ("Tab", "list item", None),
        # First rows: the first type (Application), biggest first.
        ("Home", 'Type Application', None),
        ("Down", 'Type Application', None),
        # The column menu: a list a screen reader follows, item by item.
        ("shift+Tab", "button", None),
        ("Home", 'button "Name', None),
        ("Right", 'button "Size', None),
        ("Menu", 'list item "Sort ascending" #1', None),
        ("Down", 'list item "Sort descending" #2', None),
        ("End", 'list item "Owner" #13', None),
        ("Return", 'button "Size', "Owner, hidden"),
        ("Menu", 'list item "Sort ascending"', None),
        ("End", 'list item "Owner" #13', None),
        ("Return", 'button "Size', "Owner, shown"),
        # The same without the menu: Delete hides, Ctrl+Shift+arrows move.
        ("Delete", 'button "Modified', "Size, hidden"),
        ("Left", 'button "Name', None),
        ("Menu", "list item", None),
        ("Escape", 'button "Name', None),
        ("ctrl+shift+Right", 'button "Name', "Name, column 2 of 6"),
        ("ctrl+shift+Left", 'button "Name', "Name, column 1 of 6"),
        # The row menu: also followed item by item; the focus comes back.
        ("Tab", "list item", None),
        ("Menu", 'list item "Open" #1', None),
        ("Down", 'list item "Copy" #2', None),
        ("Down", 'list item "Delete" #4', None),
        ("Return", "list item", "(not really)"),
        ("shift+F10", 'list item "Open" #1', None),
        ("Escape", "list item", None),
    ],
    "transfer": [
        ("Tab", 'list item "holiday-00.mp4, 6 MB" #1', None),
        ("space", '"holiday-00.mp4, 6 MB, ticked" #1 selected', "1 ticked"),
        ("Down", 'list item "invoice-01.pdf', None),
        ("space", "ticked", "2 ticked"),
        ("Tab", 'button "Send 2 files"', None),
        ("Return", 'button "Send"', "2 files are being sent"),
        ("Tab", 'button "⏸ Pause"', None),
        ("Return", 'button "▶ Resume"', "Paused"),
        ("Return", 'button "⏸ Pause"', "Resumed"),
        ("shift+Tab", "list item", None),
        ("Up", "list item", None),
    ],
    "autocomplete": [
        ("Tab", 'entry "Computer"', None),
        ("t", 'entry "Computer"', None),
        ("e", 'entry "Computer"', None),
        ("s", 'entry "Computer"', None),
        ("t", 'entry "Computer"', None),
        ("wait", 'entry "Computer"', "3 suggestions"),
        ("Down", 'list item "test-vm-01" #1', None),
        ("Down", 'list item "test-vm-02" #2', None),
        ("End", 'list item "test-vm-03" #3', None),
        ("Home", 'list item "test-vm-01" #1', None),
        ("Up", 'entry "Computer"', None),
        ("text", "test", None),
        ("Down", 'list item "test-vm-01" #1', None),
        ("Down", 'list item "test-vm-02" #2', None),
        ("Return", 'entry "Computer"', None),
        ("text", "test-vm-02", None),
        # Escape empties the field (the list is closed already).
        ("Escape", 'entry "Computer"', None),
        ("text", "", None),
        ("alt+Down", 'entry "Computer"', None),
        ("wait", 'entry "Computer"', "14 suggestions"),
        ("Escape", 'entry "Computer"', None),
        ("a", 'entry "Computer"', None),
        ("l", 'entry "Computer"', None),
        ("wait", 'entry "Computer"', "1 suggestion"),
        ("Down", 'list item "alan-desktop" #1', None),
        # Typing in the list goes on in the field.
        ("v", 'entry "Computer"', None),
        ("text", "alv", None),
        ("wait", 'entry "Computer"', "No suggestions"),
    ],
    "controls": [
        ("Tab", 'check box "Share the clipboard"', None),
        ("Tab", 'button "More about Share the clipboard" no-desc', None),
        # Opens; the explanation is announced; the focus stays on the button.
        ("Return", 'button "More about Share the clipboard" desc="What you copy', "What you copy on one computer"),
        ("Escape", 'button "More about Share the clipboard" no-desc', None),
        ("space", 'button "More about Share the clipboard" desc="What you copy', "What you copy"),
        ("space", 'button "More about Share the clipboard" no-desc', None),
        ("Tab", 'check box "Send sound"', None),
        ("Tab", 'button "More about Send sound" no-desc', None),
        ("Return", 'button "More about Send sound" desc="The sound', "The sound of this computer"),
        # Tab away: the explanation may stay (WCAG 1.4.13: until dismissed).
        ("Tab", 'radio button "There" #2 checked', None),
        ("shift+Tab", 'button "More about Send sound" desc="The sound', None),
        ("Escape", 'button "More about Send sound" no-desc', None),
        # Segmented: one Tab stop, on the chosen option.
        ("Tab", 'radio button "There" #2 checked', None),
        ("Right", 'radio button "Off" #3 checked', None),
        ("Right", 'radio button "Here" #1 checked', None),
        ("Left", 'radio button "Off" #3 checked', None),
        ("Tab", 'radio button "Both ways" #1 checked', None),
        # "To this computer" is not available: skipped.
        ("Right", 'radio button "To that computer" #2 checked', None),
        ("Right", 'radio button "Off" #4 checked', None),
        ("Home", 'radio button "Both ways" #1 checked', None),
        ("End", 'radio button "Off" #4 checked', None),
        ("shift+Tab", 'radio button "Off" #3 checked', None),
        ("Tab", 'radio button "Off" #4 checked', None),
        ("Tab", 'button "Save"', None),
    ],
    "tree": [
        ("Tab", 'list item "Documents" #1', None),
        # Right opens (announced on Linux: AccessKit #824), then goes in.
        ("Right", 'desc="level 1, expanded"', "expanded, 3 items"),
        ("Right", 'list item "Projects" #2', None),
        ("Right", 'list item "Projects" #2', "expanded"),
        ("Down", 'list item "2025" #3', None),
        ("Down", 'list item "2026" #4', None),
        ("Right", 'desc="level 3, expanded"', "expanded, 3 items"),
        ("Right", 'list item "plan.pdf" #5', None),
        # Left: up to the parent, then closes it, then up again.
        ("Left", 'list item "2026" #4', None),
        ("Left", 'desc="level 3, collapsed"', "collapsed"),
        ("Left", 'list item "Projects" #2', None),
        ("Left", 'list item "Projects" #2', "collapsed"),
        ("Down", 'list item "Finance" #3', None),
        ("End", 'list item "Downloads"', None),
        # An empty folder: nothing to open.
        ("Right", 'list item "Downloads" #6 selected desc="level 1"', "Empty"),
        ("Home", 'list item "Documents" #1', None),
        # Type-ahead over the visible rows: Projects, then Pictures.
        ("p", 'list item "Projects" #2', None),
        ("p", 'list item "Pictures"', None),
    ],
    "transfer-list": [
        ("Tab", 'check box "Select all in Available" unchecked', None),
        ("Tab", 'list item "Type" #1', None),
        ("Down", 'list item "Modified" #2 selected', None),
        ("shift+Down", 'list item "Created" #3 selected', "2 selected"),
        # Enter moves both; the focus stays, on the item after them.
        ("Return", 'list item "Owner" #2 selected', "Moved to Shown: 2 items"),
        ("Return", 'list item "Permissions" #2 selected', "Moved to Shown: 1 item"),
        # Type-ahead.
        ("t", 'list item "Tags"', None),
        # The buttons say where items go.
        ("Tab", 'button "Add to Shown"', None),
        ("Tab", 'button "Remove from Shown"', None),
        ("Tab", 'check box "Select all in Shown" unchecked', None),
        # The moved items are selected over there.
        ("Tab", 'list item "Owner" #5 selected', None),
        # In order: Alt+Up, and the new place is announced.
        ("alt+Up", 'list item "Owner" #4 selected', "position 4 of 5"),
        ("Home", 'list item "Name" #1 selected', None),
        # Delete moves back.
        ("Delete", 'list item "Size" #1 selected', "Moved to Available: 1 item"),
        ("ctrl+a", 'list item "Size" #1 selected', "4 selected"),
        # Emptied: the focus goes along, to what just arrived.
        ("Return", 'list item "Size" #7 selected', "Moved to Available: 4 items"),
    ],
    "notices": [
        ("Tab", 'button "Save"', None),
        # An alert: shown in the window, announced; the focus stays.
        ("Return", 'button "Save"', "Success: Settings saved. They apply from the next connection."),
        ("find", '"Success: Settings saved"', None),
        # Errors are announced at once, with the button that comes with it.
        ("Tab", 'button "Send"', None),
        ("Return", 'button "Send"', "Error: 3 files could not be sent. The other computer has no room left. Try again is available"),
        ("find", '"Error: 3 files could not be sent"', None),
        ("find", 'button "Close message: 3 files could not be sent"', None),
        # A snackbar with a button: announced, never takes the focus...
        ("Tab", 'button "Delete a file"', None),
        ("Return", 'button "Delete a file"', "File deleted. Undo is available"),
        ("find", '"File deleted"', None),
        # ... but Tab reaches it, and it stays (it has a button).
        ("Tab", 'button "Copy link"', None),
        ("Tab", 'button "Undo"', None),
        ("wait 7", 'button "Undo"', None),
        ("Return", 'button "Delete a file"', None),
        ("gone", '"File deleted"', None),
        ("find", 'label "3 files"', None),
        # Without a button: away by itself, but not while it has the focus.
        ("Tab", 'button "Copy link"', None),
        ("Return", 'button "Copy link"', "Link copied"),
        ("Tab", 'button "Close message: Link copied"', None),
        ("wait 7", 'button "Close message: Link copied"', None),
        # Escape closes it; the focus goes back.
        ("Escape", 'button "Delete a file"', None),
        ("gone", '"Link copied"', None),
        # Nor while the mouse is on it; then it goes.
        ("Tab", 'button "Copy link"', None),
        ("Return", 'button "Copy link"', "Link copied"),
        ("hover", 'button "Close message: Link copied"', None),
        ("wait 7", 'button "Copy link"', None),
        ("find", '"Link copied"', None),
        ("unhover", "", None),
        ("wait 7", 'button "Copy link"', None),
        ("gone", '"Link copied"', None),
    ],
    "remote": [
        ("Tab", 'entry "Search on the server"', None),
        ("Tab", 'button "Name" #1', None),
        ("Tab", 'list item "report-000000.pdf, Size ', None),
        # Far away: "Loading…" first, then it arrives (0.4 s) in place.
        ("End", "list item", None),
        ("wait", 'list item "song-099999.mp3, Size ', None),
        ("Home", 'list item "report-000000.pdf', None),
        # A search on the server: the new total is announced.
        ("shift+Tab", "button", None),
        ("shift+Tab", "entry", None),
        ("9", "entry", None),
        ("9", "entry", None),
        ("9", "entry", None),
        ("9", "entry", None),
        ("wait", "entry", "of 100000 shown"),
        ("Tab", "button", None),
        ("Tab", 'list item "', None),
    ],
}


def find_app(name):
    """The app on the accessibility bus. An app that just quit (the previous
    scenario) may still be listed and fail when asked its name: skip it."""
    for a in pyatspi.Registry.getDesktop(0):
        try:
            if a is not None and a.name == name:
                return a
        except Exception:
            continue
    return None


def run(name):
    exe = os.path.join(ROOT, "examples", name, "target", "debug", name)
    # English, whatever the system language is.
    env = dict(os.environ, LANGUAGE="en", LANG="C.UTF-8", LC_ALL="C.UTF-8")
    log = open(f"/tmp/a11y-test-{name}.log", "w+")
    app_proc = subprocess.Popen([exe], env=env, stdout=log, stderr=subprocess.STDOUT)
    try:
        app = None
        for _ in range(60):
            app = find_app(name)
            if app:
                break
            time.sleep(0.25)
        if not app:
            print(f"{name}: not on the accessibility bus (running: {app_proc.poll() is None})")
            log.seek(0)
            print("  " + log.read()[-2000:].replace("\n", "\n  "))
            return False
        win = subprocess.run(["xdotool", "search", "--sync", "--onlyvisible", "--pid", str(app_proc.pid)], capture_output=True, text=True).stdout.split()
        subprocess.run(["xdotool", "windowfocus", "--sync", win[-1]])
        pump(1.0)
        heard.clear()
        for key, want_focus, want_heard in SCENARIOS[name]:
            if key == "clipboard":
                # The text on the clipboard must contain `want_focus` and
                # have three lines (three rows).
                clip = subprocess.run(["xclip", "-o", "-selection", "clipboard"], capture_output=True, text=True).stdout
                ok = want_focus in clip and clip.count("\n") == int(want_heard)
                print(f"{'ok  ' if ok else 'FAIL'} clipboard  {clip!r}")
                if not ok:
                    return False
                continue
            if key == "text":
                # The text in the focused field must be exactly `want_focus`.
                f = focused(app)
                try:
                    got = f.queryText().getText(0, -1)
                except Exception as e:
                    got = f"(no text: {e})"
                ok = got == want_focus
                print(f"{'ok  ' if ok else 'FAIL'} text       {got!r}")
                if not ok:
                    print(f"     wanted {want_focus!r}")
                    return False
                continue
            if key in ("find", "gone"):
                # Shown and hidden things can take a frame to reach the bus.
                t1 = time.time()
                while (find(app, want_focus) is None) == (key == "find") and time.time() - t1 < 2:
                    pump(0.1)
                f = find(app, want_focus)
                ok = (f is not None) == (key == "find")
                print(f"{'ok  ' if ok else 'FAIL'} {key:<10} {describe(f) if f else want_focus + ' (not there)'}")
                if not ok:
                    return False
                continue
            if key in ("hover", "unhover"):
                if key == "hover":
                    f = find(app, want_focus)
                    if f is None:
                        print(f"FAIL hover      {want_focus!r} not there")
                        return False
                    e = f.queryComponent().getExtents(pyatspi.DESKTOP_COORDS)
                    x, y = e.x + e.width // 2, e.y + e.height // 2
                else:
                    x, y = 5, 5
                subprocess.run(["xdotool", "mousemove", str(x), str(y)], check=True)
                pump(0.3)
                print(f"ok   {key:<10} {x},{y}")
                continue
            if key.startswith("wait"):
                pump(float(key[4:] or 1))
            else:
                subprocess.run(["xdotool", "key", "--clearmodifiers", key], check=True)
                t0 = time.time()
                pump(0.5)
                # The clipboard can take a moment on X11; give an
                # announcement up to 3 s (and say how long it took).
                while want_heard and not any(want_heard in h for h in heard) and time.time() - t0 < 3:
                    pump(0.1)
                if want_heard and time.time() - t0 > 0.7:
                    print(f"     (heard after {time.time() - t0:.1f} s)")
            f = focused(app)
            got = describe(f) if f else "nothing"
            # Slow to react? Wait up to 2 s more, and say how long it took.
            t1 = time.time()
            while want_focus not in got and time.time() - t1 < 2:
                pump(0.1)
                f = focused(app)
                got = describe(f) if f else "nothing"
            if want_focus in got and time.time() - t1 > 0.15:
                print(f"     (focus after {0.5 + time.time() - t1:.1f} s)")
            ok = want_focus in got and (want_heard is None or any(want_heard in h for h in heard))
            print(f"{'ok  ' if ok else 'FAIL'} {key:<10} {got}" + (f"   heard {heard}" if heard else ""))
            if not ok:
                print(f"     wanted focus with {want_focus!r}" + (f" and heard {want_heard!r}" if want_heard else ""))
                return False
            heard.clear()
        return True
    finally:
        app_proc.kill()


if __name__ == "__main__":
    names = sys.argv[1:] or list(SCENARIOS)
    results = {n: run(n) for n in names}
    print(" ".join(f"{n}: {'passed' if r else 'FAILED'}" for n, r in results.items()))
    sys.exit(0 if all(results.values()) else 1)
