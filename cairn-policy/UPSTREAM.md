# Cairn policy schema source

`contracts.ncl` and the initial complete field set in `default.ncl` were bootstrapped from `github.com/OnixResearch/cairn` revision `7e9ed636203395b3808a65962f6bb6da60f57268` on 2026-07-13. Replay cases unavailable in the published release CLI were removed, and final compatibility is checked against the pinned release revision `a22ea2bff65f16abec4f0f7ba2d7ddc14dc35871`.

`default.ncl` is project-owned after import: its project name, traceability roots, claim boundaries, and accepted external policy identities are maintained here. Refreshes must compare against a pinned Cairn revision, preserve local policy fields deliberately, regenerate `generated/cairn-policy.json`, and run `cairn policy export --check` plus `cairn validate`.
