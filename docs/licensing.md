# Nickel Export licensing boundary

The typed source of truth is `config/repository.ncl`.

| Package | License |
|---|---|
| `nickel-export-core` | `MPL-2.0` |
| `nickel-export` | `AGPL-3.0-or-later` |

`nickel-export-core` is deterministic, evaluator-neutral, and `no_std` capable. It must not depend on the evaluator/file shell. The shell may depend on the core.

Complete texts are the root `LICENSE` for AGPL-3.0-or-later and `LICENSES/MPL-2.0.txt` for MPL-2.0. Each Cargo package directory carries the applicable text so package archives are self-contained. Dependencies and generated material containing upstream code retain their own terms.

Package licensing is distribution metadata and is outside canonical export, manifest, and receipt identity unless a versioned schema explicitly includes it. Prior grants remain valid. The split does not transfer evaluator, filesystem, process, or network authority into the core and does not establish legal compliance or release eligibility.
