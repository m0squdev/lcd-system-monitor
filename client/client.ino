#include <Wire.h>
#include <LiquidCrystal_I2C.h>

#define ADDR 0x27
#define WIDTH 16
#define HEIGHT 2
#define DEG_CHAR 0
#define CHARGING_CHAR 1

const byte degBytes[] = {
  0b00110,
  0b01001,
  0b01001,
  0b00110,
  0b00000,
  0b00000,
  0b00000,
  0b00000
};

const byte chargingBytes[] = {
  B01010,
  B01010,
  B11111,
  B11111,
  B01110,
  B00100,
  B00100,
  B00100
};

LiquidCrystal_I2C lcd(ADDR, WIDTH, HEIGHT);

void write_line(String line)
{
  for (short n = 0; n < WIDTH; n++)
  {
    if (n >= line.length()) lcd.print(" ");
    else
    {
      Serial.print(n);
      Serial.print(line[n]);
      if (line[n] == '^') lcd.write(DEG_CHAR);
      else if (line[n] == '`') lcd.write(CHARGING_CHAR);
      else lcd.print(line[n]);
    }
  }
}

void setup()
{
  lcd.init();
  lcd.backlight();
  lcd.createChar(DEG_CHAR, degBytes);
  lcd.createChar(CHARGING_CHAR, chargingBytes);
  Serial.begin(9600);
  lcd.setCursor(0, 0);
  lcd.print("Listening to");
  lcd.setCursor(0, 1);
  lcd.print("serial...");
}

void loop()
{
  if (Serial.available())
  {
    String data = Serial.readStringUntil('\n');
    Serial.println(data);
    int separatorIndex = data.indexOf(';');

    String line1 = data.substring(0, separatorIndex);
    lcd.setCursor(0, 0);
    write_line(line1);

    String line2 = data.substring(separatorIndex + 1);
    lcd.setCursor(0, 1);
    write_line(line2);

    delay(1000);
  }
}
