{
  description = "koi devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "llvm-tools"
        ];
        targets = ["wasm32-unknown-unknown"];
      };

      rustfmtNightly = pkgs.rust-bin.nightly.latest.rustfmt;
    in {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          rustfmtNightly
          rustToolchain
          rust-analyzer
          bacon
          just

          nodejs_24
          pnpm_11

          pkg-config
          gtk3
          webkitgtk_4_1
          xdotool
          libappindicator-gtk3
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
        ];

        shellHook = ''
          export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [pkgs.libappindicator-gtk3]}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
          export WEBKIT_DISABLE_DMABUF_RENDERER=1
          export pnpm_config_auto_install_peers=false
          export pnpm_config_ignore_scripts=true
          just
        '';
      };
    });
}
