{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.wattwolf;
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

    config = mkOption {
      type = types.str;
      description = ''
        Configuration for wattwolf in TOML format.
        This will be written to a file and passed to wattwolf via WATTWOLF_CONFIG environment variable.
      '';
      example = literalExpression ''
        '''
          [connection]
          socket = "localhost:2000"

          [mqtt]
          host = "localhost"
          port = 1883
          topic_prefix = "wattwolf"

          [[sensors]]
          name = "consumed"
          friendly_name = "Energy Consumed"
          obis = "1.8.0"
          device_class = "energy"
          state_class = "total_increasing"
        '''
      '';
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
        WATTWOLF_CONFIG = toString (pkgs.writeText "config.toml" cfg.config);
        MQTT_PASSWORD_FILE = mkIf (cfg.mqttPasswordFile != null) "%d/mqtt-password";
      };
    };
  };
}

