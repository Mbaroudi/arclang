// A lamp toggled by a button, on an Arduino UNO.
// The button is between pin 2 and GND (pressed = LOW, the pin is pulled up inside the chip).
// The lamp is the LED of the board (pin 13). A press counts once it has held for 50 ms,
// timed with micros(): the finer clock, whose reads are never a millisecond short.
const int BUTTON = 2;
const int LAMP = 13;
int lit = 0;
int pressed = 0;
int counting = 0;
unsigned long since = 0;

void setup() {
  pinMode(BUTTON, INPUT_PULLUP);
  pinMode(LAMP, OUTPUT);
  digitalWrite(LAMP, LOW);
}

void loop() {
  int down = (digitalRead(BUTTON) == LOW) ? 1 : 0;
  if (down != pressed) {
    if (!counting) {
      counting = 1;
      since = micros();
    } else if (micros() - since >= 50000UL) {
      pressed = down;
      counting = 0;
      if (pressed) {
        lit = !lit;
        digitalWrite(LAMP, lit);
      }
    }
  } else {
    counting = 0;
  }
  delay(10);
}
