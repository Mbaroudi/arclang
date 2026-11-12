/*
 * The smart door lock on an Arduino Uno, hardware in the loop.
 *
 * The behavior is the C ArcLang generated from the model's state machines
 * (smart_door_lock.c, untouched) and the direct harness of `arclang export
 * -f c` (host/harness.c), the one `arclang sil` drives from stdin. Here the
 * harness reads and writes the serial port: this sketch is avr-libc's stdio
 * over `Serial`, nothing else. The harness's `main` is renamed at build
 * time (-Dmain=arclang_harness_main, see ../flash.sh).
 *
 * The protocol `arclang hil` speaks, at 115200 baud:
 *   board:  # arclang hil ready smart_door_lock arduino-uno
 *   host:   the number of steps, then one line of inputs per step
 *   board:  one CSV row of the outputs per step, then
 *           # arclang hil done 0
 * and again from the top for the next case. Pin 13 lights when the lock is
 * unlocked, as a sign of life: the harness itself writes no pin.
 */
#include <stdio.h>

extern "C" int arclang_harness_main(void);

static int uart_putchar(char c, FILE *stream) {
  (void)stream;
  if (c == '\n') Serial.write('\r');
  Serial.write(c);
  return 0;
}

static int uart_getchar(FILE *stream) {
  (void)stream;
  while (!Serial.available()) {
  }
  return Serial.read();
}

static FILE serial_stdio;

void setup() {
  Serial.begin(115200);
  pinMode(LED_BUILTIN, OUTPUT);
  fdev_setup_stream(&serial_stdio, uart_putchar, uart_getchar, _FDEV_SETUP_RW);
  stdin = &serial_stdio;
  stdout = &serial_stdio;
}

void loop() {
  Serial.println("# arclang hil ready smart_door_lock arduino-uno");
  digitalWrite(LED_BUILTIN, LOW);
  int code = arclang_harness_main();
  Serial.print("# arclang hil done ");
  Serial.println(code);
}
