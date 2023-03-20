{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    devshell.url = "github:numtide/devshell";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    herbstluftwm = {
      url = "github:herbstluftwm/herbstluftwm";
      flake = false;
    };
  };
  outputs =
    { self, nixpkgs, utils, fenix, naersk, devshell, herbstluftwm, ... }@inputs:
    utils.lib.eachSystem [ "x86_64-linux" ] (system:
    let
      lib = nixpkgs.lib;
      stdenv = pkgs.stdenv;
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ devshell.overlays.default ];
      };
      rust-toolchain = with fenix.packages.${system};
        combine [
          stable.rustc
          stable.cargo
          stable.clippy
          stable.rustfmt
          targets.x86_64-unknown-linux-musl.stable.rust-std
        ];
      C_INCLUDE_PATH = with pkgs;
        lib.concatStringsSep ":" [
          "${xorg.libX11.dev}/include"
          "${xorg.xorgproto}/include"
        ];
      LD_LIBRARY_PATH = "${pkgs.libclang.lib}/lib";
      LIBRARY_PATH = "${pkgs.xorg.libX11}/lib";

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
          nativeBuildInputs = with pkgs; [ installShellFiles ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          preConfigure = ''
            export HERBSTLUFTWM_DIR=${herbstluftwm}
            export BINDGEN_EXTRA_CLANG_ARGS="-isystem ${pkgs.musl.dev}/include"
            export LD_LIBRARY_PATH=${LD_LIBRARY_PATH}
            export LIBRARY_PATH=${LIBRARY_PATH}
            export C_INCLUDE_PATH=${C_INCLUDE_PATH}
          '';
          postInstall = ''
            OUT_DIR=target/x86_64-unknown-linux-musl/release/build/herbst3-*/out
            install -Dm0444 -t $out/etc/profile.d $OUT_DIR/herbst3.bash
            install -Dm0444 -t $out/share/zsh/site-functions $OUT_DIR/_herbst3
            install -Dm0444 -t $out/share/fish/vendor_functions.d $OUT_DIR/herbst3.fish
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
          gcc
          musl.dev
        ];
        git.hooks = {
          enable = true;
          pre-commit.text = "nix flake check";
        };
        env = [
          {
            name = "LD_LIBRARY_PATH";
            value = LD_LIBRARY_PATH;
          }
          {
            name = "LIBRARY_PATH";
            value = LIBRARY_PATH;
          }
          {
            name = "C_INCLUDE_PATH";
            value = C_INCLUDE_PATH;
          }
        ];
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
        herbst3 = self.packages.${system}.herbst3;
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

