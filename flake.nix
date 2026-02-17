{
  description = "Description for the project";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        # To import an internal flake module: ./other.nix
        # To import an external flake module:
        #   1. Add foo to inputs
        #   2. Add foo as a parameter to the outputs function
        #   3. Add here: foo.flakeModule

      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      perSystem =
        {
          config,
          self',
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };
        in
        {
          # Per-system attributes can be defined here. The self' and inputs'
          # module parameters provide easy access to attributes of the same
          # system.

          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "wattwolf";
            version = "0.1.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [ pkgs.pkg-config ];

            buildInputs = [ pkgs.systemd ];

            meta = with pkgs.lib; {
              description = "Read values from smart meter";
              license = with licenses; [
                mit
                asl20
              ];
            };
          };

          devShells.default = pkgs.mkShell {
            buildInputs = [
              # for libudev
              pkgs.systemd
            ];
            nativeBuildInputs = [
              pkgs.rust-bin.stable.latest.default
              pkgs.pkg-config

              pkgs.bacon
              pkgs.cargo-nextest
              pkgs.cargo-edit
              pkgs.cargo-outdated
              pkgs.just

              pkgs.mosquitto
            ];
          };
        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.

        nixosModules.default = { config, lib, pkgs, ... }: {
          imports = [ ./nixos-module.nix ];
          config = lib.mkIf config.services.wattwolf.enable {
            nixpkgs.overlays = [ inputs.self.overlays.default ];
          };
        };

        overlays.default = final: prev: {
          wattwolf = inputs.self.packages.${final.system}.default;
        };
      };
    };
}
