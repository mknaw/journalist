{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, pre-commit-hooks }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
      in
      with pkgs;
      {
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            configPath = ".nix-pre-commit-config.yaml";
            hooks = {
              rustfmt.enable = true;
              cargo-sort = {
                enable = true;
                name = "cargo-sort";
                entry = "cargo-sort";
                types = [ "toml" ];
                language = "system";
              };

              nixfmt-rfc-style.enable = true;
              deadnix.enable = true;
            };
          };
        };

        devShell = mkShell {
          buildInputs = self.checks.${system}.pre-commit-check.enabledPackages ++ [
            (rust-bin.stable.latest.minimal.override {
              extensions = [ "clippy" "rust-analyzer" "rust-docs" "rust-src" ];
            })
            # We use nightly rustfmt features.
            (rust-bin.selectLatestNightlyWith (toolchain: toolchain.rustfmt))

            cargo-sort
            bacon
          ];
        };
      });
}
