// A thermostat with hysteresis, written by hand.
const int SENSOR = A0;
const int HEATER = 13;
int mode = 0; // 0: idle, 1: heating

void setup() {
  pinMode(HEATER, OUTPUT);
  digitalWrite(HEATER, LOW);
}

void loop() {
  int t = analogRead(SENSOR);
  if (mode == 0 && t < 400) {
    mode = 1;
    digitalWrite(HEATER, HIGH);
  } else if (mode == 1 && t > 450) {
    mode = 0;
    digitalWrite(HEATER, LOW);
  }
  delay(100);
}
