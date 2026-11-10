// A button debounced: its level counts once it has held for 50 ms.
const int BUTTON = 2;
const int LED = 13;
int stable = 0;
int counting = 0;
unsigned long since = 0;

void setup() {
  pinMode(BUTTON, INPUT);
  pinMode(LED, OUTPUT);
  digitalWrite(LED, LOW);
}

void loop() {
  int raw = digitalRead(BUTTON);
  if (raw != stable) {
    if (!counting) {
      counting = 1;
      since = millis();
    } else if (millis() - since >= 50) {
      stable = raw;
      counting = 0;
      digitalWrite(LED, stable);
    }
  } else {
    counting = 0;
  }
  delay(10);
}
