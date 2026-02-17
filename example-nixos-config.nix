# Example NixOS Configuration using wattwolf module
#
# This file demonstrates how to use the wattwolf NixOS module
# in a complete NixOS configuration.

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
          # Example 1: Basic configuration with serial device
          services.wattwolf = {
            enable = true;
            config = ''
              [connection]
              device = "/dev/ttyUSB0"

              [mqtt]
              host = "localhost"
              port = 1883
              topic_prefix = "wattwolf"
              username = "wattwolf"
              password = "changeme"  # Better: use mqttPasswordFile instead!

              [[sensors]]
              name = "consumed"
              friendly_name = "Energy Consumed"
              obis = "1-0:1.8.0*255"
              device_class = "energy"
              state_class = "total_increasing"

              [[sensors]]
              name = "produced"
              friendly_name = "Energy Produced"
              obis = "1-0:2.8.0*255"
              device_class = "energy"
              state_class = "total_increasing"
            '';
          };
        })

        # Example 2: Configuration with agenix secret management
        # Uncomment this and comment out Example 1 above
        # ({ config, pkgs, ... }: {
        #   services.wattwolf = {
        #     enable = true;
        #     mqttPasswordFile = "/run/agenix/wattwolf-mqtt-password";
        #     config = ''
        #       [connection]
        #       socket = "smart-meter-gateway:2000"
        #
        #       [mqtt]
        #       host = "mqtt.example.com"
        #       port = 1883
        #       topic_prefix = "home/energy"
        #       username = "wattwolf"
        #       # password will be read from mqttPasswordFile
        #
        #       [[sensors]]
        #       name = "consumed"
        #       friendly_name = "Energy Consumed"
        #       obis = "1-0:1.8.0*255"
        #       device_class = "energy"
        #       state_class = "total_increasing"
        #
        #       [[sensors]]
        #       name = "produced"
        #       friendly_name = "Energy Produced"
        #       obis = "1-0:2.8.0*255"
        #       device_class = "energy"
        #       state_class = "total_increasing"
        #     '';
        #   };
        #
        #   # agenix secret configuration
        #   age.secrets.wattwolf-mqtt-password = {
        #     file = ./secrets/wattwolf-mqtt-password.age;
        #     owner = config.services.wattwolf.user;
        #     group = config.services.wattwolf.group;
        #   };
        # })
      ];
    };
  };
}

