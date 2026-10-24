# Security audit — PoE2 Campaign Copilot 0.2.0-alpha.1

Historical findings below describe alpha.1. For remediation in alpha.2, see
[the security update](SECURITY-UPDATE-alpha2.md).

Audit date: October 24, 2026. Scope: current tracked and untracked source,
including the PoE2 conversion, dependency lockfiles, Tauri/native commands,
frontend, file/log processing, pinned guide importer and GitHub workflows.
Base HEAD: `4940ae59044770222d72edd71ea3858d5ffb8f58`; the audited code also
includes the uncommitted preview changes. All files in the previously
provided source ZIP were byte-for-byte identical to the audit working tree.
No application fixes were made during this audit.

## Assessment

Two priorities before wider distribution or routine source-based use are
updating the development toolchain's vulnerable dependencies, cleaning up
flagged retained Rust dependencies, and removing
private diagnostics from outgoing bug-report URLs. Several lower-severity
input/resource and privilege-minimization issues also need attention.

No application-level remote-code-execution chain was demonstrated. Registry
severity labels, including critical transitive development advisories, are
not a claim that this packaged overlay has a remotely exploitable critical
vulnerability. Windows runtime/installer penetration testing was not possible
on this Linux host; this is a source/dependency audit with targeted benign
probes, not a certification or a full dependency implementation review.

## Findings

| ID | Severity / class | Finding | Evidence |
| --- | --- | --- | --- |
| S01 | High priority; development dependency risk | Known advisories in locked Vite/Vitest tooling | Live npm audit; exact lock versions. |
| S02 | Medium; confirmed privacy defect | Diagnostics transmitted in bug-report URL before review | Executed a pure URL-generation probe with a fake private path. |
| S03 | Low; confirmed input-boundary defect | Oversized partial log records lose framing; completed records lack the partial-buffer limit | Executed two bounded assembler probes. |
| S04 | Low; resource exhaustion | No overall live queue/journal/session-cache retention bound | Source review; no destructive load test. |
| S05 | Low; defense in depth | Renderer retains unused native privileges | Capability and plugin implementation review; no XSS demonstrated. |
| S06 | Medium; conditional supply-chain exposure | Inherited release jobs expose signing secrets to dependency-install/build steps | Workflow review; no secrets read and no compromise demonstrated. |
| S07 | Medium priority; latent Rust dependency risk | 11 RustSec lockfile matches, largely behind disabled updater/import paths | Current RustSec database, Windows dependency and feature trees, source review. |

### S01 — vulnerable development toolchain

Location: `package.json`; `package-lock.json:3162` (tinypool), `:3283`
(Vite), `:3381` (Vitest). `npm audit --json` reports **17 affected package
entries: 2 critical, 12 high, 2 moderate, 1 low**. These are package entries,
including inherited/transitive effects, not 17 independent demonstrated
application exploits. `npm audit --omit=dev --json` reports **zero** affected
production npm package entries. This does not assess Rust or WebView2.

Important exact matches:

- Vite **6.4.3**: Windows launch-editor command/argument handling advisories,
  including [GHSA-587h-7wmj-h5pm](https://github.com/advisories/GHSA-587h-7wmj-h5pm),
  [GHSA-698v-4r3x-3vcv](https://github.com/advisories/GHSA-698v-4r3x-3vcv)
  and [GHSA-r825-w8mm-5pxw](https://github.com/advisories/GHSA-r825-w8mm-5pxw).
  The installed Vite code registers `/__open-in-editor`; that middleware
  is relevant to the documented `npm run tauri dev` workflow. Exploitation
  requires the advisory-specific inputs/editor/environment and access to
  the development server; Windows command execution was not attempted.
- Vitest **3.2.7** / `@vitest/mocker` **3.2.7**:
  [GHSA-82fw-gwwq-j7x9](https://github.com/advisories/GHSA-82fw-gwwq-j7x9)
  and [GHSA-qxg6-89mg-rqv8](https://github.com/advisories/GHSA-qxg6-89mg-rqv8).
  The repository runs `vitest run`, not an intentionally exposed test UI.
- Tinypool **1.1.1**:
  [GHSA-5gmw-xhrv-c9v3](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3)
  and [GHSA-85c8-ppgw-ccpr](https://github.com/advisories/GHSA-85c8-ppgw-ccpr).
  These critical registry advisories describe prototype-pollution gadgets;
  an attacker-controlled pollution/input route in this project was not
  demonstrated. Do not equate them with log-to-native-code execution.
- Other direct advisory matches: Browserslist **4.28.6**, PostCSS **8.5.20**,
  source-map-js **1.2.1**, nanoid **3.3.16**, and
  baseline-browser-mapping **2.10.44**. Raw npm evidence records their
  advisory URLs/ranges and inherited Babel/plugin effects.

Impact: developer/build/test machine exposure under the applicable advisory
conditions. Vite and test-runner code are not bundled as a running Node
server in the installed static Tauri UI. The dev server uses Vite's default
loopback binding; do not expose it with `--host 0.0.0.0` as a workaround.

Remediation: deliberately upgrade Vite, its React plugin and Vitest to
compatible maintained versions that clear the current advisories; refresh
transitives and lockfiles. Some current major-version constraints do not
have an in-range fix according to the audit response, so do not blindly run
`npm audit fix --force`. Validate Node engine requirements, both lockfiles'
intended changes, 120 frontend tests, typecheck/build and the Tauri dev path.
Use the latest advisory database when selecting versions; a particular
major upgrade alone is not evidence that all findings are resolved.

### S02 — diagnostic disclosure occurs before issue-form consent

Locations: `src-tauri/src/bug_report.rs:40–49` and
`src-tauri/src/main.rs:854–868`. The URL contains
`https://github.com/.../issues/new?...&details=<diagnostic text>`, and the
tray command opens it immediately. HTTPS protects transport from ordinary
network observers but still sends the query to GitHub when navigating;
it can also persist in browser history. Editing the rendered issue form
cannot retract that request. Comments claiming nothing is transmitted until
Submit are incorrect. This is not public issue publication by itself.

Prerequisite: user clicks Report a bug and the diagnostics tail contains
sensitive text, such as a local path/Windows username from an error.
The benign probe confirmed that a fake `AuditExample` username and private
path become URL-query contents. No real diagnostics were uploaded.

Remediation: open a plain issue URL. Prepare diagnostics locally in a review
window or a redacted export, then let the user explicitly copy/attach them.
Never encode unreviewed diagnostics into a navigation URL. Correct the
consent comments and test that private strings never appear in URLs.

### S03 — line length handling does not preserve record boundaries

Location: `crates/input_log/src/assembler.rs:19–36`.

- A complete newline-terminated **128 KiB** record is returned despite the
  separate 64 KiB partial-buffer limit: the check occurs after draining
  complete lines. The current cap only promises to bound retained partial
  data; it does not enforce a full-record size limit. The probe demonstrated this with a bounded in-memory input.
- After an overlong partial record is cleared, its next chunk is treated
  as the beginning of a new record, even if no newline separated them.
  The second probe demonstrated a plausible log header in the suffix being
  returned as a standalone record. Depending on follow-on events this can
  create misleading tracking/timer/journal input.

A controlled/corrupt selected log is required. Ordinary PoE2 chat was not
shown capable of injecting raw Client.txt records. The live poller reads at
most 1 MiB per poll; this finding is not proof of unlimited single-record
allocation or remote code execution.

Remediation: enforce length while scanning complete and partial records;
when a record exceeds the cap, enter a discard-until-newline state. Bound
both memory and record size. Test oversized complete records, partial
records spanning polls, CRLF, and a valid-looking discarded suffix.

### S04 — retention is bounded per read, not for the whole live session

Locations: `src-tauri/src/main.rs:769` (unbounded mpsc channel),
`crates/input_log/src/tailer.rs:44–56`,
`src-tauri/src/journal.rs:176–189`, and
`crates/session/src/lib.rs:254–255`.

The producer can queue records without backpressure while the consumer does
file I/O/parsing/UI work. The session journal appends without an on-write
size cap; compaction only runs on application startup. The per-area seed
map retains every distinct observed ID until a session reset. The 1 MiB
poll limit, journal read cap and bounded pending-scene list do not bound
these accumulators. New PoE2 connection/scene records also count as journal
significant data, increasing retained data and including raw endpoints.

Prerequisite: sufficiently high-volume/prolonged input or an attacker-
controlled selected file. The expected normal game's finite area IDs
reduce seed-cache risk; a normal-play exhaustion was not demonstrated.

Remediation: use a bounded queue with cancellation-aware backpressure;
rotate/compact journals during writes; cap session history and individual
fields, evicting stale unknown IDs. Prefer normalized journal events that
do not retain server endpoints/unused raw message data. Test bounded loads
without filling a real disk, and test that tailer shutdown cannot deadlock
behind a full queue. Separately, stopping the producer does not join the
consumer; tag queued records with a generation or join both before changing
log/config so old buffered data cannot enter a new session.

### S05 — reduce renderer privileges to current product needs

Location: `src-tauri/capabilities/default.json:4–18`.
Both overlay and settings can invoke updater operations, process restart,
URL opening and dialogs, despite the preview removing update/import UI.
The opener's `allow-default-urls` scope permits arbitrary HTTP/HTTPS,
mailto and tel URLs. A renderer compromise could use these as additional
native side effects. Custom application commands also need review per
window rather than relying on hidden UI controls for authorization.

This is defense in depth: no untrusted HTML execution/XSS was found.
Importantly, `opener:allow-open-path` alone is **not** evidence of arbitrary
file execution: the installed plugin additionally enforces path scopes,
and this capability does not define a broad path allowlist. Empty updater
endpoints also mean the retained public key does not currently provide an
attacker-controlled update path.

Remediation: remove unused updater/process/opener permissions and plugins
from the preview, split overlay/settings capabilities, keep filesystem and
browser operations behind minimal fixed-destination Rust commands, and
verify actual denials in a Windows WebView. Do not broadly loosen scopes
to make a test pass.

### S06 — signing credentials are available during dependency execution

Location: `.github/workflows/release.yml:9–10,19–37` and the equivalent
Linux job. Signing key requirements are at **job** environment scope, so
`npm ci`, third-party setup/cache actions and the full native build inherit
these credentials if configured. The workflow-wide token has
`contents: write`, including build jobs. A compromised dependency lifecycle
script, build script or mutable action could steal signing material or
abuse the repository token. The problem is unnecessary blast radius, not
proof that a dependency/action or key is compromised.

Remediation: make build jobs read-only, grant write only to the protected
publication job, remove signing secrets from installation steps and ideally
isolate signing from dependency execution. Pin actions to reviewed full
commit SHAs and protect release environments/tags. The inherited workflow
also still names the original upstream release URL and expects updater
signatures while this preview disables updater artifacts; replace/disable
that release flow before using it for PoE2. No release workflow was run.

### S07 — Rust advisory matches need cleanup and reachability tracking

`cargo-audit 0.22.2` reports **11 vulnerability matches**, plus **9
unmaintained** and **3 unsoundness** warnings, using RustSec database commit
`21d909e4c8bd4e14c7aea58bf980c881a0053ab1` (updated October 24, 2026).
A nonzero audit exit is expected for these findings; the scan completed,
it did not fail to fetch/parse its database. No advisories were suppressed.

| Locked dependency | Advisory IDs | Patch information from the database | Preview applicability |
| --- | --- | --- | --- |
| quick-xml 0.37.5 | RUSTSEC-2026-0194, -0195 | >=0.41.0 | Old version is used by legacy pob_import. The native import command rejects input before parsing; plain Reader is used, not the NsReader path affected by -0195. Attribute parsing would still need review before re-enabling imports. A separate patched quick-xml 0.41.0 also exists in the lockfile. |
| rustls 0.23.42 | RUSTSEC-2026-0285, -0346, -0348, -0349 | >=0.23.46 covers the listed matches | Windows normal dependency tree reaches it through the retained tauri-plugin-updater/reqwest stack. Current preview has no configured update endpoints or frontend checks. Brotli is not enabled in the inspected Windows feature tree, excluding the -0348 decompressor path. No custom session-secret Debug logger was found for -0346. Other TLS findings should still be fixed before networking is enabled. |
| rustls-webpki 0.103.13 | RUSTSEC-2026-0382 | >=0.103.16 within the current stable series | Present in Windows dependency tree; distrust handling depends on the actual trust/verifier path. No attacker-controlled TLS connection or certificate bypass was demonstrated in the disabled updater. |
| webpki-root-certs 1.0.9 | RUSTSEC-2026-0383 | >=1.0.10 | Lockfile match, absent from the inspected Windows normal dependency tree. Do not misreport this as an active Windows root-store finding. |
| zip 4.6.1 | RUSTSEC-2026-0367, -0368, -0369 | >=9.0.0 per the advisory database; likely requires an upstream updater change | Retained updater has a Windows ZIP extraction branch calling ZipArchive::extract. Updates are disabled; downloaded updates also require signature verification. Unix SUID/SGID and Unix chmod findings are not Windows privilege escalation. The reported symlink demonstrations are macOS/Linux; no Windows exploit is claimed. |

The warnings are: unmaintained proc-macro-error, unic-char-property,
unic-char-range, unic-common, unic-ucd-ident, unic-ucd-version, zbus_names,
zvariant and zvariant_derive; unsound event-listener, glib and tokio APIs.
Tokio's flagged AsyncFd API is Unix-only, and glib is outside the selected
Windows runtime. An unmaintained warning is maintenance risk, not proof of
an exploitable vulnerability. Raw JSON preserves all IDs and conditions.

Remediation: remove unused updater/build-import dependencies/plugins where
possible; otherwise update affected compatible crates and the upstream
Tauri plugin that constrains ZIP. Do not force an incompatible ZIP major
through a Cargo patch or disable TLS/signature verification. Re-run the
full lockfile scan and Windows target/feature analysis, plus compile/tests,
before enabling networking/imports. Document remaining platform/API-specific
exceptions individually, with evidence, rather than globally ignoring
advisories. This cleanup is not performed by this audit.

## Positive controls verified by source review

- PoE2 UI renders log/guide text as React text, not `dangerouslySetInnerHTML`;
  the route importer parses pinned HTML as data without running its scripts.
- PoE2 build import command returns unsupported without decoding user XML;
  legacy PoB code is not an active production import route.
- Runtime PoE2 data is embedded; disabled layout output does not derive
  filesystem paths from uploaded log IDs. The retained image helper also
  restricts filenames to one normal path component.
- Production CSP restricts default sources to self and does not permit
  inline scripts; development-mode server risk remains separate.
- Update endpoints are empty and the settings container does not initiate
  update checks. Local settings/journals use a separate PoE2 namespace.
- Log polling and several persisted-file reads have size limits. The lone
  unsafe block sets Linux GDK_BACKEND before application threads start;
  no handwritten unsafe memory manipulation was found in project code.
- A targeted common private-key/GitHub-token/AWS-key pattern scan found no
  matches in the inspected source/config text. This is not proof that all
  possible credential forms or historical commits are secret-free.

## Additional hardening / inactive code

The config/timer readers check metadata size before an uncapped read
(`config.rs:180–189`, `run_timer.rs:119–130`); a concurrent same-user writer
can grow/replace a file after that check. Prefer `take(limit + 1)` on the
opened handle. This is a local robustness gap, not an established new
privilege escalation. Fixed-path settings/journal writes should also use
atomic replacement and preserve Windows ACLs.

The legacy PoB decoder checks trimmed input length but returns the original
raw XML string, allowing very large leading/trailing whitespace to evade
the intended raw-size bound. Check `code.len()` before trimming if this
library is reused. It is not reachable via the preview's disabled import
command. No external XML entity/file/network resolver was found in the
legacy pull-parser code.

## Reproduction and evidence

Benign probes are in [security/audit-probes.rs](security/audit-probes.rs).
From the repository root, compile with stable Rust:

```text
rustc --edition=2024 -Awarnings docs/security/audit-probes.rs -o <temporary-output-path>
```

Run that executable; it asserts the URL-disclosure and two framing/cap
behaviors with fake data and bounded buffers. It does not launch a browser,
send diagnostics, exhaust memory or fill disk. Review source before running.

Dependency scan commands:

```text
npm audit --json
npm audit --omit=dev --json
cargo audit --json
cargo tree --locked --target x86_64-pc-windows-msvc --edges normal
```

Audit tools and database snapshots were installed outside the repository.
Raw npm and Rust audit results and the Windows dependency/feature trees
are saved with the downloadable audit evidence. No application
patch, dependency upgrade, release, push or Windows penetration test was
performed as part of this audit.
