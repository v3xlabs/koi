{
  description = "koi devshell and pre-built packages";

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
      versions = import ./nix/versions.nix;

      pkgs = import nixpkgs {inherit system;};

      devPkgs = import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      };

      rustTarget =
        versions.rustTargets.${system}
        or (throw "koi: no pre-built release for ${system}");

      mkKoi = version:
        import ./nix/package.nix {
          inherit pkgs version rustTarget;
          hash =
            versions.hashes.${version}.${rustTarget}
            or (throw "koi: no hash for version ${version} on ${rustTarget}");
        };
    in {
      packages = {
        default = mkKoi versions.latest;
        koi = mkKoi versions.latest;
        koi-v0_0_1 = mkKoi "0.0.1";
      };

      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/koi";
      };

      devShells.default = import ./nix/devshell.nix {pkgs = devPkgs;};
    });
}
