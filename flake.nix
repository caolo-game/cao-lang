{
  description = "cao-lang devshell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # python = pkgs.python3;
        # pythonldlibpath = pkgs.lib.makeLibraryPath (with pkgs; [
        #   pkgs.stdenv.cc.cc
        # ]);
        #
        # patchedpython = (python.overrideAttrs (
        #   previousAttrs: {
        #     # Add the nix-ld libraries to the LD_LIBRARY_PATH.
        #     # creating a new library path from all desired libraries
        #     postInstall = previousAttrs.postInstall + ''
        #       mv  "$out/bin/python3.12" "$out/bin/unpatched_python3.12"
        #       cat << EOF >> "$out/bin/python3.12"
        #       #!/run/current-system/sw/bin/bash
        #       export LD_LIBRARY_PATH="${pythonldlibpath}"
        #       exec "$out/bin/unpatched_python3.12" "\$@"
        #       EOF
        #       chmod +x "$out/bin/python3.12"
        #     '';
        #   }
        # ));

      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            python3
            python3Packages.tox
            just
            nodejs
            (rust-bin.nightly.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            cargo-edit
            cargo-all-features
            wasm-pack
            wasm-bindgen-cli
            stdenv.cc
            ninja
            cmake
          ];
        };
      }
    );
}

