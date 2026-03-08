# pi-touchscreen-control

This tool allows automatically dimming the backlight of the Raspberry Pi 7" touchscreen display after inactivity and
controlling it from Home Assistant. It was developed for the older variant of the touchscreen display, but it may also
work for the new one and even similar screens (feel free to test it).

Features (can be used both or separately, depending on the configuration):

* Automatically dim the backlight after a configurable period of inactivity.
* Control the backlight and configure dimming via Home Assistant (MQTT).

The following properties can be controlled from Home Assistant:

* Automatic dimming (switch)
* Activity timeout for dimming in seconds
* Backlight enabled (switch)
* Backlight enabled when inactive (switch)
* Brightness when active (slider)
* Brightness when inactive (slider)
* Current brightness (slider)

## Building

Make sure to have all relevant build tools installed (build-essential, libssl-dev, cmake and the Rust toolchain,
see https://rustup.rs/ for instructions). Then, check out this repository, `cd` to it and run

```sh
cargo build -r
```

## Installing

To install, run (after building):

```sh
sudo ./install.sh
```

To update to a newer version (without reinstaling the config file to keep local changes), run (after building):

```sh
sudo ./update.sh
```

Remember to restart the systemd service after updating (the update script will only stop it automatically).

To uninstall:

```sh
sudo ./uninstall.sh
```

## Configuration

After installing, the config file will be in `/etc/pi-touchscreen-control/config.yaml`. All configuration options are
explained in it and should be quite self-explanatory. You may need to adapt the backlight and input device in some
cases. The timeout and desired full and dimmed brightness can be configured as you like.

To control the backlight via Home Assistant, you need to install the MQTT integration and an MQTT broker like Eclipse
Mosquitto. If you are already using something like Zigbee2MQTT, you should already have this set up. Enable the Home
Assistant feature by setting `use_mqtt: true` in the config file and adapt the server address and (if necessary)
authentication options. If you've set up everything correctly, the new device should automatically appear in Home
Assistant after starting pi-touchscreen-control.

## Running it

The recommended way to run this is via systemd, but of course you can also run the binary itself with the path to the
config file as the only argument. Make sure to run it as root, otherwise the backlight cannot be controlled.

To start it (after installing) and automatically start it after boot:

```sh
sudo systemctl enable --now pi-touchscreen-control.service
```

Just start it:

```sh
sudo systemctl start pi-touchscreen-control.service
```

Stop it:

```sh
sudo systemctl start pi-touchscreen-control.service
```

Disable it to stop running automatically on boot:

```sh
sudo systemctl disable pi-touchscreen-control.service
```

Have fun!