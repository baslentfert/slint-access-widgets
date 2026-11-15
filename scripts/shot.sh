#!/bin/sh
# A screenshot of an example in an invisible display (never your desktop):
#   scripts/shot.sh APP OUT.png WIDTHxHEIGHT [xdotool steps …]
# Steps are passed to xdotool one by one, e.g. "key Tab" "mousemove 200 160
# click 1" "sleep 1"; "shot NAME.png" takes an extra screenshot midway.
# Needs: Xvfb, xdotool, xwd, ffmpeg; the example built in release.
set -e
app=$1; out=$2; size=$3; shift 3
w=${size%x*}; h=${size#*x}
dir=$(cd "$(dirname "$0")/.." && pwd)
exec "$dir/scripts/headless.sh" sh -c '
  app=$1; out=$2; w=$3; h=$4; dir=$5; shift 5
  tmp=$(mktemp -d)
  LANG=C.UTF-8 LC_ALL=C.UTF-8 "$dir/examples/$app/target/release/$app" >/dev/null 2>&1 &
  pid=$!
  sleep 3
  win=$(xdotool search --sync --onlyvisible --pid $pid | tail -1)
  xdotool windowmove $win 0 0 windowsize $win $w $h windowfocus --sync $win
  sleep 0.5
  snap() { xwd -root -silent > "$tmp/s.xwd"; ffmpeg -loglevel error -y -i "$tmp/s.xwd" -vf crop=$w:$h:0:0 "$1"; }
  for step in "$@"; do
    case $step in
      shot\ *) snap "${step#shot }" ;;
      sleep\ *) sleep "${step#sleep }" ;;
      *) xdotool $step; sleep 0.3 ;;
    esac
  done
  sleep 0.5
  snap "$out"
  kill $pid
  rm -rf "$tmp"
' sh "$app" "$out" "$w" "$h" "$dir" "$@"
