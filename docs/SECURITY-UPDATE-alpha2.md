# Security update — 0.2.0-alpha.2

This update addresses the two issues prioritized before local source-based
Windows testing. It is not a claim that every finding in the historical
[alpha.1 audit](SECURITY-AUDIT.md) has been fixed.

## Changes

- Updated Vite to 8.3.5, its React plugin to 6.1.2 and Vitest to 5.0.5,
  refreshed the npm lockfile and removed the unused updater/process JS
  dependencies. Node 24 LTS is recommended; Node 22.12+ in the 22 LTS series
  is also supported. Vitest 5 needed an explicit function type for one
  existing resize-observer test mock; assertions/behavior were preserved.
- Report a bug now opens a fixed GitHub issue-form URL. The action does not
  read diagnostics or put diagnostics/version/path text in query parameters.
  Users can review/redact files via Open logs and attach them explicitly.
  Tests for the removed automatic prefill were replaced by the fixed-URL
  privacy invariant; this accounts for the Rust test-count change.
- Removed unused native updater/process plugins, their runtime dependencies,
  updater public-key configuration and related renderer permissions. Also
  removed direct renderer dialog/opener permissions: browsing and the fixed
  log/report actions continue through existing Rust commands. This reduces
  privileges without granting the renderer a broad file/URL scope.

## Verification

- A clean `npm ci` succeeded; 120 frontend tests and TypeScript/Vite build
  passed. A Vite 8 server served the HTML and transformed React entry.
- 234 Rust tests, formatting, Clippy with warnings denied, and Linux desktop
  compilation passed. No Windows build/gameplay execution is claimed.
- Fresh `npm audit --json`: **0 advisory matches**, including development
  dependencies (alpha.1 had 17 affected package entries).
- RustSec scan against the same October 24, 2026 database: **2 vulnerability
  matches**, down from 11. Both are quick-xml 0.37.5 through the retained
  legacy pob_import library. The PoE2 import command still rejects all
  imports before parsing. Nine unmaintained and three unsoundness warnings
  remain; see the original report for API/platform applicability. Do not
  enable build imports without fixing/reviewing that path.
- The benign privacy probe now verifies a fixed URL; the remaining framing
  probes still reproduce S03. No diagnostics were sent during validation.

## Findings still open

Malformed-log framing/retention (S03/S04), the inherited signing workflow
(S06), and complete per-window IPC/capability separation (remaining S05)
were not remediated in this update. Do not use the inherited release flow
for production. Use your own actual game log for testing, run the overlay
as your normal Windows user, and keep development-server access local.
These precautions do not constitute a fix for the remaining findings.

## Run the refreshed preview

Extract the new **0.2.0-alpha.2** ZIP into a fresh folder instead of mixing
it into the previous dependency/build directories. Follow
[Run on Windows](RUN-ON-WINDOWS.md), then the
[Windows validation checklist](POE2-WINDOWS-VALIDATION.md). Normal launch:

```powershell
npm.cmd ci
npm.cmd run tauri dev
```

The source ZIP is not a Windows executable and no installer was published.
The preview still covers Act 1 rather than the full campaign.
