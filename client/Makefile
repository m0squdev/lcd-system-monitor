all: compile upload

install-libs:
	arduino-cli lib install "LiquidCrystal I2C"

uninstall-libs:
	arduino-cli lib uninstall "LiquidCrystal I2C"

compile:
	arduino-cli compile -b $(board) .

upload:
	arduino-cli upload -p $(port) --fqbn $(board) .
