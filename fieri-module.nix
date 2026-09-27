{ config, lib, pkgs, ... }:

let
  cfg = config.services.fieri;
in
{
  options.services.fieri = {
    enable = lib.mkEnableOption "Fieri daemon";

    package = lib.mkOption {
      type = lib.types.package;
      description = "Package containing the fieri daemon executable";
    };

    ui = {
      enable = lib.mkEnableOption "Fieri UI";

      package = lib.mkOption {
        type = lib.types.package;
        description = "Package containing the fieri ui executable";
      };
    };

    environment = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      description = "Environment variables for the fieri daemon";
    };

  };

  config = lib.mkIf cfg.enable {
    home.packages = [
      cfg.package
    ] ++ lib.optional cfg.ui.enable cfg.ui.package;

    systemd.user.services.fieri = {
      Unit = {
        Description = "Fieri daemon";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
      };

      Service = {
        ExecStart = "${cfg.package}/bin/fieri-daemon";
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
