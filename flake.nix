{
  description = "patisserie flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixpkgs-unstable";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      naersk,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
      ];
      forAllSystems =
        fn:
        nixpkgs.lib.foldl nixpkgs.lib.recursiveUpdate { } (
          nixpkgs.lib.forEach systems (
            system:
            let
              overlays = [ (import rust-overlay) ];
              pkgs = (import nixpkgs) {
                inherit system overlays;
              };
              toolchain = pkgs.rust-bin.stable.latest.default;
              naersk-lib = pkgs.callPackage naersk {
                cargo = toolchain;
                rustc = toolchain;
              };
            in
            (fn { inherit pkgs naersk-lib system toolchain; })
          )
        );
    in
    forAllSystems (
      {
        pkgs,
        naersk-lib,
        system,
        toolchain,
      }:
      {
        defaultPackage.${system} = naersk-lib.buildPackage {
          src = ./.;
        };

        devShells.${system}.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            (toolchain.override {
              extensions = [ "rust-src" ];
            })

            openssl
            pkg-config
          ];
        };
      }
    );
}
