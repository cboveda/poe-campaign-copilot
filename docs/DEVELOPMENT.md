# Developing the PoE2 preview

Use the existing checkout. The app is PoE2-only; retained PoE1 fixtures test
shared engine behavior and do not enable a PoE1 product mode.

## Windows

For first-time PC setup, follow [Run on Windows](RUN-ON-WINDOWS.md).

Install Node.js 24 LTS (or Node 22.12+ within the 22 LTS series), stable Rust through rustup, and the Visual Studio C++
build tools with a Windows SDK. Tauri uses the Windows WebView2 runtime.
From the repository root, run:

```powershell
npm ci
npm run tauri dev
```

The current preview is 0.2.0-alpha.5. Use the source containing the PoE2
changes; an older main checkout is not equivalent. Select the real PoE2 log
from Settings. Normal tailing starts at EOF; enter a zone after starting it.

To build a local Windows test installer (not a published/signed release):

```powershell
npm run tauri build -- --bundles nsis
```

Workspace output is in `target/release/bundle/nsis/`. Updates are disabled
and the PoE2 identifier is distinct from the original application. Do not
publish using the inherited release workflow; release identity, signing,
Windows package validation and updater configuration remain separate work.

Run [the Windows checklist](POE2-WINDOWS-VALIDATION.md) and record actual
results. Unit/replay tests do not establish in-game overlay correctness.

## Automated checks

From the repository root:

```text
python tools/import_poe2_campaign.py --check
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

The sanitized PoE2 fixture contains independently annotated expected output
for 206 transitions. To inspect a log without a GUI:

```text
cargo run --locked -p replay --bin track-poe2 -- crates/replay/fixtures/poe2-sample.log
```

Output contains observed location IDs/names and new/revisited instance flags;
it does not print character/chat contents. Use your own file path to inspect
another English log. Do not commit full private logs.

## Cloud Linux development

Activate the prepared local tools/libraries in each shell:

```sh
. /workspace/.poe-setup/env.sh
cd /workspace/poe-campaign-copilot
```

The shared Rust/frontend tests and native compile work here. GUI startup is
blocked by WebKit's fixed helper paths and container privileges. Linux GUI
support is outside the agreed scope; use Windows for gameplay checks.

## Route content

`content/poe2/observed-areas.json` is a partial observation catalog from the
supplied log. `act1-route.json` and `remaining-campaign.json` are generated
from pinned source in `vendor/poe2-leveling/`, with its MIT license. Run:

```text
python tools/import_poe2_campaign.py
python tools/import_poe2_campaign.py --check
```

The generator verifies source step counts and excludes the source endgame
section and explicit league-only Act 3 objectives. Source checksums use
UTF-8/LF normalization to remain reproducible across Windows checkouts.
It does not run embedded scripts or infer game IDs. See the
[coverage matrix](POE2-CAMPAIGN-COVERAGE.md) for grouping, source ambiguities,
synthetic-fixture limits and manual checks. Gameplay requires no network.

The original `compile-content` binary remains a PoE1 test/reference tool;
it does not generate or validate the PoE2 embedded preview. Do not use its
successful output as evidence of PoE2 campaign coverage.
