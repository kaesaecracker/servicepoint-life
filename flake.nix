{
  description = "Flake for servicepoint-life";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    nix-filter.url = "github:numtide/nix-filter";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      naersk,
      nix-filter,
    }:
    let
      lib = nixpkgs.lib;
      supported-systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forAllSystems =
        f:
        lib.genAttrs supported-systems (
          system:
          f rec {
            pkgs = nixpkgs.legacyPackages.${system};
            naersk' = pkgs.callPackage naersk { };
            selfPkgs = self.packages."${system}";
            inherit system;
          }
        );
    in
    rec {
      packages = forAllSystems (
        { pkgs, naersk', ... }:
        rec {
          servicepoint-life = naersk'.buildPackage rec {
            strictDeps = true;
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [
              xe
              xz
            ];
            src = nix-filter.lib.filter {
              root = ./.;
              include = [
                ./Cargo.toml
                ./Cargo.lock
                ./src
                ./README.md
                ./LICENSE
              ];
            };
          };

          default = servicepoint-life;
        }
      );

      legacyPackages = packages;

      apps = forAllSystems (
        { selfPkgs, ... }:
        rec {
          servicepoint-life = {
            type = "app";
            program = "${selfPkgs.servicepoint-life}/bin/servicepoint-life";
          };
          default = servicepoint-life;
        }
      );

      devShells = forAllSystems (
        { pkgs, selfPkgs, ... }:
        {
          default = pkgs.mkShell rec {
            inputsFrom = [ selfPkgs.default ];
            packages = [
              pkgs.gdb
              (pkgs.symlinkJoin {
                name = "rust-toolchain";
                paths = with pkgs; [
                  rustc
                  cargo
                  rustfmt
                  clippy
                ];
              })
            ];
          };
        }
      );

      formatter = forAllSystems ({ pkgs, ... }: pkgs.nixfmt-rfc-style);
    };
}
