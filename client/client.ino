#include <Wire.h>
#include <LiquidCrystal_I2C.h>

#define ADDR 0x27
#define WIDTH 16
#define HEIGHT 2
#define DEG_CHAR 0
#define CHARGING_CHAR 1
#define DISCHARGING_CHAR 2
#define PLAYING_CHAR 3
#define PAUSED_CHAR 4
#define ELLIPSIS_CHAR 5
#define ARROW_UP_CHAR 6
#define ARROW_DOWN_CHAR 7

const byte degBytes[] =
{
    B00110,
    B01001,
    B01001,
    B00110,
    B00000,
    B00000,
    B00000,
    B00000
};
const byte chargingBytes[] =
{
    B01010,
    B01010,
    B11111,
    B11111,
    B01110,
    B00100,
    B00100,
    B00100
};
const byte dischargingBytes[] =
{
    B01110,
    B11111,
    B10001,
    B10001,
    B10001,
    B10001,
    B10001,
    B11111
};
const byte playingBytes[] =
{
    B00000,
    B11011,
    B11011,
    B11011,
    B11011,
    B11011,
    B11011,
    B00000
};
const byte pausedBytes[] =
{
    B10000,
    B11000,
    B11100,
    B11110,
    B11110,
    B11100,
    B11000,
    B10000
};
const byte ellipsisBytes[] =
{
    B00000,
    B00000,
    B00000,
    B00000,
    B00000,
    B00000,
    B10101,
    B00000
};
const byte arrowUpBytes[] =
{
    B00100,
    B01110,
    B11111,
    B00100,
    B00100,
    B00100,
    B00100,
    B11111
};
const byte arrowDownBytes[] =
{
    B00100,
    B00100,
    B00100,
    B00100,
    B11111,
    B01110,
    B00100,
    B11111
};

LiquidCrystal_I2C lcd(ADDR, WIDTH, HEIGHT);

void write_line(String line)
{
    for (short n = 0; n < WIDTH; n++)
    {
        if (n >= line.length()) lcd.print(" ");
        else if (n == WIDTH - 1 && n + 1 < line.length())
        {
            lcd.write(ELLIPSIS_CHAR);
            return;
        }
        else
        {
            switch (line[n])
            {
            case '^':
                lcd.write(DEG_CHAR);
                break;
            case '`':
                lcd.write(CHARGING_CHAR);
                break;
            case '&':
                lcd.write(DISCHARGING_CHAR);
                break;
            case '#':
                lcd.write(PLAYING_CHAR);
                break;
            case '$':
                lcd.write(PAUSED_CHAR);
                break;
            case '[':
                lcd.write(ARROW_UP_CHAR);
                break;
            case ']':
                lcd.write(ARROW_DOWN_CHAR);
                break;
            default:
                lcd.print(line[n]);
            }
        }
    }
}

void setup()
{
    lcd.init();
    lcd.backlight();
    lcd.createChar(DEG_CHAR, degBytes);
    lcd.createChar(CHARGING_CHAR, chargingBytes);
    lcd.createChar(DISCHARGING_CHAR, dischargingBytes);
    lcd.createChar(PLAYING_CHAR, playingBytes);
    lcd.createChar(PAUSED_CHAR, pausedBytes);
    lcd.createChar(ELLIPSIS_CHAR, ellipsisBytes);
    lcd.createChar(ARROW_UP_CHAR, arrowUpBytes);
    lcd.createChar(ARROW_DOWN_CHAR, arrowDownBytes);
    Serial.begin(9600);
    lcd.setCursor(0, 0);
    lcd.print("Listening");
    lcd.setCursor(0, 1);
    lcd.print("to serial");
    lcd.write(ELLIPSIS_CHAR);
}

void loop()
{
    if (Serial.available())
    {
        String data = Serial.readStringUntil('\n');
        int separatorIndex = data.indexOf(';');
        String line1 = data.substring(0, separatorIndex);
        lcd.setCursor(0, 0);
        write_line(line1);
        String line2 = data.substring(separatorIndex + 1);
        lcd.setCursor(0, 1);
        write_line(line2);
        /* It's better for this delay to be smaller than the delay of the server
           since you have to consider the time it takes to print to the LCD */
        delay(900);
    }
}
