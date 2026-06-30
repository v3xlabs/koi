{pkgs}:
let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [
      "rust-src"
      "llvm-tools"
    ];
    targets = ["wasm32-unknown-unknown"];
  };

  rustfmtNightly = pkgs.rust-bin.nightly.latest.rustfmt;

  commonPackages = with pkgs; [
    rustfmtNightly
    rustToolchain
    rust-analyzer
    bacon
    just

    nodejs_24
    pnpm_11

    pkg-config
  ];

  linuxPackages = with pkgs; [
    gtk3
    webkitgtk_4_1
    xdotool
    libappindicator-gtk3
    gst_all_1.gstreamer
    gst_all_1.gst-plugins-base
  ];
in
pkgs.mkShell {
  packages = commonPackages
    ++ pkgs.lib.optionals pkgs.stdenv.isLinux linuxPackages;

  shellHook = ''
    ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [pkgs.libappindicator-gtk3]}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
    export WEBKIT_DISABLE_DMABUF_RENDERER=1
    ''}
    export pnpm_config_auto_install_peers=false
    export pnpm_config_ignore_scripts=true
    just
  '';
}
