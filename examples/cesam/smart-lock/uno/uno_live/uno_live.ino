/*
 * The smart door lock of examples/cesam/smart-lock running free on an
 * Arduino Uno: the lock as the bench (../bench.py) sees it.
 *
 * The behavior is the C ArcLang generated from the model's three state
 * machines (smart_door_lock.c, untouched). This sketch is the target glue:
 * it steps the machines every SMART_DOOR_LOCK_PERIOD_US (10 ms), takes the
 * conditions and events from a serial line, and reports every state
 * change with a microsecond timestamp, so that a bench can replay the
 * model's scenarios and measure the response time.
 *
 * Two lines carry the same protocol: the USB serial port (115200 baud, a
 * PC), and a software serial on pins 2 (RX) and 9 (TX) at 57600 baud for
 * a bench board wired to them, the UDOO KEY of the README: its GPIO13
 * drives pin 2 through 1 kOhm, pin 9 reaches its GPIO14 through a
 * 10k/20k divider (the Uno is 5 V, the ESP32 is 3.3 V). Pin 3 stays an
 * input: the same bench holds it through 1 kOhm.
 *
 * Line protocol, one command per line:
 *   C <condition> <0|1>   set a condition (see names below)
 *   E <event>             post an event to "Lock modes": SF-1 or SF-2
 *   ?                     print the active states
 *   R                     reset (init) the machines
 * Reports, on both lines:
 *   S <machine> <state> <micros>   a state became active (after the step)
 *   A <micros>                     acknowledgement of a command, when it was applied
 *   I <name>=<value> ...           answer to ?
 * Conditions: commissioned battery_low batteries_replaced
 *             credential_accepted door_closed
 *             pairing_button_pressed phone_paired
 * LED 13 lights when the lock is Unlocked.
 */
#include <SoftwareSerial.h>
#include "smart_door_lock.h"

static SoftwareSerial bench(2, 9);  /* RX from the bench board, TX to it */

static smart_door_lock_state st;
static smart_door_lock_inputs in;
static smart_door_lock_outputs out, prev;
static unsigned long next_step_us;
static char line_usb[48], line_bench[48];
static uint8_t len_usb, len_bench;
static char text[96];

static void emit(const char *s) {
  Serial.println(s);
  bench.println(s);
}

static void report(const char *machine, const char *state, unsigned long t) {
  snprintf(text, sizeof text, "S %s %s %lu", machine, state, t);
  emit(text);
}

static void diff_states(unsigned long t) {
  if (out.lock_lifecycle.st_installation != prev.lock_lifecycle.st_installation && out.lock_lifecycle.st_installation) report("lifecycle", "Installation", t);
  if (out.lock_lifecycle.st_service != prev.lock_lifecycle.st_service && out.lock_lifecycle.st_service) report("lifecycle", "Service", t);
  if (out.lock_lifecycle.st_maintenance != prev.lock_lifecycle.st_maintenance && out.lock_lifecycle.st_maintenance) report("lifecycle", "Maintenance", t);
  if (out.lock_modes.st_locked != prev.lock_modes.st_locked && out.lock_modes.st_locked) report("modes", "Locked", t);
  if (out.lock_modes.st_unlocked != prev.lock_modes.st_unlocked && out.lock_modes.st_unlocked) report("modes", "Unlocked", t);
  if (out.lock_modes.st_pairing != prev.lock_modes.st_pairing && out.lock_modes.st_pairing) report("modes", "Pairing", t);
  if (out.lock_configurations.st_nominal != prev.lock_configurations.st_nominal && out.lock_configurations.st_nominal) report("configurations", "Nominal", t);
  if (out.lock_configurations.st_radio_only != prev.lock_configurations.st_radio_only && out.lock_configurations.st_radio_only) report("configurations", "Radio only", t);
  prev = out;
}

static bool *condition(const char *name) {
  if (!strcmp(name, "commissioned")) return &in.lock_lifecycle.cond_commissioned;
  if (!strcmp(name, "battery_low")) return &in.lock_lifecycle.cond_battery_low;
  if (!strcmp(name, "batteries_replaced")) return &in.lock_lifecycle.cond_batteries_replaced;
  if (!strcmp(name, "credential_accepted")) return &in.lock_modes.cond_credential_accepted;
  if (!strcmp(name, "door_closed")) return &in.lock_modes.cond_door_closed;
  if (!strcmp(name, "pairing_button_pressed")) return &in.lock_configurations.cond_pairing_button_pressed;
  if (!strcmp(name, "phone_paired")) return &in.lock_configurations.cond_phone_paired;
  return NULL;
}

static void ack(unsigned long t) {
  snprintf(text, sizeof text, "A %lu", t);
  emit(text);
}

static void command(char *l) {
  unsigned long t = micros();
  if (l[0] == 'C') {
    char *name = strtok(l + 1, " "); char *val = strtok(NULL, " ");
    bool *c = name ? condition(name) : NULL;
    if (c && val) { *c = (val[0] == '1'); ack(t); }
    else emit("? unknown condition");
  } else if (l[0] == 'E') {
    char *name = strtok(l + 1, " ");
    uint16_t ev; bool ok = true;
    if (name && !strcmp(name, "SF-1")) ev = SMART_DOOR_LOCK_LOCK_MODES_EV_SF_1;
    else if (name && !strcmp(name, "SF-2")) ev = SMART_DOOR_LOCK_LOCK_MODES_EV_SF_2;
    else ok = false;
    if (ok && smart_door_lock_lock_modes_post(&in.lock_modes, ev)) ack(t);
    else emit("? unknown event or queue full");
  } else if (l[0] == '?') {
    snprintf(text, sizeof text, "I installation=%d service=%d maintenance=%d locked=%d unlocked=%d pairing=%d nominal=%d radio_only=%d",
             out.lock_lifecycle.st_installation, out.lock_lifecycle.st_service, out.lock_lifecycle.st_maintenance,
             out.lock_modes.st_locked, out.lock_modes.st_unlocked, out.lock_modes.st_pairing,
             out.lock_configurations.st_nominal, out.lock_configurations.st_radio_only);
    emit(text);
  } else if (l[0] == 'R') {
    smart_door_lock_init(&st); memset(&in, 0, sizeof in); memset(&prev, 0, sizeof prev);
    ack(t);
  }
}

/* Gather a line from a stream, one character at a time; run it when complete. */
static void poll(Stream &s, char *line, uint8_t &len, uint8_t size) {
  while (s.available()) {
    char c = s.read();
    if (c == '\n' || c == '\r') { if (len) { line[len] = 0; command(line); len = 0; } }
    else if (len < size - 1) line[len++] = c;
  }
}

void setup() {
  Serial.begin(115200);
  bench.begin(57600);
  pinMode(LED_BUILTIN, OUTPUT);
  pinMode(3, INPUT);  /* held by the bench board through 1 kOhm: never an output */
  smart_door_lock_init(&st);
  memset(&in, 0, sizeof in); memset(&prev, 0, sizeof prev);
  next_step_us = micros();
  emit("# ArcLang smart door lock on Arduino Uno, step 10 ms");
}

void loop() {
  poll(Serial, line_usb, len_usb, sizeof line_usb);
  poll(bench, line_bench, len_bench, sizeof line_bench);
  unsigned long now = micros();
  if ((long)(now - next_step_us) >= 0) {
    next_step_us += SMART_DOOR_LOCK_PERIOD_US;
    smart_door_lock_step(&st, &in, &out);
    in.lock_modes.queued = 0;            /* the events of this step were consumed */
    digitalWrite(LED_BUILTIN, out.lock_modes.st_unlocked ? HIGH : LOW);
    diff_states(micros());
  }
}
