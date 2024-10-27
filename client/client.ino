#include <Wire.h>
#include <LiquidCrystal_I2C.h>

LiquidCrystal_I2C lcd(0x27, 16, 2);

void setup() {
  lcd.init();
  lcd.backlight();
  Serial.begin(9600);
  lcd.setCursor(0, 0);
  lcd.print("Listening to");
  lcd.setCursor(0, 1);
  lcd.print("serial...");
}

void loop() {
  if (Serial.available()) {
    String data = Serial.readStringUntil('\n');
    Serial.println(data);
    int separatorIndex = data.indexOf(';');
    String line1 = data.substring(0, separatorIndex);
    String line2 = data.substring(separatorIndex + 1);

    lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print(line1);
    lcd.setCursor(0, 1);
    lcd.print(line2);

    delay(1000);
  }
}
