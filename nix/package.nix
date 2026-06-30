{
  pkgs,
  version,
  rustTarget,
  hash,
}:
pkgs.stdenv.mkDerivation {
  pname = "koi";
  inherit version;

  src = pkgs.fetchurl {
    url = "https://github.com/v3xlabs/koi/releases/download/v${version}/koi-${rustTarget}.tar.gz";
    inherit hash;
  };

  nativeBuildInputs = with pkgs; []
    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
      autoPatchelfHook
      makeWrapper
      patchelf
    ];

  buildInputs = with pkgs; []
    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
      gtk3
      webkitgtk_4_1
      libappindicator-gtk3
      xdotool
      gst_all_1.gstreamer
      gst_all_1.gst-plugins-base
      stdenv.cc.cc.lib
    ];

  sourceRoot = ".";

  postUnpack = pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    chmod +w koi
    patchelf --replace-needed libxdo.so.3 ${pkgs.xdotool}/lib/libxdo.so.4 koi
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 koi $out/bin/koi
  '' + pkgs.lib.optionalString pkgs.stdenv.isLinux ''
    wrapProgram $out/bin/koi \
      --set WEBKIT_DISABLE_DMABUF_RENDERER 1
  '' + ''
    runHook postInstall
  '';

  meta = with pkgs.lib; {
    description = "Privacy-focused Ethereum wallet";
    homepage = "https://github.com/v3xlabs/koi";
    mainProgram = "koi";
    platforms = builtins.attrNames (import ./versions.nix).rustTargets;
  };
}
