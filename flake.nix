{
  description = "";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem(system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [
            pkg-config
            clang
            rust-analyzer
            cargo-outdated
            cargo-msrv
          ];

          buildInputs = [
            (rust-bin.nightly."2023-05-12".default.override {
              extensions = [ "rust-src" ];
            })
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

          RUST_BACKTRACE = 1;
        };
      }
    );
}