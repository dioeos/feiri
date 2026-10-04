{ ... }:

{
  nixpkgs.hostPlatform = "aarch64-linux";
  networking.hostName = "feiri-aarch64";

  virtualisation.qemu.options = [
    "-machine"
    "virt"
    "-cpu"
    "max"
  ];

  system.stateVersion = "26.05";
}
