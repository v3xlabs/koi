{pkgs}: let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [
      "rust-src"
      "llvm-tools"
    ];
    targets = [
      "aarch64-linux-android"
      "armv7-linux-androideabi"
      "x86_64-linux-android"
      "aarch64-apple-ios"
      "aarch64-apple-ios-sim"
      "x86_64-apple-ios"
    ];
  };

  rustfmtNightly = pkgs.rust-bin.nightly.latest.rustfmt;

  # Exactly one NDK: exported as $ANDROID_NDK_HOME and consumed by the
  # justfile — never hand-install a second one.
  ndkVersion = "28.2.13676358";

  androidSdk = pkgs.androidenv.composeAndroidPackages {
    platformVersions = ["35" "36"];
    buildToolsVersions = ["28.0.3" "36.0.0"];
    includeNDK = true;
    ndkVersions = [ndkVersion];
    # flutter's gradle plugin runs cmake configure tasks; AGP can't
    # auto-install into the read-only store
    cmakeVersions = ["3.22.1"];
    includeEmulator = false;
    includeSystemImages = false;
  };
in
  pkgs.mkShell {
    packages = with pkgs; [
      rustfmtNightly
      rustToolchain
      rust-analyzer
      bacon
      just
      cargo-ndk

      flutter

      android-tools
      androidSdk.androidsdk

      temurin-bin-21

      pkg-config
      openssl
    ];

    ANDROID_HOME = "${androidSdk.androidsdk}/libexec/android-sdk";
    ANDROID_NDK_HOME = "${androidSdk.androidsdk}/libexec/android-sdk/ndk/${ndkVersion}";

    shellHook = ''
      # flutter_rust_bridge_codegen is cargo-installed (pinned =2.9.0, must
      # match the Cargo.toml + pubspec pins)
      export PATH="$HOME/.cargo/bin:$PATH"
    '';
  }
