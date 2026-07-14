{
  # r[impl nickel_export.release.profile]
  description = "Evaluator-neutral deterministic Nickel export identities and receipts";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    cairn = {
      url = "github:onixresearch/cairn/a22ea2bff65f16abec4f0f7ba2d7ddc14dc35871";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay, cairn }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem = function: nixpkgs.lib.genAttrs systems (system: function system);
    in
    {
      packages = eachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
        in
        {
          nickel-export = rustPlatform.buildRustPackage {
            pname = "nickel-export";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [
              "-p"
              "nickel-export"
            ];
            cargoTestFlags = [ "--workspace" ];
            postInstall = ''
              install -Dm644 config/generated/repository.json "$out/share/nickel-export/repository.json"
              install -Dm644 config/generated/resource-limits.json "$out/share/nickel-export/resource-limits.json"
              install -Dm644 release/generated/profile.json "$out/share/nickel-export/release-profile.json"
              install -Dm644 release/evidence.md "$out/share/doc/nickel-export/release-evidence.md"
              install -Dm644 README.md "$out/share/doc/nickel-export/README.md"
              install -Dm644 docs/schemas.md "$out/share/doc/nickel-export/schemas.md"
              install -Dm644 docs/examples.md "$out/share/doc/nickel-export/examples.md"
              install -Dm644 docs/migration.md "$out/share/doc/nickel-export/migration.md"
              install -Dm644 LICENSE "$out/share/licenses/nickel-export/LICENSE"
            '';
          };
          default = self.packages.${system}.nickel-export;
        }
      );

      checks = eachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = toolchain;
            rustc = toolchain;
          };
          coreCheck =
            name: target:
            rustPlatform.buildRustPackage {
              pname = name;
              version = "0.1.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [
                "-p"
                "nickel-export-core"
                "--no-default-features"
              ] ++ pkgs.lib.optionals (target != null) [
                "--target"
                target
              ];
              doCheck = false;
              installPhase = ''
                touch "$out"
              '';
            };
        in
        {
          cargo-test = rustPlatform.buildRustPackage {
            pname = "nickel-export-tests";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "--workspace" ];
            cargoTestFlags = [ "--workspace" ];
          };

          cargo-clippy = rustPlatform.buildRustPackage {
            pname = "nickel-export-clippy";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            buildPhase = ''
              cargo clippy --workspace --all-targets --offline -- -D warnings
            '';
            doCheck = false;
            installPhase = ''
              touch "$out"
            '';
          };

          cargo-format = pkgs.runCommand "nickel-export-format" {
            nativeBuildInputs = [ toolchain ];
            src = self;
          } ''
            set -eu
            cd "$src"
            cargo fmt --all -- --check
            cargo fmt --manifest-path fuzz/Cargo.toml -- --check
            touch "$out"
          '';

          fuzz-target = rustPlatform.buildRustPackage {
            pname = "nickel-export-fuzz-target";
            version = "0.1.0";
            src = self;
            cargoRoot = "fuzz";
            cargoLock.lockFile = ./fuzz/Cargo.lock;
            doCheck = false;
            installPhase = ''
              touch "$out"
            '';
          };

          core-no-std-host = coreCheck "nickel-export-core-no-std-host" null;
          core-no-std-wasm = coreCheck "nickel-export-core-no-std-wasm" "wasm32-unknown-unknown";

          cairn-policy = pkgs.runCommand "nickel-export-cairn-policy" {
            nativeBuildInputs = [ cairn.packages.${system}.cairn ];
            src = self;
          } ''
            set -eu
            cairn policy export \
              --source "$src/cairn-policy/default.ncl" \
              --output "$TMPDIR/cairn-policy.json"
            cmp "$TMPDIR/cairn-policy.json" "$src/cairn-policy/generated/cairn-policy.json"
            cairn validate \
              --root "$src" \
              --policy "$src/cairn-policy/generated/cairn-policy.json"
            cairn traceability coverage \
              --root "$src" \
              --policy "$src/cairn-policy/generated/cairn-policy.json" \
              --profile nickel-export-default \
              --json > "$TMPDIR/traceability.json"
            touch "$out"
          '';

          nickel-contracts = pkgs.runCommand "nickel-export-contracts" {
            nativeBuildInputs = [ pkgs.nickel ];
            src = self;
          } ''
            set -eu
            cd "$src"
            nickel typecheck config/repository.ncl
            nickel typecheck config/resource-limits.ncl
            nickel typecheck release/profile.ncl
            nickel export --format json config/repository.ncl > "$TMPDIR/repository.json"
            nickel export --format json config/resource-limits.ncl > "$TMPDIR/resource-limits.json"
            nickel export --format json release/profile.ncl > "$TMPDIR/release-profile.json"
            cmp "$TMPDIR/repository.json" config/generated/repository.json
            cmp "$TMPDIR/resource-limits.json" config/generated/resource-limits.json
            cmp "$TMPDIR/release-profile.json" release/generated/profile.json
            touch "$out"
          '';

          cli-e2e = pkgs.runCommand "nickel-export-cli-e2e" {
            nativeBuildInputs = [
              self.packages.${system}.nickel-export
              pkgs.nickel
            ];
            src = self;
          } ''
            set -eu
            cp -R --no-preserve=mode "$src" work
            cd work
            for format in json toml yaml raw; do
              nickel-export export \
                --spec "fixtures/requests/$format.json" \
                --root . \
                --evaluator "${pkgs.nickel}/bin/nickel" \
                --evaluator-identity nixpkgs:nickel \
                --evaluator-version nickel-lang-cli-1.17.0 \
                --manifest "fixtures/generated/$format.manifest.json" \
                --check > "$TMPDIR/$format.receipt.json"
            done
            nickel-export export \
              --spec examples/service-config/request.json \
              --root . \
              --evaluator "${pkgs.nickel}/bin/nickel" \
              --evaluator-identity nixpkgs:nickel \
              --evaluator-version nickel-lang-cli-1.17.0 \
              --manifest examples/service-config/generated/manifest.json \
              --check > "$TMPDIR/service-config.receipt.json"
            nickel-export verify \
              --manifest examples/service-config/generated/manifest.json \
              --root . \
              --check-artifacts > "$TMPDIR/service-config.integrity.json"
            if nickel-export export \
              --spec fixtures/requests/unsafe.json \
              --root . \
              --evaluator "${pkgs.nickel}/bin/nickel" \
              --evaluator-identity nixpkgs:nickel \
              --evaluator-version nickel-lang-cli-1.17.0 \
              --manifest fixtures/generated/unsafe.manifest.json \
              --check > /dev/null 2>&1; then
              echo "unsafe fixture unexpectedly passed" >&2
              exit 1
            fi
            if nickel-export export \
              --spec fixtures/requests/json.json \
              --root . \
              --evaluator "${pkgs.nickel}/bin/nickel" \
              --evaluator-identity nixpkgs:nickel \
              --evaluator-version nickel-lang-cli-0.0.0-invalid \
              --manifest fixtures/generated/json.manifest.json \
              --check > /dev/null 2>&1; then
              echo "mismatched evaluator version unexpectedly passed" >&2
              exit 1
            fi
            cp fixtures/generated/config.json "$TMPDIR/config.json"
            printf '\n' >> fixtures/generated/config.json
            if nickel-export export \
              --spec fixtures/requests/json.json \
              --root . \
              --evaluator "${pkgs.nickel}/bin/nickel" \
              --evaluator-identity nixpkgs:nickel \
              --evaluator-version nickel-lang-cli-1.17.0 \
              --manifest fixtures/generated/json.manifest.json \
              --check > /dev/null 2>&1; then
              echo "tampered output unexpectedly passed freshness check" >&2
              exit 1
            fi
            cmp "$TMPDIR/config.json" "$src/fixtures/generated/config.json"
            touch "$out"
          '';
        }
      );

      devShells = eachSystem (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          default = pkgs.mkShell {
            packages = [
              toolchain
              pkgs.nickel
              pkgs.nixfmt
            ];
          };
        }
      );
    };
}
