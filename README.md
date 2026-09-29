<div align="center">
  <h1>
    Feiri <small><em>(FYUR-ee)</em></small>
  </h1>
</div>

<p align="center">
    <strong>Supercharge your Niri workflow</strong>
</p>

<p align="center">
    A growing collection of tools to make navigating, organizing, and interacting with Niri faster and more effortless
</p>

<p align="center">
    <img src="https://img.shields.io/badge/Compositor-Niri-00BFFF?style=for-the-badge" alt="Niri"/>
    <img
        src="https://img.shields.io/badge/Built_with-Rust-orange?style=for-the-badge&logo=rust&logoColor=white"
        alt="Rust"/>
    <img
        src="https://img.shields.io/badge/Platform-Wayland-7D5FFF?style=for-the-badge&logo=wayland&logoColor=white"
        alt="Wayland"/>
</p>


## Features


## Installation
Feiri consists of three executables:
- `feiri-daemon` connects to Niri and manages key functionality such as marks.
- `fury` is the CLI used by Niri keybindings to send commands to the daemon.
- `feiri-ui` is a supplemental add-on that makes working with Feiri easer.

> [!NOTE]
> Must install *both* `feiri-daemon` and `fury` executables. The CLI cannot manage state unless the daemon is running in your Niri session, and the daemon cannot be used via Niri unless the CLI is installed. The UI is completely optional and only improves experience.

### Nix
The flake's `daemon` package for `x86_64-linux` contains *both* `feiri-daemon` and `fury`. Install and check availability via:
```sh
nix profile install github:dioeos/feiri#daemon

command -v feiri-daemon
command -v fury
```


The flake also has the `ui` package that contains the `feiri-ui`. Install and check availability via:
```sh
nix profile install github:dioeos/feiri#ui

command -v feiri-ui
```

#### Home Manager (Recommended)
The flake also offers a [Home Manager](https://github.com/nix-community/home-manager) module. To install Feiri within your user environment, add the following to your the `inputs` section of your flake configuration:
```nix
{
  inputs = {
    feiri = { url = "github:dioeos/feiri"; };
  }
}
```

Once added, enable the executables by using the module:
```nix
{ inputs, ... }:

{
  imports = [
    inputs.feiri.homeManagerModules.default
  ];

  services.feiri = {
    enable = true;
    package = inputs.feiri.packages.x86_64-linux.daemon;

    ui = {
      enable = true;
      package = inputs.feiri.packages.x86_64-linux.ui;
    };
  };
}
```

This installs all three exectuables and configures a systemd user service to start the daemon with your graphical session. This allows the daemon to run on system start without needing a separate Niri `spawn-at-startup` entry when using the service.

## Niri Integration
> [!NOTE]
> In order for the CLI commands to be work with Niri, the Feiri daemon needs to be running

It is recommend to have the daemon autostart. If you installed Feiri by any method other than the Home Manager module, you will need to start the `feiri-daemon` yourself. The easiest way to do this is by adding the following to your Niri configuration:

```kdl
spawn-at-startup "feiri-daemon"
```

To use Feiri from Niri, add the following quickstart configuration to your Niri configuration's `binds` section:

```kdl
Super+Shift+1 { spawn "fury" "msg" "action" "mark-window" "1"; }
Super+Shift+2 { spawn "fury" "msg" "action" "mark-window" "2"; }
Super+Shift+3 { spawn "fury" "msg" "action" "mark-window" "3"; }
Super+Shift+4 { spawn "fury" "msg" "action" "mark-window" "4"; }
Super+Shift+5 { spawn "fury" "msg" "action" "mark-window" "5"; }
Super+Shift+6 { spawn "fury" "msg" "action" "mark-window" "6"; }
Super+Shift+7 { spawn "fury" "msg" "action" "mark-window" "7"; }
Super+Shift+8 { spawn "fury" "msg" "action" "mark-window" "8"; }
Super+Shift+9 { spawn "fury" "msg" "action" "mark-window" "9"; }

Super+1 { spawn "fury" "msg" "action" "focus-mark" "1"; }
Super+2 { spawn "fury" "msg" "action" "focus-mark" "2"; }
Super+3 { spawn "fury" "msg" "action" "focus-mark" "3"; }
Super+4 { spawn "fury" "msg" "action" "focus-mark" "4"; }
Super+5 { spawn "fury" "msg" "action" "focus-mark" "5"; }
Super+6 { spawn "fury" "msg" "action" "focus-mark" "6"; }
Super+7 { spawn "fury" "msg" "action" "focus-mark" "7"; }
Super+8 { spawn "fury" "msg" "action" "focus-mark" "8"; }
Super+9 { spawn "fury" "msg" "action" "focus-mark" "9"; }

Mod+N { spawn "fury" "msg" "action" "next-mark"; }
Mod+B { spawn "fury" "msg" "action" "prev-mark"; }

Mod+Shift+M {
    spawn "sh" "-c" "if pkill -x feiri-ui; then :; else exec feiri-ui; fi";
}

```

## Why Feiri?

Got too tired of having to manually switch between frequently used windows and having to mentally track where the important stuff was. Wanting to have [marks.nvim](https://github.com/chentoast/marks.nvim) and [harpoon](https://github.com/theprimeagen/harpoon) functionality with my windows and tabs, I created Feiri. On paper, the second difference (measured in `scripts/timer.py`) in navigation does not seem like a lot, but it does save a lot of effort...
