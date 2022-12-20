{
  inputs = {
    devenv.url = "github:cachix/devenv";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , fenix
    , devenv
    } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
    let pkgs = import nixpkgs {
      inherit system;
    };
    in
    {
      devShell = devenv.lib.mkShell {
        inherit inputs pkgs;

        modules = [
          {
            packages = with pkgs; [
              solc
            ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk; [
              libiconv
              frameworks.Security
            ]);

            # https://devenv.sh/languages/
            languages.nix.enable = true;
            languages.rust = {
              enable = true;
              version = "stable";
              packages = {
                rustfmt = inputs.fenix.packages.${pkgs.system}.latest.rustfmt;
                clippy = inputs.fenix.packages.${pkgs.system}.latest.clippy;
              };
            };

            # https://devenv.sh/pre-commit-hooks/
            pre-commit.hooks = {
              shellcheck.enable = true;

              clippy.enable = true;
              rustfmt.enable = true;
            };
          }
        ];
      };
    });
}
