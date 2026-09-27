{
  description = "Fieri flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
  };

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    fieri-daemon-pkg = pkgs.rustPlatform.buildRustPackage {
      pname = "fieri-daemon";
      version = "0.1.0";

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      cargoBuildFlags = [
        "-p" "fieri-daemon"
        "-p" "fieri-cli"
      ];
    };

    fieri-ui-pkg = pkgs.rustPlatform.buildRustPackage {
      pname = "fieri-ui";
      version = "0.1.0";

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      cargoBuildFlags = [ "-p" "fieri-ui" ];

      nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
      buildInputs = [ pkgs.fontconfig ];

      postFixup = ''
        wrapProgram "$out/bin/fieri-ui" \
          --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [
            pkgs.wayland
            pkgs.libxkbcommon
          ]}"
      '';
    };
  in
  {
    devShells.${system}.default =
      import ./shell.nix {
        inherit
        pkgs
        fieri-daemon-pkg
        fieri-ui-pkg;
      };

    packages.${system} = {
      daemon = fieri-daemon-pkg;
      ui = fieri-ui-pkg;
    };

    homeManagerModules.default =
      import ./fieri-module.nix;
  };
}

