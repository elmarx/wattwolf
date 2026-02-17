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
        The password will be read from this file and passed via MQTT_PASSWORD environment variable.
        This is useful for integration with agenix or other secret management tools.
      '';
      example = "/run/agenix/wattwolf-mqtt-password";
    };

    user = mkOption {
      type = types.str;
      default = "wattwolf";
      description = "User account under which wattwolf runs.";
    };

    group = mkOption {
      type = types.str;
      default = "wattwolf";
      description = "Group under which wattwolf runs.";
    };

    extraGroups = mkOption {
      type = types.listOf types.str;
      default = [ "dialout" ];
      description = ''
        Additional groups for the wattwolf user.
        By default includes 'dialout' for serial port access.
      '';
    };
  };

  config = mkIf cfg.enable {
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      extraGroups = cfg.extraGroups;
      description = "wattwolf smart meter reader service user";
    };

    users.groups.${cfg.group} = { };

    systemd.services.wattwolf = {
      description = "Wattwolf Smart Meter Reader";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        Restart = "on-failure";
        RestartSec = "10s";

        # Security hardening
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ ];
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [ "AF_UNIX" "AF_INET" "AF_INET6" ];
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        RemoveIPC = true;
        PrivateMounts = true;
        SystemCallArchitectures = "native";
        UMask = "0077";

        # Allow device access for serial ports
        DevicePolicy = "closed";
        DeviceAllow = [ "/dev/ttyUSB* rw" "/dev/ttyACM* rw" ];
      };

      environment = {
        WATTWOLF_CONFIG = toString (pkgs.writeText "config.toml" cfg.config);
      };

      script = ''
        ${optionalString (cfg.mqttPasswordFile != null) ''
          export MQTT_PASSWORD="$(cat ${escapeShellArg cfg.mqttPasswordFile})"
        ''}
        exec ${cfg.package}/bin/wattwolf
      '';
    };
  };
}

