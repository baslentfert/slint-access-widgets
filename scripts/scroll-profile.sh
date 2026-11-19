#!/bin/sh
# Where the time goes while a table scrolls: gdb takes a stack N times
# (default 12) while the mouse wheel turns, in an invisible display of its
# own; prints the stacks' innermost frames that belong to Slint, parley and
# the example.
#   scripts/scroll-profile.sh [APP] [N] > stacks.txt
# Needs: Xvfb, xdotool, gdb; the example built in release.
set -e
app=${1:-gallery}
n=${2:-12}
dir=$(cd "$(dirname "$0")/.." && pwd)
Xvfb :84 -screen 0 1100x800x24 -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
trap 'kill $wheel $app_pid $xvfb 2>/dev/null' EXIT
sleep 1
DISPLAY=:84 LANG=C.UTF-8 LC_ALL=C.UTF-8 "$dir/examples/$app/target/release/$app" >/dev/null 2>&1 &
app_pid=$!
sleep 5
win=$(DISPLAY=:84 xdotool search --sync --onlyvisible --pid $app_pid | tail -1)
DISPLAY=:84 xdotool windowmove $win 0 0 windowsize $win 1100 800
DISPLAY=:84 xdotool mousemove 500 450
sh -c 'while true; do DISPLAY=:84 xdotool click 5; sleep 0.05; done' &
wheel=$!
sleep 1
i=0
while [ $i -lt $n ]; do
  echo "=== sample $i"
  gdb -p $app_pid -batch -ex "thread 1" -ex "bt 60" 2>/dev/null | grep '^#' | cut -c1-220
  i=$((i + 1))
done
