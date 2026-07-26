{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.wattwolf;

  tomlFormat = pkgs.formats.toml { };

  # drop null attributes, since TOML has no concept of null
  filterNulls = filterAttrs (_: v: v != null);

  connectionSettings =
    if cfg.connection.device != null then
      { device = cfg.connection.device; }
    else
      { socket = cfg.connection.socket; };

  mqttSettings = filterNulls {
    host = cfg.mqtt.host;
    port = cfg.mqtt.port;
    username = cfg.mqtt.username;
    topic_prefix = cfg.mqtt.topicPrefix;
    availability_topic = cfg.mqtt.availabilityTopic;
    min_publish_interval = cfg.mqtt.minPublishInterval;
    max_publish_interval = cfg.mqtt.maxPublishInterval;
  };

  sensorSettings = map (
    sensor:
    filterNulls {
      name = sensor.name;
      friendly_name = sensor.friendlyName;
      obis = sensor.obis;
      device_class = sensor.deviceClass;
      state_class = sensor.stateClass;
    }
  ) cfg.sensors;

  settings = {
    connection = connectionSettings;
    mqtt = mqttSettings;
    sensors = sensorSettings;
  };

  configFile = tomlFormat.generate "wattwolf-config.toml" settings;
in
{
  options.services.wattwolf = {
    enable = mkEnableOption "wattwolf smart meter reader service";

    package = mkOption {
      type = types.package;
      default = pkgs.wattwolf;
      defaultText = literalExpression "pkgs.wattwolf";
      description = "The wattwolf package to use.";
    };

    connection = mkOption {
      description = ''
        How to connect to the smart meter: either a serial `device` or a
        `socket` (e.g. a ser2net TCP proxy). Exactly one of the two must be
        set.
      '';
      default = { };
      type = types.submodule {
        options = {
          device = mkOption {
            type = types.nullOr types.str;
            default = null;
            example = "/dev/ttyUSB0";
            description = "Path to the serial device the smart meter is connected to.";
          };

          socket = mkOption {
            type = types.nullOr types.str;
            default = "localhost:2000";
            example = "smart-meter-gateway:2000";
            description = "host:port of a TCP endpoint (e.g. ser2net) to connect to.";
          };
        };
      };
    };

    mqtt = mkOption {
      description = "MQTT broker connection settings.";
      type = types.submodule {
        options = {
          host = mkOption {
            type = types.str;
            description = "MQTT broker host name or IP address.";
          };

          port = mkOption {
            type = types.port;
            default = 1883;
            description = "MQTT broker port.";
          };

          username = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "MQTT username.";
          };

          topicPrefix = mkOption {
            type = types.str;
            default = "wattwolf";
            description = "Prefix for published MQTT topics.";
          };

          availabilityTopic = mkOption {
            type = types.str;
            default = "wattwolf/availability";
            description = "MQTT topic used to publish availability (online/offline).";
          };

          minPublishInterval = mkOption {
            type = types.ints.unsigned;
            default = 10;
            description = "Minimum interval in seconds between published values, i.e. do not send updates more often than this.";
          };

          maxPublishInterval = mkOption {
            type = types.ints.unsigned;
            default = 300;
            description = "Maximum interval in seconds between published values, i.e. publish at least this often.";
          };
        };
      };
    };

    sensors = mkOption {
      description = "Sensors to read from the smart meter and publish via MQTT (Home Assistant discovery format).";
      default = [ ];
      type = types.listOf (
        types.submodule {
          options = {
            name = mkOption {
              type = types.str;
              example = "consumed";
              description = "Internal sensor name.";
            };

            friendlyName = mkOption {
              type = types.str;
              example = "Energy Consumed";
              description = "Human-readable sensor name.";
            };

            obis = mkOption {
              type = types.str;
              example = "1.8.0";
              description = "OBIS code identifying the value to read from the smart meter.";
            };

            deviceClass = mkOption {
              type = types.nullOr (types.enum [ "power" "energy" ]);
              default = null;
              description = "Home Assistant device class for this sensor.";
            };

            stateClass = mkOption {
              type = types.nullOr types.str;
              default = null;
              example = "total_increasing";
              description = "Home Assistant state class for this sensor.";
            };
          };
        }
      );
    };

    mqttPasswordFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        Path to a file containing the MQTT password.
        It is loaded via systemd's LoadCredential= mechanism and passed to
        wattwolf as a file path (MQTT_PASSWORD_FILE), per the credentials
        recommendation in systemd.exec(5): secrets are kept out of the
        service's environment and are only exposed as a file readable by the
        service itself.
        This is useful for integration with agenix or other secret management tools.
      '';
      example = "/run/agenix/wattwolf-mqtt-password";
    };

    extraGroups = mkOption {
      type = types.listOf types.str;
      default = [ "dialout" ];
      description = ''
        Additional groups for the dynamically created wattwolf service user.
        By default includes 'dialout' for serial port access.
      '';
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = (cfg.connection.device == null) != (cfg.connection.socket == null);
        message = "services.wattwolf.connection: exactly one of `device` or `socket` must be set.";
      }
    ];

    systemd.services.wattwolf = {
      description = "Wattwolf Smart Meter Reader";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Restart = "on-failure";
        RestartSec = "10s";
        ExecStart = "${cfg.package}/bin/wattwolf";

        LoadCredential = mkIf (cfg.mqttPasswordFile != null) "mqtt-password:${cfg.mqttPasswordFile}";

        # Run as an ephemeral, systemd-managed user/group instead of a static one;
        # DynamicUser also implies NoNewPrivileges, ProtectSystem=strict,
        # ProtectHome=read-only, PrivateTmp and RemoveIPC.
        DynamicUser = true;
        SupplementaryGroups = cfg.extraGroups;

        # Security hardening
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" ];
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        UMask = "0077";

        # Allow device access for serial ports
        DevicePolicy = "closed";
        DeviceAllow = [ "/dev/ttyUSB* rw" "/dev/ttyACM* rw" ];
      };

      environment = {
        WATTWOLF_CONFIG = toString configFile;
        MQTT_PASSWORD_FILE = mkIf (cfg.mqttPasswordFile != null) "%d/mqtt-password";
      };
    };
  };
}

