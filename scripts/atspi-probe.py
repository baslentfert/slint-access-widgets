#!/usr/bin/env python3
"""Reads what a screen reader (Orca) sees of an app over AT-SPI: the tree,
and after each key where the focus is. Run inside scripts/headless.sh.

    atspi-probe.py APP-NAME "Tab Down Down Next End Up Home b" [depth] [pid]
"""
import sys, time, subprocess
import pyatspi
from gi.repository import GLib

# What a screen reader would speak by itself: announcements (live regions).
heard = []
def on_announcement(e):
    heard.append(str(e.any_data))
pyatspi.Registry.registerEventListener(on_announcement, "object:announcement")

def pump(seconds):
    """Let AT-SPI events come in."""
    ctx = GLib.MainContext.default()
    end = time.time() + seconds
    while time.time() < end:
        while ctx.pending():
            ctx.iteration(False)
        time.sleep(0.02)

app_name, keys = sys.argv[1], sys.argv[2].split()
depth = int(sys.argv[3]) if len(sys.argv) > 3 else 3

def line(a, d=0):
    try:
        st = a.getState()
        extra = []
        if st.contains(pyatspi.STATE_SELECTED): extra.append("selected")
        if st.contains(pyatspi.STATE_FOCUSED): extra.append("focused")
        attrs = dict(x.split(":", 1) for x in a.getAttributes() if ":" in x)
        for k in ("posinset", "setsize", "live", "container-live"):
            if k in attrs: extra.append(f"{k}={attrs[k]}")
        return "  " * d + f'{a.getRoleName()} "{a.name}"' + (" [" + ", ".join(extra) + "]" if extra else "")
    except Exception as e:
        return "  " * d + f"? {e}"

def tree(a, d=0):
    print(line(a, d))
    if d >= depth: return
    for c in a:
        if c is not None: tree(c, d + 1)

def find_app():
    for _ in range(40):
        for a in pyatspi.Registry.getDesktop(0):
            if a is not None and a.name == app_name:
                return a
        time.sleep(0.25)
    sys.exit(f"app {app_name!r} not on the AT-SPI bus")

def focused(a):
    st = a.getState()
    if st.contains(pyatspi.STATE_FOCUSED): return a
    for c in a:
        if c is None: continue
        f = focused(c)
        if f: return f
    return None

def live(a, out=None):
    """Live regions with something in them (status messages)."""
    out = [] if out is None else out
    try:
        attrs = dict(x.split(":", 1) for x in a.getAttributes() if ":" in x)
        if any("live" in key for key in attrs) and a.name.strip():
            out.append(a)
        for c in a:
            if c is not None: live(c, out)
    except Exception:
        pass
    return out

app = find_app()
time.sleep(1)
# No window manager in Xvfb: give the window keyboard focus ourselves.
win = subprocess.run(["xdotool", "search", "--sync", "--onlyvisible", "--pid", sys.argv[4]] if len(sys.argv) > 4 else
                     ["xdotool", "search", "--sync", "--onlyvisible", "--name", "."], capture_output=True, text=True).stdout.split()
if win:
    subprocess.run(["xdotool", "windowfocus", "--sync", win[-1]])
    time.sleep(0.3)
print("== tree"); tree(app)
print("== focus per key")
for k in [""] + keys:
    if k == "sleep":
        pump(1.3)
    elif k:
        subprocess.run(["xdotool", "key", "--clearmodifiers", k], check=True)
        pump(0.4)
    f = focused(app)
    print(f"after {k or '(start)'}: " + (line(f) if f else "nothing focused"))
    for h in heard:
        print(f"    announced: {h!r}")
    heard.clear()
