{
  description = "rust-template";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            }
          )
        );
      # Version comes from Cargo.toml (single source of truth).
      cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          # Same toolchain as rustup and devenv.
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "rust-template";
            version = cargo.workspace.package.version;
            src = pkgs.lib.cleanSource ./.;
            # Dependency hashes come from Cargo.lock; nothing to update by hand.
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "--package" "rust-template-cli" ];
            cargoTestFlags = [ "--package" "rust-template-cli" ];
            meta.mainProgram = "rust-template";
          };
        }
      );
    };
}
