#!/bin/bash

content=$(cat << EOF
[Unit]
Description=LCD System Monitor

[Service]
WorkingDirectory=$(pwd)
ExecStart=cargo run --release

[Install]
WantedBy=default.target
EOF
)
systemd_dir="$HOME/.config/systemd"
mkdir -p "$systemd_dir"
mkdir -p "$systemd_dir/user"
echo "$content" > "$systemd_dir/user/lcd-system-monitor.service"
systemctl enable lcd-system-monitor.service --user
