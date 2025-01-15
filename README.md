# System Monitor with LCD Display

A real-time system monitoring solution that displays system metrics, battery status, and system information on an LCD screen through serial communication. The project consists of an Arduino-based client that controls the LCD display and a Rust-based server that collects system information.

## Components

### Hardware Requirements
- Arduino board (or compatible microcontroller)
- I2C LCD Display (16x2 characters)
- USB connection to your PC

### Program Components
- Arduino client program (./client)
- Rust server application (./server)

## Features
- Real-time monitoring of system metrics:
  - CPU usage percentage
  - CPU temperature
  - RAM usage percentage
  - Swap usage percentage
- Battery information (if available):
  - Charging status
  - Battery percentage
- Your hostname
- Network upload and download rate
- Now playing music information:
  - Playing/paused
  - Author
  - Song title
- 1-second refresh rate
- Serial communication at 9600 baud
- Automatic serial port detection and reconnection

## Dependencies

### Arduino
- `Wire` (built-in)
- [`LiquidCrystal_I2C`](https://github.com/johnrickman/LiquidCrystal_I2C) (1.1.2)

### Rust
- `battery` (0.7.8)
- `hostname` (0.4.0)
- `mpris` (2.0.1)
- `serial` (0.4.0)
- `serialport` (4.6.0)
- `sysinfo` (0.32.0)

## Installation

### Arduino Client Setup
1. Install `arduino-cli`
2. Navigate to [client](client)
3. Install the required [libraries](#arduino) with `arduino-cli` (user-wide):
   ```bash
   make install-libs
   ```
4. Connect the I2C LCD display to your Arduino. The code is intended to be ran with a 16x2 display.
5. Compile [client/client.ino](client/client.ino) and upload it to your board:
   ```bash
   make port=<port> board=<board>
   ```
   The arguments are directly passed to `arduino-cli`, hence the board parameter syntax is the same you'd expect when using `arduino-cli`. Board parameter example: arduino:avr:nano.

### Rust Server Setup
1. Install `cargo`
2. Navigate to [server](server)
3. Build the program:
   ```bash
   cargo build --release
   ```
## Rust Server Execution
Execute the program:
```bash
cargo run --release
```
The program will try to detect automatically the client device. It will connect to the first device found that has the Arduino vendor ID (0x0403). If none are found, it will connect to the first USB device found. If you want to bind the program to a certain device append it to the end of the command. Nevertheless, I don't recommend doing it on Linux as the Arduino might be recognised as another device when it is connected a second time.

### Server Autostart With Systemd (Optional)
Execute [`server/autostart-systemd.sh`](server/autostart-systemd.sh) to make the program start automatically on login

## Uninstallation

### Arduino Client Dependencies Removal
Uninstall the libraries **if you don't need them** (user-wide):
```bash
make uninstall-libs
```

### Rust Server Autostart Removal
You only need to do this if you did [this step](#server-autostart-with-systemd-optional) earlier.
```bash
systemctl disable --now lcd-system-monitor.service
```
