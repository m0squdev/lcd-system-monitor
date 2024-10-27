# System Monitor with LCD Display

A real-time system monitoring solution that displays CPU, memory, and temperature information on an LCD screen through serial communication. The project consists of an Arduino-based client that controls the LCD display and a Rust-based server that collects system metrics.

## Components

### Hardware Requirements
- Arduino board (or compatible microcontroller)
- I2C LCD Display (16x2 characters)
- USB-to-Serial connection

### Program Components
- Arduino client program (`./client`)
- Rust server application (`./server`)

## Features

- Real-time monitoring of:
  - CPU usage percentage
  - CPU temperature
  - Memory usage percentage
  - Swap usage percentage
- 1-second refresh rate
- 16x2 LCD display output
- Serial communication at 9600 baud
- Configurable serial port selection

## Installation

### Arduino Client Setup

1. Connect the I2C LCD display to your Arduino

2. Install required Arduino libraries:
   - Wire (built-in)
   - LiquidCrystal_I2C

3. Upload `./client/client.ino` to your Arduino board

### Rust Server Setup

1. Connect the Arduino to your computer
2. Navigate to `./server` directory
3. Run the server application:
   ```bash
   cargo run --release
   ```
4. When prompted, enter your serial device name:
   - Press Enter for default (`/dev/ttyUSB0`)
   - Or enter the specific device suffix (e.g., `ACM0` for `/dev/ttyACM0`)

You can also run `cargo run --release <device>` to avoid being prompted and create a connection to the specified device.

And you're done!

## Dependencies

### Server (Rust)
- `serial` (0.4.0): Serial port communication
- `sysinfo` (0.32.0): System information collection

### Client (Arduino)
- `Wire`: I2C communication
- `LiquidCrystal_I2C`: LCD control

## Communication Protocol

The server sends data to the client in the following format:
```
CPU%{usage} Temp {temp};Mem%{mem} Swp%{swap}\n
```
- Lines are separated by a semicolon (`;`)
- First line contains CPU information
- Second line contains memory information
