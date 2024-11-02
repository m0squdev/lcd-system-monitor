#!/bin/bash

content=$(cat << EOF
[Unit]
Description=LCD System Monitor

[Service]
#User=<user e.g. root>
WorkingDirectory=$(pwd)/server
ExecStart=cargo run --release
# optional items below
#Restart=always
#RestartSec=3

[Install]
WantedBy=multi-user.target
EOF
)
systemd_dir="$HOME/.config/systemd"
mkdir -p "$systemd_dir"
mkdir -p "$systemd_dir/user"
echo "$content" > "$systemd_dir/user/lcd-system-monitor.service"
systemctl enable lcd-system-monitor.service --user
