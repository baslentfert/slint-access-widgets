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


# (key, text the focus must contain, text an announcement must contain)
SCENARIOS = {
    "gallery": [
        ("Tab", 'entry "Filter files"', None),
        ("Tab", 'combo box "Rows"', None),
        ("Tab", 'button "Reload"', None),
        ("Tab", 'button "Name" #1', None),
        ("Return", '"Name, sorted ascending"', "Name, sorted ascending"),
        ("Return", '"Name, sorted descending"', "Name, sorted descending"),
        ("Return", '"Name, sorted ascending"', None),
        ("Tab", 'list item "backup-00004.zip, Size 32 kB, Modified in May" #1', None),
        ("End", "#2000", None),
        ("Home", "#1", None),
        ("Next", "#", None),
        ("Home", "#1", None),
        ("shift+Down", "#2 selected", "2 selected"),
        ("shift+Down", "#3 selected", "3 selected"),
        ("ctrl+Down", "#4", None),
        ("space", "#4 selected", "4 selected"),
        ("ctrl+a", "selected", "2000 selected"),
        ("Right", "list item", "Size, "),
        ("Right", "list item", "Modified, "),
        ("i", 'list item "invoice-', None),
        ("shift+Tab", 'button "Name, sorted ascending"', None),
        ("Tab", "list item", None),
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
}


def run(name):
    exe = os.path.join(ROOT, "examples", name, "target", "debug", name)
    # English, whatever the system language is.
    env = dict(os.environ, LANGUAGE="en", LANG="C.UTF-8", LC_ALL="C.UTF-8")
    app_proc = subprocess.Popen([exe], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    try:
        app = None
        for _ in range(60):
            app = next((a for a in pyatspi.Registry.getDesktop(0) if a is not None and a.name == name), None)
            if app:
                break
            time.sleep(0.25)
        if not app:
            print(f"{name}: not on the accessibility bus")
            return False
        win = subprocess.run(["xdotool", "search", "--sync", "--onlyvisible", "--pid", str(app_proc.pid)], capture_output=True, text=True).stdout.split()
        subprocess.run(["xdotool", "windowfocus", "--sync", win[-1]])
        pump(1.0)
        heard.clear()
        for key, want_focus, want_heard in SCENARIOS[name]:
            subprocess.run(["xdotool", "key", "--clearmodifiers", key], check=True)
            pump(0.5)
            f = focused(app)
            got = describe(f) if f else "nothing"
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
