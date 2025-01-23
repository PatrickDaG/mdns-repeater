{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    nci.url = "github:yusdacra/nix-cargo-integration";
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devshell.flakeModule
        inputs.nci.flakeModule
        inputs.pre-commit-hooks.flakeModule
        inputs.treefmt-nix.flakeModule
      ];

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        {
          config,
          pkgs,
          system,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            config.allowUnfree = true;
          };

          pre-commit.settings.hooks = {
            treefmt.enable = true;
          };
          devshells.default = {
            packages = with pkgs; [
              hello
              config.treefmt.build.wrapper
              (python3.withPackages (p: with p; [ tqdm ]))
            ];
            commands = [
              {
                package = pkgs.nix;
                help = "nichts";
              }
            ];
            env = [
              {
                name = "TEST";
                value = "TSET";
              }
            ];
            devshell.startup.pre-commit.text = config.pre-commit.installationScript;
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              deadnix.enable = true;
              statix.enable = true;
              shellcheck.enable = true;
              beautysh.enable = true;
              nixfmt.enable = true;
              rustfmt.enable = true;
            };
          };

          # Rust
          nci.projects.mdns-repeater = {
            path = ./.;
            numtideDevshell = "default";
          };
          nci.crates.mdns-repeater = {
            depsDrvConfig = {
              mkDerivation = {
                nativeBuildInputs = [ pkgs.pkg-config ];
              };
            };
            drvConfig = {
              mkDerivation = {
                nativeBuildInputs = [ pkgs.pkg-config ];
                meta.mainProgram = "mdns-repeater";
              };
            };
          };
          packages.default = config.nci.outputs.mdns-repeater.packages.release;
        };
    };
}
