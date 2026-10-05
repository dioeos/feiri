{
  modulesPath,
  pkgs,
  ...
}:

{
  imports = [
    (modulesPath + "/virtualisation/qemu-vm.nix")
  ];

  nix.settings.experimental-features = [
    "nix-command"
    "flakes"
  ];

  environment.systemPackages = with pkgs; [
    git
    curl
    vim
  ];

  users.users.nixos = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    initialPassword = "nixos";
  };
  security.sudo.wheelNeedsPassword = false;
  services.getty.autologinUser = "nixos";

  virtualisation = {
    memorySize = 4096;
    cores = 4;
    graphics = false;
  };
}
