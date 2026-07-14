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
    octet = {
      url = "github:OnixResearch/octet?rev=374bd16b26cee2af34211a29bfa531c016811f51";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay, cairn, octet }:
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
          octetPackages = octet.packages.${system};
          octetProductionVerus = builtins.getAttr "octet-production-verus" octetPackages;
          octetVerusfmt = builtins.getAttr "octet-verusfmt" octetPackages;
          proofRlimit = 200;
          proofVerifiedObligations = 30;
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

          identity-proofs = pkgs.runCommand "nickel-export-identity-proofs" {
            nativeBuildInputs = [
              octetProductionVerus
              octetVerusfmt
              pkgs.b3sum
              pkgs.coreutils
              pkgs.diffutils
              pkgs.gnugrep
              pkgs.jq
              pkgs.nickel
            ];
            src = self;
          } ''
            set -euo pipefail
            proof_file="$src/proofs/identity_primitives.rs"
            invalid_fixture="$src/proofs/fixtures/invalid/ambiguous-prefix.rs"
            evidence="$src/proofs/generated/evidence.json"
            verifier_root=${octetProductionVerus}/libexec/verus

            octet-production-verus --identity > "$TMPDIR/verifier-identity.txt"
            grep -F 'proof-verifier: verus@0.2026.05.17.e479cce' "$TMPDIR/verifier-identity.txt" > /dev/null
            grep -F 'proof-verifier-source-revision: e479cce36490b8fa4b0fd7755aa742aec354372c' \
              "$TMPDIR/verifier-identity.txt" > /dev/null

            octet-production-verus \
              --triggers-mode silent \
              --rlimit ${toString proofRlimit} \
              --crate-type=lib \
              --extern vstd="$verifier_root/libvstd.rlib" \
              --extern builtin="$verifier_root/libverus_builtin.rlib" \
              --extern builtin_macros="$verifier_root/libverus_builtin_macros.so" \
              --extern state_machines_macros="$verifier_root/libverus_state_machines_macros.so" \
              -L "$verifier_root" \
              --edition 2021 \
              "$proof_file" 2>&1 | tee "$TMPDIR/proof.log"
            grep -F 'verification results:: ${toString proofVerifiedObligations} verified, 0 errors' \
              "$TMPDIR/proof.log" > /dev/null

            set +e
            octet-production-verus \
              --triggers-mode silent \
              --rlimit ${toString proofRlimit} \
              --crate-type=lib \
              --extern vstd="$verifier_root/libvstd.rlib" \
              --extern builtin="$verifier_root/libverus_builtin.rlib" \
              --extern builtin_macros="$verifier_root/libverus_builtin_macros.so" \
              --extern state_machines_macros="$verifier_root/libverus_state_machines_macros.so" \
              -L "$verifier_root" \
              --edition 2021 \
              "$invalid_fixture" > "$TMPDIR/invalid-proof.log" 2>&1
            invalid_status="$?"
            set -e
            if test "$invalid_status" -eq 0; then
              echo "invalid proof fixture unexpectedly verified" >&2
              exit 1
            fi
            grep -F 'postcondition not satisfied' "$TMPDIR/invalid-proof.log" > /dev/null
            echo "negative proof fixture rejected as expected"

            octet-verusfmt --check --verus-only "$proof_file"
            nickel typecheck "$src/proofs/evidence.ncl"
            nickel export --format json "$src/proofs/evidence.ncl" > "$TMPDIR/evidence.json"
            cmp "$TMPDIR/evidence.json" "$evidence"

            verify_artifact() {
              role="$1"
              count="$(jq --arg role "$role" '[.artifacts[] | select(.role == $role)] | length' "$evidence")"
              test "$count" -eq 1
              path="$(jq --raw-output --arg role "$role" '.artifacts[] | select(.role == $role) | .path' "$evidence")"
              expected_blake3="$(jq --raw-output --arg role "$role" '.artifacts[] | select(.role == $role) | .blake3' "$evidence")"
              expected_bytes="$(jq --raw-output --arg role "$role" '.artifacts[] | select(.role == $role) | .bytes' "$evidence")"
              actual_blake3="$(b3sum "$src/$path")"
              actual_blake3="''${actual_blake3%% *}"
              actual_bytes="$(wc -c < "$src/$path")"
              test "$actual_blake3" = "$expected_blake3"
              test "$actual_bytes" -eq "$expected_bytes"
            }

            verify_artifact proof-source
            verify_artifact implementation-source
            verify_artifact correspondence-vectors
            verify_artifact negative-proof-fixture
            touch "$out"
          '';

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
              pkgs.jq
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
            replay_runs="$(jq --raw-output '.replay.runs' release/generated/profile.json)"
            expected_replay_output_lines=2
            nickel-export export \
              --spec fixtures/requests/json.json \
              --root . \
              --evaluator "${pkgs.nickel}/bin/nickel" \
              --evaluator-identity nixpkgs:nickel \
              --evaluator-version nickel-lang-cli-1.17.0 \
              --manifest fixtures/generated/json.manifest.json \
              --replay-runs "$replay_runs" \
              --check > "$TMPDIR/fixtures-json-replay.jsonl"
            test "$(wc -l < "$TMPDIR/fixtures-json-replay.jsonl")" \
              -eq "$expected_replay_output_lines"
            grep -F '"schema":"onix-nickel-export-replay-report/v1"' \
              "$TMPDIR/fixtures-json-replay.jsonl" > /dev/null
            for replay_attempt in first second; do
              nickel-export export \
                --spec examples/service-config/request.json \
                --root . \
                --evaluator "${pkgs.nickel}/bin/nickel" \
                --evaluator-identity nixpkgs:nickel \
                --evaluator-version nickel-lang-cli-1.17.0 \
                --manifest examples/service-config/generated/manifest.json \
                --replay-runs "$replay_runs" \
                --check > "$TMPDIR/service-config-replay-$replay_attempt.jsonl"
              test "$(wc -l < "$TMPDIR/service-config-replay-$replay_attempt.jsonl")" \
                -eq "$expected_replay_output_lines"
              grep -F '"schema":"onix-nickel-export-replay-report/v1"' \
                "$TMPDIR/service-config-replay-$replay_attempt.jsonl" > /dev/null
            done
            cmp "$TMPDIR/service-config-replay-first.jsonl" \
              "$TMPDIR/service-config-replay-second.jsonl"
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
