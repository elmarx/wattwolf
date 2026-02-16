# Wattwolf

Read smart meter data and send it (via MQTT) to home assistant.

## Configuration

Create a `config.toml` based on `config.toml.example`.

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

## Building for Raspberry Pi 2/Zero 2 and up

- via [cross](https://github.com/cross-rs/cross)
- `cross build --target=aarch64-unknown-linux-gnu`