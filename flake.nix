{
  description = "Feiri flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    feiri-daemon-pkg = pkgs.rustPlatform.buildRustPackage {
      pname = "feiri-daemon";
      version = "0.1.0";

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      cargoBuildFlags = [
        "-p" "feiri-daemon"
        "-p" "feiri-cli"
      ];
      # The check phase runs `cargo test` separately; don't compile the UI here.
      cargoTestFlags = [
        "-p" "feiri-daemon"
        "-p" "feiri-cli"
      ];
    };

    feiri-ui-pkg = pkgs.rustPlatform.buildRustPackage {
      pname = "feiri-ui";
      version = "0.1.0";

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      cargoBuildFlags = [ "-p" "feiri-ui" ];
      cargoTestFlags = [ "-p" "feiri-ui" ];

      nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
      buildInputs = [ pkgs.fontconfig ];

      postFixup = ''
        wrapProgram "$out/bin/feiri-ui" \
          --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
            pkgs.wayland
            pkgs.libxkbcommon
          ]}"
      '';
    };
  in
  {
    devShells.${system} = {
      default =
        import ./shell.nix {
          inherit
          pkgs
          feiri-daemon-pkg
          feiri-ui-pkg;
        };

      #@NOTE: CI lightweight shell (does not include inputs to build Slint UI yet)
      ci = pkgs.mkShell {
        inputsFrom = [
          feiri-daemon-pkg
          feiri-ui-pkg
        ];

        packages = with pkgs; [
          rustfmt
          clippy
        ];
      };
    };

    packages.${system} = {
      daemon = feiri-daemon-pkg;
      ui = feiri-ui-pkg;
    };

    homeManagerModules.default =
      import ./feiri-module.nix;
  };
}

