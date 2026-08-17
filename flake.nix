{
  description = "Lowestbins made in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        inherit (pkgs) lib;

        manifest = lib.importTOML ./Cargo.toml;
        rustToolchain = pkgs.rust-bin.nightly.latest.minimal;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          pname = manifest.package.name;
          version = manifest.package.version;
          strictDeps = true;

          RUSTFLAGS = "--cfg reqwest_unstable";
          CARGO_PROFILE_RELEASE_INCREMENTAL = "false";

          nativeBuildInputs = with pkgs; [
            pkg-config
            curl
          ];

          buildInputs = with pkgs; [
            openssl
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        lowestbins = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.removeReferencesTo ];

            postFixup = ''
              for bin in "$out"/bin/*; do
                remove-references-to -t ${rustToolchain} "$bin"
              done
            '';

            meta = {
              inherit (manifest.package) description;
              homepage = manifest.package.homepage;
              license = lib.licenses.asl20;
              mainProgram = "lowestbins";
            };
          }
        );

        app = {
          type = "app";
          program = lib.getExe lowestbins;
        };

        rustDevToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };
      in
      {
        packages = {
          default = lowestbins;
          inherit lowestbins;
        };

        apps = {
          default = app;
          lowestbins = app;
        };

        checks.default = lowestbins;

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustDevToolchain
            openssl
            pkg-config
            eza
            fd
            clang
            mold
            ffmpeg
            libqalculate
            nixfmt-rfc-style
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.openssl ];
          RUSTFLAGS = "--cfg reqwest_unstable";

          shellHook = ''
            export PORT=8081
            export ENABLE_HISTORY=1
            export SAVE_TO_DISK=1
            if [ -f .env ]; then
              set -a
              source .env
              set +a
              echo "Loaded environment variables from .env"
            fi
          '';
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
