{
  description = "A Discord bot made for my discord server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustc-dev"
            "llvm-tools-preview"
          ];
        };
      in
      {
        packages = {
          default = self.packages.${system}.lowestbins;

          lowestbins = pkgs.rustPlatform.buildRustPackage {
            pname = "lowestbins";
            version = "1.4.0";

            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustToolchain
              curl
              nukeReferences
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            RUSTFLAGS = "--cfg reqwest_unstable";

            # cargo embeds absolute /nix/store/...-rust-default-X paths
            # for the rust-stdlib source files (panic-info, file!() macros)
            # into release binaries. The nix runtime-reference scanner
            # picks those up and pulls the whole nightly rust toolchain
            # (~4 GiB) into any closure that contains lowestbins. Replace
            # those refs with placeholder paths; keep only the libs we
            # actually link to at runtime.
            postFixup = ''
              for f in $out/bin/*; do
                nuke-refs \
                  -e ${pkgs.glibc} \
                  -e ${pkgs.gcc-unwrapped.lib} \
                  -e ${pkgs.openssl.out} \
                  "$f"
              done
            '';

            postInstall = null;
          };
        };

        devShells.default =
          with pkgs;
          mkShell {
            buildInputs = [
              openssl
              pkg-config
              eza
              fd
              clang
              mold
              rustToolchain
              ffmpeg
              libqalculate
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath [ openssl ];

            shellHook = ''
              export RUSTFLAGS='--cfg reqwest_unstable'
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
      }
    );
}
