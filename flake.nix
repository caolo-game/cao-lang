{
  description = "cao-lang devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            python3
            python3Packages.tox
            python3Packages.setuptools-rust
            just
            nodejs
            (rust-bin.nightly.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            cargo-edit
            cargo-all-features
            wasm-pack
            wasm-bindgen-cli
            stdenv.cc
            ninja
            cmake
            cargo-deny
            firefox
            git-cliff
          ];
        };
      }
    );
}
