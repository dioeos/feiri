{ config, lib, pkgs, ... }:

let
  cfg = config.services.feiri;
in
{
  options.services.feiri = {
    enable = lib.mkEnableOption "Feiri";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Package containing the feiri daemon and cli executable";
    };

    ui = {
      enable = lib.mkEnableOption "Feiri UI";

      package = lib.mkOption {
        type = lib.types.package;
        description = "Package containing the feiri ui executable";
      };
    };

    environment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Environment variables for the feiri daemon";
    };

  };

  config = lib.mkIf cfg.enable {
    home.packages = [
      cfg.package
    ] ++ lib.optional cfg.ui.enable cfg.ui.package;

    systemd.user.services.feiri = {
      Unit = {
        Description = "Feiri daemon";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
      };

      Service = {
        ExecStart = "${cfg.package}/bin/feiri-daemon";
        Restart = "on-failure";

        Environment = lib.mapAttrsToList (
          name: value: "${name}=${value}"
        ) cfg.environment;
      };

      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };
  };
}
