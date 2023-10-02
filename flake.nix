{
  description =
    "Simple, fast, and featureful alternative to rm and trash-cli written in rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs;
          mkShell {
            buildInputs =
              [ cargo rustc rustfmt pre-commit rustPackages.clippy ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
      });
}
