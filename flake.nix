{
  description = "nql - Rust project using fenix latest nightly toolchain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs @ { self, nixpkgs, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        toolchain = fenix.packages.${system}.latest.toolchain;
        rust-analyzer = fenix.packages.${system}.rust-analyzer;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            rust-analyzer

            pkgs.just

            # for openssl-sys:
            pkgs.openssl
            pkgs.pkg-config
          ];
        };
      }
    );
}


