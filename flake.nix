{
  description = "Feiri flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
  };

  outputs =
    { self, nixpkgs }:
    let
      platforms = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllPlatforms = f: nixpkgs.lib.genAttrs platforms (sys: f nixpkgs.legacyPackages.${sys});

      mkFeiriPackages = pkgs: {
        daemon = pkgs.rustPlatform.buildRustPackage {
          pname = "feiri-daemon";
          version = "0.1.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          cargoBuildFlags = [
            "-p"
            "feiri-daemon"
            "-p"
            "feiri-cli"
          ];
          # The check phase runs `cargo test` separately; don't compile the UI here.
          cargoTestFlags = [
            "-p"
            "feiri-daemon"
            "-p"
            "feiri-cli"
          ];
        };

        ui = pkgs.rustPlatform.buildRustPackage {
          pname = "feiri-ui";
          version = "0.1.0";

          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          cargoBuildFlags = [
            "-p"
            "feiri-ui"
          ];
          cargoTestFlags = [
            "-p"
            "feiri-ui"
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.makeWrapper
          ];
          buildInputs = [ pkgs.fontconfig ];

          postFixup = ''
            wrapProgram "$out/bin/feiri-ui" \
              --prefix LD_LIBRARY_PATH : "${
                pkgs.lib.makeLibraryPath [
                  pkgs.wayland
                  pkgs.libxkbcommon
                ]
              }"
          '';
        };
      };
    in
    {
      packages = forAllPlatforms (pkgs: mkFeiriPackages pkgs);

      devShells = forAllPlatforms (
        pkgs:
        let
          feiriPackages = mkFeiriPackages pkgs;
        in
        {
          default = import ./shell.nix {
            inherit pkgs;
            feiri-daemon-pkg = feiriPackages.daemon;
            feiri-ui-pkg = feiriPackages.ui;
          };

          ci = pkgs.mkShell {
            inputsFrom = [
              feiriPackages.daemon
              feiriPackages.ui
            ];

            packages = with pkgs; [
              rustfmt
              clippy
            ];
          };
        }
      );
      homeManagerModules.default = import ./feiri-module.nix;
    };
}
