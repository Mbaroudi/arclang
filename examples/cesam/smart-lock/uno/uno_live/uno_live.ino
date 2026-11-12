/*
 * The smart door lock of examples/cesam/smart-lock running free on an
 * Arduino Uno: the lock as the bench (../bench.py) sees it.
 *
 * The behavior is the C ArcLang generated from the model's three state
 * machines (smart_door_lock.c, untouched). This sketch is the target glue:
 * it steps the machines every SMART_DOOR_LOCK_PERIOD_US (10 ms), takes the
 * conditions and events from the serial line, and reports every state
 * change with a microsecond timestamp, so that a bench (the UDOO KEY, or a
 * PC) can replay the model's scenarios and measure the response time.
 *
 * Line protocol, 115200 baud, one command per line:
 *   C <condition> <0|1>   set a condition (see names below)
 *   E <event>             post an event to "Lock modes": SF-1 or SF-2
 *   ?                     print the active states
 *   R                     reset (init) the machines
 * Reports:
 *   S <machine> <state> <micros>   a state became active (after the step)
 *   A <micros>                     acknowledgement of a command, when it was applied
 *   I <name>=<value> ...           answer to ?
 * Conditions: commissioned battery_low batteries_replaced
 *             credential_accepted door_closed
 *             pairing_button_pressed phone_paired
 * LED 13 lights when the lock is Unlocked.
 */
#include "smart_door_lock.h"

static smart_door_lock_state st;
static smart_door_lock_inputs in;
static smart_door_lock_outputs out, prev;
static unsigned long next_step_us;
static char line[48];
static uint8_t len;

static void report(const char *machine, const char *state, unsigned long t) {
  Serial.print('S'); Serial.print(' '); Serial.print(machine); Serial.print(' ');
  Serial.print(state); Serial.print(' '); Serial.println(t);
}

static void diff_states(unsigned long t, bool force) {
  if (force || out.lock_lifecycle.st_installation != prev.lock_lifecycle.st_installation) if (out.lock_lifecycle.st_installation) report("lifecycle", "Installation", t);
  if (force || out.lock_lifecycle.st_service != prev.lock_lifecycle.st_service) if (out.lock_lifecycle.st_service) report("lifecycle", "Service", t);
  if (force || out.lock_lifecycle.st_maintenance != prev.lock_lifecycle.st_maintenance) if (out.lock_lifecycle.st_maintenance) report("lifecycle", "Maintenance", t);
  if (force || out.lock_modes.st_locked != prev.lock_modes.st_locked) if (out.lock_modes.st_locked) report("modes", "Locked", t);
  if (force || out.lock_modes.st_unlocked != prev.lock_modes.st_unlocked) if (out.lock_modes.st_unlocked) report("modes", "Unlocked", t);
  if (force || out.lock_modes.st_pairing != prev.lock_modes.st_pairing) if (out.lock_modes.st_pairing) report("modes", "Pairing", t);
  if (force || out.lock_configurations.st_nominal != prev.lock_configurations.st_nominal) if (out.lock_configurations.st_nominal) report("configurations", "Nominal", t);
  if (force || out.lock_configurations.st_radio_only != prev.lock_configurations.st_radio_only) if (out.lock_configurations.st_radio_only) report("configurations", "Radio only", t);
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

static void command(char *l) {
  unsigned long t = micros();
  if (l[0] == 'C') {
    char *name = strtok(l + 1, " "); char *val = strtok(NULL, " ");
    bool *c = name ? condition(name) : NULL;
    if (c && val) { *c = (val[0] == '1'); Serial.print('A'); Serial.print(' '); Serial.println(t); }
    else Serial.println("? unknown condition");
  } else if (l[0] == 'E') {
    char *name = strtok(l + 1, " ");
    uint16_t ev; bool ok = true;
    if (name && !strcmp(name, "SF-1")) ev = SMART_DOOR_LOCK_LOCK_MODES_EV_SF_1;
    else if (name && !strcmp(name, "SF-2")) ev = SMART_DOOR_LOCK_LOCK_MODES_EV_SF_2;
    else ok = false;
    if (ok && smart_door_lock_lock_modes_post(&in.lock_modes, ev)) { Serial.print('A'); Serial.print(' '); Serial.println(t); }
    else Serial.println("? unknown event or queue full");
  } else if (l[0] == '?') {
    Serial.print("I installation="); Serial.print(out.lock_lifecycle.st_installation);
    Serial.print(" service="); Serial.print(out.lock_lifecycle.st_service);
    Serial.print(" maintenance="); Serial.print(out.lock_lifecycle.st_maintenance);
    Serial.print(" locked="); Serial.print(out.lock_modes.st_locked);
    Serial.print(" unlocked="); Serial.print(out.lock_modes.st_unlocked);
    Serial.print(" pairing="); Serial.print(out.lock_modes.st_pairing);
    Serial.print(" nominal="); Serial.print(out.lock_configurations.st_nominal);
    Serial.print(" radio_only="); Serial.println(out.lock_configurations.st_radio_only);
  } else if (l[0] == 'R') {
    smart_door_lock_init(&st); memset(&in, 0, sizeof in); memset(&prev, 0, sizeof prev);
    Serial.print('A'); Serial.print(' '); Serial.println(t);
  }
}

void setup() {
  Serial.begin(115200);
  pinMode(LED_BUILTIN, OUTPUT);
  smart_door_lock_init(&st);
  memset(&in, 0, sizeof in); memset(&prev, 0, sizeof prev);
  next_step_us = micros();
  Serial.println("# ArcLang smart door lock on Arduino Uno, step 10 ms");
}

void loop() {
  while (Serial.available()) {
    char c = Serial.read();
    if (c == '\n' || c == '\r') { if (len) { line[len] = 0; command(line); len = 0; } }
    else if (len < sizeof line - 1) line[len++] = c;
  }
  unsigned long now = micros();
  if ((long)(now - next_step_us) >= 0) {
    next_step_us += SMART_DOOR_LOCK_PERIOD_US;
    smart_door_lock_step(&st, &in, &out);
    in.lock_modes.queued = 0;            /* the events of this step were consumed */
    digitalWrite(LED_BUILTIN, out.lock_modes.st_unlocked ? HIGH : LOW);
    diff_states(micros(), false);
  }
}
