{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    pkgsForAllSystems = f:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ] (system:
        f {
          pkgs = import nixpkgs {
            inherit system;
            overlays = [(import rust-overlay)];
          };
        });
  in {
    devShells = pkgsForAllSystems (
      {pkgs}: {
        default =
          pkgs.mkShell.override {
            stdenv = pkgs.clangStdenv;
          } {
            packages = with pkgs; [
              taplo-lsp
              cargo-edit
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
            ];
          };
      }
    );
  };
}
