{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      system = "x86_64-linux";
      target = "x86_64-unknown-linux-gnu";
      name = "killjoy";

      pkgs = (import nixpkgs) {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };
      src =
        let
          unfilteredRoot = ./.;
        in
        pkgs.lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = craneLib.fileset.commonCargoSources unfilteredRoot;
        };
      craneLib = (crane.mkLib pkgs).overrideToolchain (
        p:
        p.rust-bin.stable.latest.default.override {
          targets = [ target ];
        }
      );
      package =
        let
          commonArgs = {
            inherit src;
            strictDeps = true;
            doCheck = false;
            CARGO_BUILD_TARGET = target;
            buildInputs = [
              pkgs.udev
            ];
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            NIX_DEV_VERSION = "${
              if self ? rev then
                self.rev
              else if self ? dirtyRev then
                self.dirtyRev
              else
                ""
            }";
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          };
          cargoArtifacts = craneLib.buildDepsOnly (
            commonArgs
            // {
            }
          );
        in
        craneLib.buildPackage (
          commonArgs
          // {
            pname = name;
            release = true;
            inherit cargoArtifacts;
            propagatedBuildInputs = with pkgs; [
              rofi
            ];
          }
        );
    in
    {
      defaultPackage.${system} = package;
      devShell.${system} = pkgs.mkShell {
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        buildInputs = with pkgs; [
          cargo
          cargo-audit
          cargo-bundle
          cargo-llvm-cov
          cargo-machete
          cargo-nextest
          cargo-outdated
          cargo-udeps
          cargo-unused-features
          clippy
          rust-analyzer
          rustc
          rustc.llvmPackages.llvm
          rustfmt
          rustup
          clang
          libclang
        ];
      };
    };
}
