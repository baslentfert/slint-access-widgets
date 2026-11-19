#!/bin/sh
# What one mouse-wheel step in a table costs: CPU seconds of the app for ten
# wheel steps (xdotool click 5, the XTest path a remote desktop uses too), in
# an invisible display of its own (never your desktop):
#   scripts/scroll-cost.sh [APP]        (default: gallery, release build)
# Needs: Xvfb, xdotool; the example built in release.
set -e
app=${1:-gallery}
dir=$(cd "$(dirname "$0")/.." && pwd)
Xvfb :83 -screen 0 1100x800x24 -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
trap 'kill $app_pid $xvfb 2>/dev/null' EXIT
sleep 1
# (LOG=file keeps what the app prints, e.g. with
# SLINT_DEBUG_PERFORMANCE=refresh_lazy,console: frames per second)
DISPLAY=:83 LANG=C.UTF-8 LC_ALL=C.UTF-8 "$dir/examples/$app/target/release/$app" >"${LOG:-/dev/null}" 2>&1 &
app_pid=$!
sleep 5
win=$(DISPLAY=:83 xdotool search --sync --onlyvisible --pid $app_pid | tail -1)
DISPLAY=:83 xdotool windowmove $win 0 0 windowsize $win 1100 800
DISPLAY=:83 xdotool mousemove 500 450
sleep 1
# utime + stime (fields 14 and 15 of /proc/PID/stat), in clock ticks.
ticks() { cut -d' ' -f14,15 /proc/$app_pid/stat | tr ' ' '+' | bc; }
hz=$(getconf CLK_TCK)
before=$(ticks)
for _ in 1 2 3 4 5 6 7 8 9 10; do
  # (IDLE=1: no wheel, to see what the app costs doing nothing)
  [ -n "$IDLE" ] || DISPLAY=:83 xdotool click 5
  sleep 0.1
done
sleep 2
after=$(ticks)
echo "$app: $(echo "scale=3; ($after - $before) / $hz" | bc) s CPU for 10 wheel steps ($(echo "scale=3; ($after - $before) / $hz / 10" | bc) s per step)"
