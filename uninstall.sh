#!/bin/bash
rm /usr/local/bin/pi-touchscreen-control
rm -r /etc/pi-touchscreen-control
rm /etc/systemd/system/pi-touchscreen-control.service
systemctl daemon-reload