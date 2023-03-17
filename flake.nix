{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "git+https://github.com/numtide/flake-utils.git";
    devshell.url = "github:numtide/devshell";
    fenix = {
      url = "git+https://github.com/nix-community/fenix.git?ref=main";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "git+https://github.com/nix-community/naersk.git";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, utils, fenix, naersk, devshell, ... }@inputs:
    utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        lib = nixpkgs.lib;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };

        rust-toolchain = with fenix.packages.${system};
          combine [
            latest.rustc
            latest.cargo
            latest.clippy
            latest.rustfmt
            targets.x86_64-unknown-linux-musl.latest.rust-std
          ];

        # overrides a naersk-lib which uses the stable toolchain expressed above
        naersk-lib = (naersk.lib.${system}.override {
          cargo = rust-toolchain;
          rustc = rust-toolchain;
        });
      in
      rec {
        packages = {
          default = packages.herbst3;
          herbst3 = naersk-lib.buildPackage rec {
            pname = "herbst3";
            root = ./.;
            cargoBuildOptions = x:
              x ++ [ "--target" "x86_64-unknown-linux-musl" ];
            cargoTestOptions = x:
              x ++ [ "--target" "x86_64-unknown-linux-musl" ];
            nativeBuildInputs = [ pkgs.installShellFiles ];
            postInstall = ''
              OUT_DIR=target/x86_64-unknown-linux-musl/release/build/herbst3-*/out
              install -Dm0444 -t $out/etc/profile.d $OUT_DIR/herbst3.bash
              install -Dm0444 -t $out/share/zsh/site-functions $OUT_DIR/_herbst3
              install -Dm0444 -t $out/share/fish/vendor_functions.d $OUT_DIR/herbst3.fish

              # installShellCompletion --bash $OUT_DIR/herbst3.bash
              # installShellCompletion --zsh $OUT_DIR/_herbst3
              # installShellCompletion --fish $OUT_DIR/herbst3.fish
            '';
          };
        };

        # a devshell with all the necessary bells and whistles
        devShells.default = (pkgs.devshell.mkShell {
          imports = [ "${devshell}/extra/git/hooks.nix" ];
          name = "herbst3-dev-shell";
          packages = with pkgs; [
            rust-toolchain
            rust-analyzer
            cargo-outdated
            cargo-udeps
            cargo-watch
            nixpkgs-fmt
          ];
          git.hooks = {
            enable = true;
            pre-commit.text = "nix flake check";
          };
          commands = [
            { package = "treefmt"; }
            {
              name = "udeps";
              command = ''
                PATH=${fenix.packages.${system}.latest.rustc}/bin:$PATH
                cargo udeps $@
              '';
              help = pkgs.cargo-udeps.meta.description;
            }
            {
              name = "outdated";
              command = "cargo-outdated outdated";
              help = pkgs.cargo-outdated.meta.description;
            }
          ];
        });

        # always check these
        checks = {
          nixpkgs-fmt = pkgs.runCommand "nixpkgs-fmt"
            {
              nativeBuildInputs = [ pkgs.nixpkgs-fmt ];
            } "nixpkgs-fmt --check ${./.}; touch $out";
          cargo-fmt = pkgs.runCommand "cargo-fmt"
            {
              nativeBuildInputs = [ rust-toolchain ];
            } "cd ${./.}; cargo fmt --check; touch $out";
        };

        # instructions for the CI server
        hydraJobs = packages // checks;
      });
}

