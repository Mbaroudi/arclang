// A door lock: a press opens it for three seconds; a second press while open closes it at once.
// An alarm lamp is lit while the door sensor says open and the lock is closed.
const int BUTTON = 2;
const int SENSOR = 3;
const int LOCK = 8;
const int ALARM = 9;
int open_ = 0;
int held = 0;
unsigned long opened = 0;

void setup() {
  pinMode(BUTTON, INPUT);
  pinMode(SENSOR, INPUT);
  pinMode(LOCK, OUTPUT);
  pinMode(ALARM, OUTPUT);
  digitalWrite(LOCK, LOW);
  digitalWrite(ALARM, LOW);
}

void loop() {
  int pressed = digitalRead(BUTTON);
  int ajar = digitalRead(SENSOR);
  if (pressed && !held) {
    if (!open_) {
      open_ = 1;
      opened = millis();
      digitalWrite(LOCK, HIGH);
    } else {
      open_ = 0;
      digitalWrite(LOCK, LOW);
    }
  } else if (open_ && millis() - opened >= 3000) {
    open_ = 0;
    digitalWrite(LOCK, LOW);
  }
  held = pressed;
  digitalWrite(ALARM, (ajar && !open_) ? HIGH : LOW);
  delay(50);
}
