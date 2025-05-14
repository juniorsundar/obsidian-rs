{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    fenix,
    nixpkgs,
  }: let
    # Define supported systems
    supportedSystems = ["x86_64-linux" "aarch64-linux"];

    # Helper function to generate outputs for each system
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

    # Define the Rust toolchain components
    rustToolchainComponents = [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
    ];
  in {
    # Development shells for each supported system
    devShells = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      rustToolchain = fenix.packages.${system}.complete.withComponents rustToolchainComponents;
    in {
      default = pkgs.mkShell {
        name = "rust-dev-shell";
        packages = [
          rustToolchain
          pkgs.rust-analyzer # Or fenix.packages.${system}.rust-analyzer for fenix provided one
          # Add other development tools specific to the shell here
          # e.g., pkgs.openssl pkgs.pkg-config
        ];
        # You can set environment variables here if needed
        # RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };
    });
  };
}
