// A lamp dimmed by two buttons, with a heartbeat.
// A press on UP takes the lamp one step brighter (85, 170, 255, where it stays) and prints it;
// OFF puts it out. The heartbeat LED changes at every loop: it reads its own pin back to do so.
const int UP = 2;
const int OFF = 3;
const int LAMP = 9;
const int BEAT = 13;
int level = 0;
int was = 0;

void setup() {
  pinMode(UP, INPUT);
  pinMode(OFF, INPUT);
  pinMode(LAMP, OUTPUT);
  pinMode(BEAT, OUTPUT);
  Serial.begin(9600);
  analogWrite(LAMP, 0);
  digitalWrite(BEAT, LOW);
}

void loop() {
  int up = digitalRead(UP);
  if (digitalRead(OFF) == HIGH) {
    level = 0;
    analogWrite(LAMP, 0);
  } else if (up == 1 && was == 0) {
    level = constrain(level + 85, 0, 255);
    analogWrite(LAMP, level);
    Serial.print("brightness ");
    Serial.println(level);
  }
  was = up;
  digitalWrite(BEAT, !digitalRead(BEAT));
  delay(20);
}
