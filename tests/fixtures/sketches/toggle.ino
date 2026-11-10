// A lamp toggled by each press of a button.
const int BUTTON = 2;
const int LED = 13;
int lit = 0;
int was = 0;

void setup() {
  pinMode(BUTTON, INPUT_PULLUP);
  pinMode(LED, OUTPUT);
  digitalWrite(LED, LOW);
}

void loop() {
  int now = digitalRead(BUTTON);
  if (now == 1 && was == 0) {
    lit = !lit;
    digitalWrite(LED, lit);
  }
  was = now;
  delay(10);
}
