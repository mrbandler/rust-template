{ pkgs, ... }:
{
  # Toolchain comes from rust-toolchain.toml (single source of truth).
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
  };

  # Keep in sync with the `setup` recipe in the justfile.
  packages = with pkgs; [
    just
    prek
    cargo-deny
    cargo-shear
    cargo-nextest
    cargo-hack
    typos
    zizmor
    mdbook
  ];

  env.RUST_BACKTRACE = "1";

  enterShell = "prek install";
}
