{ pkgs, guestModule, feiriPackage, feiriUiPackage }:

pkgs.testers.runNixOSTest {
  name = "target-est";

  #guest modules configure nixpkgs.hostPlatform
  node.pkgsReadOnly = false;
  qemu.package = pkgs.qemu;

  #allow CI runners without /dev/kvm to use software emulation
  requiredFeatures.kvm = false;
  qemu.forceAccel = false;

  nodes.machine = {
    imports = [
      ../vm/common.nix
      guestModule
    ];

    environment.systemPackages = [
      feiriPackage
      feiriUiPackage
    ];
  };

  testScript = ''
    machine.start()
    machine.wait_for_unit("multi-user.target")

    with subtest("binaries are installed"):
      machine.succeed("test -x /run/current-system/sw/bin/feiri-daemon")
      machine.succeed("test -x /run/current-system/sw/bin/fury")
      machine.succeed("test -x /run/current-system/sw/bin/feiri-ui")

    with subtest("cli launches"):
      machine.succeed("fury --help")
  '';
}
