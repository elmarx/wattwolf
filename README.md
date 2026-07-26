# Wattwolf

Read smart meter data* (Smart Message Language) via infrared port and send sensor-data (via MQTT) to Home Assistant.

*To be precise: "moderne Messeinrichtung", "smart meter" typically includes a gateway.

## Hardware

Tested with:

- Landis Gyr E320
- Hichi IR USB reader

## Configuration

Create a config file (e.g. `config.toml`) based on `config.toml.example`.

Wattwolf looks in the following order for config files:

- CLI argument `--config`
- Environment variable `WATTWOLF_CONFIG`
- Current working directory: `config.toml`
- global `/etc/wattwolf.toml`

### Connection

Connect to the reader directly (e.g. `device = /dev/ttyUSB0`) or via TCP (e.g.
`socket = localhost:2000`).

To connect via TCP (great for debugging/development) setup [ser2net](https://github.com/cminyard/ser2net), e.g.:

```yaml
connection: &con01
  accepter: tcp,localhost,2000
  connector: serialdev,/dev/ttyUSB0,9600n81
  options:
    kickolduser: true
```

### MQTT Configuration

Setup the MQTT connection to publish [discovery messages](https://www.home-assistant.io/integrations/sensor.mqtt/) and of course the actual sensor values.

*username*, *password* and *host* may be overriden via environment-variables:

```shell
export MQTT_USERNAME=your_username
export MQTT_PASSWORD=your_password
export MQTT_HOST=your_host
# or use MQTT_PASSWORD_FILE for increased security, e.g. usage with systemd's `LoadCredential=`
export MQTT_PASSWORD_FILE=/run/secrets/mqtt-password
```

Run a development MQTT server with:

```shell
docker run -d -p 1883:1883 eclipse-mosquitto
```

### Sensors

Sensors need to be specified by their OBIS code. All other settings are used for [home assistant's auto-discovery of the sensor](https://www.home-assistant.io/integrations/sensor.mqtt/).

Currently supported codes:

- 1.8.0
- 2.8.0

## Installation

### NixOS

The flake provides a NixOS module for running wattwolf as a systemd service.

Quick example:

```nix
{
  {
  description = "Example NixOS configuration with wattwolf";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    wattwolf.url = "path:/home/elmar/Projects/wattwolf";
    # In production, use: wattwolf.url = "github:yourusername/wattwolf";

    # Optional: agenix for secret management
    # agenix.url = "github:ryantm/agenix";
  };

  outputs = { self, nixpkgs, wattwolf, ... }: {
    nixosConfigurations.example-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        # Import the wattwolf module
        wattwolf.nixosModules.default

        # Your configuration
        ({ config, pkgs, ... }: {
          services.wattwolf = {
            enable = true;
            mqttPasswordFile = config.age.secrets.wattwolf.path;

            connection.device = "/dev/ttyUSB0";

            mqtt = {
              host = "localhost";
              username = "wattwolf";
            };

            sensors = [
              {
                name = "consumed";
                friendlyName = "Energy Consumed";
                obis = "1.8.0";
                deviceClass = "energy";
                stateClass = "total_increasing";
              }
            ];
          };
        })
      ];
    };
  };
}
```

### Manual Installation

Build and install manually; only libudev required (which systemd typically provides nowadays):

```shell
cargo install wattwolf --git https://github.com/elmarx/wattwolf
```

## Building

### Building for Raspberry Pi 2/Zero 2 and up

- via [cross](https://github.com/cross-rs/cross)
- `cross build --target=aarch64-unknown-linux-gnu`

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.⏎                                                                                                       