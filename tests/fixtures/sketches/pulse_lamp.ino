// A lamp that changes at each pulse of a sensor, however short the pulse.
// The pulses are counted by an interrupt: a loop that read the pin would miss one shorter than itself.
const int SENSOR = 2;
const int LAMP = 13;
volatile int pulses = 0;
int lit = 0;

void count() {
  pulses++;
}

void setup() {
  pinMode(SENSOR, INPUT);
  pinMode(LAMP, OUTPUT);
  digitalWrite(LAMP, LOW);
  attachInterrupt(digitalPinToInterrupt(SENSOR), count, RISING);
}

void loop() {
  noInterrupts();
  int seen = pulses;
  pulses = 0;
  interrupts();
  if (seen % 2 == 1) {
    lit = !lit;
    digitalWrite(LAMP, lit);
  }
  delay(10);
}
