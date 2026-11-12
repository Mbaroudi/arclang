"""The environment of the smart lock: a bench that replays the model's
scenario "Resident comes home" (UC-1) against the lock running free on an
Arduino Uno (uno_live/) and measures its response times.

The same file runs on a PC (CPython, the Uno on a USB serial port) and on
the UDOO KEY's ESP32 (MicroPython, the Uno wired to UART2: GPIO13 TX ->
Uno RX pin 0, Uno TX pin 1 -> GPIO26 RX through a 5 V to 3.3 V divider,
GND to GND). Two times per step: as the bench sees it (round trip, the
line included) and as the board clocks it (its own microseconds between
the acknowledgement of the command and the state it reports).

    python3 bench.py /dev/cu.usbmodem21101        # on a PC
    >>> import bench; bench.main()                 # on the UDOO KEY
"""
import sys

MICROPYTHON = sys.implementation.name == 'micropython'

if MICROPYTHON:
    import time

    def now_ms():
        return time.ticks_ms()

    def since_ms(start):
        return time.ticks_diff(time.ticks_ms(), start)

    class Port:
        """UART2 of the ESP32 on the UDOO KEY's UEXT header: TX on GPIO13
        (UEXT pin 3, 1 kOhm to the Uno's pin 2), RX on GPIO14 (UEXT pin 9,
        from the Uno's pin 9 through a 10k/20k divider), 57600 baud: the
        software serial of uno_live/."""

        def __init__(self, tx=13, rx=14, baud=57600):
            from machine import UART
            self.uart = UART(2, baudrate=baud, tx=tx, rx=rx, timeout=0)

        def write(self, data):
            self.uart.write(data)

        def read(self):
            return (self.uart.read(256) if self.uart.any() else None) or b''

        def close(self):
            self.uart.deinit()
else:
    import os, termios, time

    def now_ms():
        return time.monotonic() * 1000.0

    def since_ms(start):
        return time.monotonic() * 1000.0 - start

    class Port:
        """A USB serial port, raw at 115200 baud; opening it resets the Uno."""

        def __init__(self, path, baud=termios.B115200):
            self.fd = os.open(path, os.O_RDWR | os.O_NOCTTY | os.O_NONBLOCK)
            a = termios.tcgetattr(self.fd)
            a[0] = 0; a[1] = 0; a[2] = termios.CS8 | termios.CREAD | termios.CLOCAL; a[3] = 0
            a[4] = baud; a[5] = baud
            termios.tcsetattr(self.fd, termios.TCSANOW, a)

        def write(self, data):
            os.write(self.fd, data)

        def read(self):
            try:
                return os.read(self.fd, 256)
            except BlockingIOError:
                return b''

        def close(self):
            os.close(self.fd)


class Lines:
    """The lines the board prints, read with a timeout."""

    def __init__(self, port):
        self.port = port
        self.buffer = b''

    def readline(self, timeout_ms):
        start = now_ms()
        while True:
            if b'\n' in self.buffer:
                line, self.buffer = self.buffer.split(b'\n', 1)
                return line.decode('ascii', 'replace').strip()
            data = self.port.read()
            if data:
                self.buffer += data
            elif since_ms(start) > timeout_ms:
                return None

    def drain(self, timeout_ms):
        lines = []
        while True:
            line = self.readline(timeout_ms)
            if line is None:
                return lines
            lines.append(line)


def send(port, command):
    port.write((command + '\n').encode())


def expect(lines, command, machine, state, budget_ms=None, timeout_ms=400):
    """Send a command, wait for the state; the two response times, or None."""
    start = now_ms()
    send(lines.port, command)
    ack = None
    hit = None
    deadline = start + timeout_ms
    while since_ms(start) < timeout_ms:
        line = lines.readline(50)
        if line is None:
            continue
        words = line.split()
        if words and words[0] == 'A' and len(words) > 1:
            ack = int(words[1])
        if len(words) >= 4 and words[0] == 'S' and words[1] == machine and ' '.join(words[2:-1]) == state:
            hit = (since_ms(start), int(words[-1]))
            break
    label = '%-26s -> %s %s' % (command, machine, state)
    if hit is None:
        print('  %s: NOT REACHED' % label)
        return None
    bench_ms, target_us = hit
    target_ms = (target_us - ack) / 1000.0 if ack is not None else -1.0
    verdict = ''
    if budget_ms is not None:
        verdict = ('  OK' if 0 <= target_ms <= budget_ms else '  FAIL') + ' (budget %d ms)' % budget_ms
    print('  %s: bench %6.1f ms, target %5.2f ms%s' % (label, bench_ms, target_ms, verdict))
    return (bench_ms, target_ms)


def run(port):
    """The scenario, and the way back; True when every step is as the model says."""
    lines = Lines(port)
    print('waiting for the lock (banner, states):')
    for line in lines.drain(2500):
        if line.startswith('#'):
            print(' ', line)
    # The first byte out of a UART just opened can be garbled (its line
    # settles): an empty line first, then the reset, checked by the states.
    send(port, '')
    lines.drain(200)
    for _ in range(3):
        send(port, 'R')
        if any(l.startswith('A') for l in lines.drain(300)):
            break
    send(port, '?')
    for line in lines.drain(300):
        if line.startswith('I'):
            print(' ', line)
    print('scenario "Resident comes home" (UC-1), then the way back:')
    ok = True
    ok = expect(lines, 'C commissioned 1', 'lifecycle', 'Service') is not None and ok
    send(port, 'C credential_accepted 1'); lines.drain(200)
    ok = expect(lines, 'E SF-1', 'modes', 'Unlocked', budget_ms=25) is not None and ok
    send(port, 'C door_closed 1'); lines.drain(200)
    ok = expect(lines, 'E SF-2', 'modes', 'Locked', budget_ms=25) is not None and ok
    print('negative check: SF-1 without a credential must not unlock')
    send(port, 'C credential_accepted 0'); lines.drain(200)
    if expect(lines, 'E SF-1', 'modes', 'Unlocked') is not None:
        ok = False
        print('  unlocked without a credential: WRONG')
    else:
        print('  stays Locked: correct')
    print('RESULT', 'PASS' if ok else 'FAIL')
    return ok


def main(path=None):
    port = Port() if MICROPYTHON else Port(path or sys.argv[1])
    try:
        return run(port)
    finally:
        port.close()


if __name__ == '__main__' and not MICROPYTHON:
    sys.exit(0 if main() else 1)
