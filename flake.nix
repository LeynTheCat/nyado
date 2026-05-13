{
  description = "A Rust todo-list manager with TUI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        packages.default = naersk-lib.buildPackage {
          pname = "nyado";
          root = ./.;
        };
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/nyado";
        };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ rustc cargo ];
        };
      }
    );
}