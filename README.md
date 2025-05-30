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
- CPU
  - Usage
  - Percentage
- RAM usage
- Swap usage
- GPU (Nvidia only) *
  - Usage
  - Temperature
  - Memory usage
- Battery information *
  - Charging status
  - Battery percentage
- Network upload and download rate
- Now playing music information *
  - Playing/paused
  - Author
  - Song title

(*) Information will only be shown if available

## Dependencies

### Arduino
- `Wire` (built-in)
- [`LiquidCrystal_I2C`](https://github.com/johnrickman/LiquidCrystal_I2C) (1.1.2)

### Rust
See [`server/Cargo.toml`](server/Cargo.toml)

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
   The arguments are directly passed to `arduino-cli`, hence the `board` parameter syntax is the same you'd expect when using `arduino-cli`. Example use: `make port=/dev/ttyUSB0 board=arduino:avr:nano`.

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
You only need to do this if you followed [this step](#server-autostart-with-systemd-optional) before.
```bash
systemctl disable --now lcd-system-monitor.service
```
