{
  pkgs ? import <feiri-daemon-pkg> { },
  feiri-daemon-pkg,
  feiri-ui-pkg,
}:

let
  slint-lsp-override = pkgs.slint-lsp.overrideAttrs (
    final: prev: {
      version = "1.18.1";
      src = pkgs.fetchCrate {
        pname = "slint-lsp";
        version = final.version;
        hash = "sha256-et+y/JXBCG0RFFY8viMPGjneQAzyQKD+pBSpsKBt+DU=";
      };
      cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
        src = final.src;
        hash = "sha256-vC9flaCsV3DwnafunrXglP/TO46O8n2lZN2im6i8BQE=";
      };
    }
  );
in
pkgs.mkShell {
  inputsFrom = [
    feiri-daemon-pkg
    feiri-ui-pkg
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.wayland
    pkgs.libxkbcommon
  ];

  packages = with pkgs; [
    rustfmt
    clippy
    rust-analyzer

    nixd
    nixfmt

    slint-lsp-override
    python3
    pyright

    cargo-watch
    just
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  shellHook = ''
    echo "Entered Fieri shell..."
  '';
}
