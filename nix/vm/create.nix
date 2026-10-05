{
  nixpkgs,
  system,
  module
}:
nixpkgs.lib.nixosSystem {
  modules = [
    ./common.nix
    module
    {
      virtualisation.host.pkgs = nixpkgs.legacyPackages.${system};
    }
  ];
}
