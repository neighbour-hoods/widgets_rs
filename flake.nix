{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rustVersion = "1.62.1";
        wasmTarget = "wasm32-unknown-unknown";
        #
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShell =
          pkgs.mkShell {
            buildInputs = with pkgs; [
              (rust-bin.stable.${rustVersion}.default.override {
                targets = [ wasmTarget ];
              })
              miniserve
              nodejs
              nodePackages.rollup
              openssl
              pkgconfig
              wasm-pack
            ];
          };
      });
}
