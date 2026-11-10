// A lamp that blinks while a switch is on: half a second lit, half a second dark.
const int SWITCH = 4;
const int LAMP = 9;
int lit = 0;
unsigned long changed = 0;

void setup() {
  pinMode(SWITCH, INPUT);
  pinMode(LAMP, OUTPUT);
  digitalWrite(LAMP, LOW);
}

void loop() {
  if (digitalRead(SWITCH) == LOW) {
    if (lit) {
      lit = 0;
      digitalWrite(LAMP, LOW);
    }
    changed = millis();
  } else if (millis() - changed >= 500) {
    lit = !lit;
    digitalWrite(LAMP, lit);
    changed = millis();
  }
  delay(20);
}
