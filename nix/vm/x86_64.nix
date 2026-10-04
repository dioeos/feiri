{ ... }:

{
  nixpkgs.hostPlatform = "x86_64-linux";
  networking.hostName = "feiri-x86-64";

  system.stateVersion = "26.05";
}
