#!/bin/sh
# Runs a command in its own invisible X display with its own session bus
# and AT-SPI, so tests never touch your desktop:
#   scripts/headless.sh sh -c 'app & sleep 2; scripts/atspi-probe.py …'
set -e
exec dbus-run-session -- sh 2>/dev/null -c '
  Xvfb :77 -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 &
  XVFB=$!
  export DISPLAY=:77 GTK_A11Y=atspi NO_AT_BRIDGE=0
  /usr/libexec/at-spi-bus-launcher --launch-immediately >/dev/null 2>&1 &
  sleep 1
  # Tell toolkits (AccessKit) that assistive technology is listening.
  busctl --user set-property org.a11y.Bus /org/a11y/bus org.a11y.Status IsEnabled b true 2>/dev/null || true
  busctl --user set-property org.a11y.Bus /org/a11y/bus org.a11y.Status ScreenReaderEnabled b true 2>/dev/null || true
  "$@"; RC=$?
  kill $XVFB 2>/dev/null
  exit $RC
' sh "$@"
