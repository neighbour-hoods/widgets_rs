let
  sources = import ./nix/sources.nix;
  rust-overlay = import sources.rust-overlay;
in
{ pkgs ? import sources.nixpkgs {
    overlays = [ rust-overlay ];
  }
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    (rust-bin.stable.latest.default.override {
      targets = [
        "wasm32-unknown-unknown"
      ];
    })
    miniserve
    nodePackages.rollup
    wasm-pack
  ];
}
