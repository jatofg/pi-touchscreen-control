#!/bin/bash
set -eu
cp target/release/pi-touchscreen-control /usr/local/bin/
chown root:root /usr/local/bin/pi-touchscreen-control
chmod 0755 /usr/local/bin/pi-touchscreen-control
cp pi-touchscreen-control.service /etc/systemd/system/
systemctl daemon-reload