#!/bin/bash

content=$(cat << EOF
[Desktop Entry]
Exec=dbus-run-session -- /bin/bash -c "cd $(pwd) && /usr/bin/cargo run --release"
Name=LCD System Monitor
Type=Application
StartupNotify=true
Terminal=false
Hidden=false
X-GNOME-Autostart-enabled=true
EOF
)
echo "$content" > "$HOME/.config/autostart/lcd-system-monitor.desktop"