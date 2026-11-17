#!/bin/sh
# Runs a command in its own invisible X display with its own session bus
# and AT-SPI, so tests never touch your desktop:
#   scripts/headless.sh sh -c 'app & sleep 2; scripts/atspi-probe.py …'
set -e
# The bus and the display are noisy on stderr; the command's own errors
# (a Python traceback) must still show: they go out on fd 3.
exec 3>&2
exec dbus-run-session -- sh 2>/dev/null -c '
  Xvfb :77 -screen 0 1280x800x24 -nolisten tcp >/dev/null 2>&1 &
  XVFB=$!
  export DISPLAY=:77 GTK_A11Y=atspi NO_AT_BRIDGE=0
  launcher=$(ls /usr/libexec/at-spi-bus-launcher /usr/lib/at-spi2-core/at-spi-bus-launcher 2>/dev/null | head -1)
  "$launcher" --launch-immediately >/dev/null 2>&1 &
  sleep 1
  # Tell toolkits (AccessKit) that assistive technology is listening, on
  # this private session bus (not the desktop one).
  for p in IsEnabled ScreenReaderEnabled; do
    gdbus call --session --dest org.a11y.Bus --object-path /org/a11y/bus \
      --method org.freedesktop.DBus.Properties.Set org.a11y.Status $p "<true>" >/dev/null 2>&1 || true
  done
  "$@" 2>&3; RC=$?
  kill $XVFB 2>/dev/null
  exit $RC
' sh "$@"
