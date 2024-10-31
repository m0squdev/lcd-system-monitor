# System Monitor with LCD Display

A real-time system monitoring solution that displays system metrics, battery status, and system information on an LCD screen through serial communication. The project consists of an Arduino-based client that controls the LCD display and a Rust-based server that collects system information.

## Components

### Hardware Requirements
- Arduino board (or compatible microcontroller)
- I2C LCD Display (16x2 characters)
- USB connection to your PC

### Program Components
- Arduino client program (`./client`)
- Rust server application (`./server`)

## Features
- Real-time monitoring of system metrics:
  - CPU usage percentage
  - CPU temperature
  - RAM usage percentage
  - Swap usage percentage
- Battery information:
  - Charging status
  - Battery percentage
- System information:
  - Current username
  - System hostname
- 1-second refresh rate
- Serial communication at 9600 baud
- Automatic serial port detection and reconnection

## Installation

### Arduino Client Setup
1. Connect the I2C LCD display to your Arduino
2. Install required [Arduino libraries](#arduino)
3. Upload [`./client/client.ino`](https://github.com/m0squdev/lcd-system-monitor/blob/main/client/client.ino) to your Arduino board

### Rust Server Setup
1. Connect the Arduino to your computer
2. Navigate to `./server` directory
3. Run the server application:
   ```bash
   cargo run --release
   ```
4. The server should automatically detect the client device. If not, you will be prompted.

### Server Autostart (Optional)
Move [`./lcd-system-monitor.desktop`](https://github.com/m0squdev/lcd-system-monitor/blob/main/lcd-system-monitor.desktop) to `~/.config/autostart` to make the program start automatically with your GNOME session.

## Dependencies

### Arduino
- `Wire` (built-in)
- [`LiquidCrystal_I2C`](https://github.com/johnrickman/LiquidCrystal_I2C) (1.1.2)

### Rust
- `battery` (0.7.8)
- `hostname` (0.4.0)
- `serial` (0.4.0)
- `serialport` (4.6.0)
- `sysinfo` (0.32.0)
- `whoami` (1.5.2)
