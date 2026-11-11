// A lamp that changes when a sensor has pulsed since the last look, however many times.
// The interrupt only raises a flag; the loop reads it and lowers it, with interrupts on.
const int SENSOR = 2;
const int LAMP = 13;
volatile bool pulsed = false;
int lit = 0;

void mark() {
  pulsed = true;
}

void setup() {
  pinMode(SENSOR, INPUT);
  pinMode(LAMP, OUTPUT);
  digitalWrite(LAMP, LOW);
  attachInterrupt(digitalPinToInterrupt(SENSOR), mark, RISING);
}

void loop() {
  if (pulsed) {
    pulsed = false;
    lit = !lit;
    digitalWrite(LAMP, lit);
  }
  delay(10);
}
