"""Runs inside the e2e VM: starts wbindkeys, presses key combos on a uinput
virtual keyboard and checks which bindings fired.

Usage: press.py <wbindkeys binary>
"""

import os
import shutil
import subprocess
import sys
import time

from evdev import UInput, ecodes as e

HERE = os.path.dirname(os.path.abspath(__file__))
MARKS = "/tmp/wbindkeys-e2e"
LOG = os.path.join(HERE, "wbindkeys.log")

# (combo as pressed, keys in the order they are pressed, markers that should appear)
CASES = [
    ("Super+8 (AZERTY _)", [e.KEY_LEFTMETA, e.KEY_8], {"mod-8"}),
    ("8 alone", [e.KEY_8], set()),
    ("Super+1 (AZERTY &)", [e.KEY_LEFTMETA, e.KEY_1], {"mod-1"}),
    ("1 alone", [e.KEY_1], set()),
    ("Super+Shift+7 (QWERTY &)", [e.KEY_LEFTMETA, e.KEY_LEFTSHIFT, e.KEY_7], {"mod-shift-7"}),
    ("RightSuper+RightShift+- (QWERTY _)", [e.KEY_RIGHTMETA, e.KEY_RIGHTSHIFT, e.KEY_MINUS], {"mod-shift-dash"}),
    ("LeftAlt+E", [e.KEY_LEFTALT, e.KEY_E], {"alt-e"}),
    ("RightAlt+E", [e.KEY_RIGHTALT, e.KEY_E], {"alt-e"}),
    ("RightAlt+T", [e.KEY_RIGHTALT, e.KEY_T], {"rightalt-t"}),
    ("LeftAlt+T", [e.KEY_LEFTALT, e.KEY_T], set()),
    ("LeftAlt+K", [e.KEY_LEFTALT, e.KEY_K], {"alt-k"}),
    ("RightAlt+K", [e.KEY_RIGHTALT, e.KEY_K], {"rightalt-k"}),
    ("Ctrl+Alt+T", [e.KEY_LEFTCTRL, e.KEY_LEFTALT, e.KEY_T], {"ctrl-alt-t"}),
    ("Alt+Ctrl+T (out of order)", [e.KEY_LEFTALT, e.KEY_LEFTCTRL, e.KEY_T], set()),
]


def press(keyboard, keys):
    """Presses keys in order, then releases them in reverse, like a person would."""
    for key in keys:
        keyboard.write(e.EV_KEY, key, 1)
        keyboard.syn()
        time.sleep(0.05)
    for key in reversed(keys):
        keyboard.write(e.EV_KEY, key, 0)
        keyboard.syn()
        time.sleep(0.05)


def wait_for_log(text, timeout=10):
    deadline = time.time() + timeout
    while time.time() < deadline:
        if os.path.exists(LOG) and text in open(LOG).read():
            return True
        time.sleep(0.1)
    return False


def main():
    binary = sys.argv[1]
    keys = sorted({key for _, combo, _ in CASES for key in combo})

    # Create the keyboard before starting wbindkeys, so it is picked up when
    # libinput enumerates devices rather than depending on hotplug timing.
    with UInput({e.EV_KEY: keys}, name="wbindkeys-e2e keyboard") as keyboard:
        time.sleep(1)
        env = dict(os.environ, XDG_CONFIG_HOME=os.path.join(HERE, "config"))
        with open(LOG, "w") as log:
            wbindkeys = subprocess.Popen([binary, "--debug"], stderr=log, env=env)
        try:
            if not wait_for_log("listening for input events"):
                print("wbindkeys did not start, see", LOG)
                return 1
            time.sleep(1)

            failures = 0
            for name, combo, expected in CASES:
                shutil.rmtree(MARKS, ignore_errors=True)
                os.makedirs(MARKS)
                press(keyboard, combo)
                time.sleep(0.5)
                fired = set(os.listdir(MARKS))
                ok = fired == expected
                failures += not ok
                print(f"[{'ok' if ok else 'FAIL'}] {name:36} fired: {', '.join(sorted(fired)) or '-'}"
                      + ("" if ok else f" (expected: {', '.join(sorted(expected)) or '-'})"))
        finally:
            wbindkeys.terminate()
            wbindkeys.wait()

    if failures:
        print(f"\n{failures} of {len(CASES)} cases failed; wbindkeys --debug output is in {LOG}")
        return 1
    print(f"\nAll {len(CASES)} cases passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
