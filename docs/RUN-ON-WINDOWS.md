# Run PoE2 Campaign Copilot on Windows

These instructions are for **0.2.0-alpha.2**, an Act 1 development preview.
It has not yet been validated on a Windows gaming PC. The full campaign,
build imports, diagrams and automatic updates are not available yet.

## 1. Install the tools once

On a normal Windows 10/11 x64 PC, install:

1. [Node.js LTS](https://nodejs.org/en/download) — version 24 LTS is recommended (the latest 22 LTS, at least
   22.12.0, is also supported). Include npm and the option to add Node to PATH.
2. [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
   In its installer, select **Desktop development with C++**, including
   the MSVC x64/x86 build tools and a Windows 10 or Windows 11 SDK.
   The full Visual Studio editor is not required.
3. [Rust through rustup](https://rustup.rs). Use the default stable Windows
   **MSVC** toolchain, not GNU. Keep its PATH integration enabled.
4. [Microsoft Edge WebView2 Evergreen Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
   if it is not already installed. Many Windows PCs already have it.

Close and reopen PowerShell after installation. Check:

```powershell
node --version
npm.cmd --version
rustc --version
cargo --version
```

All four should print versions. Python and Git are not required to run the
provided source archive. Initial dependency downloads/builds need Internet
access; gameplay tracking does not need a key or account connection.

## 2. Get the preview source

Install [Git for Windows](https://git-scm.com/download/win), reopen PowerShell,
and clone the preview branch:

```powershell
git clone --branch codex/poe2-preview-alpha2 --single-branch https://github.com/cboveda/poe-campaign-copilot.git
Set-Location .\poe-campaign-copilot
```

For a private repository, authenticate through Git's normal browser flow
using a GitHub account with repository access. Do not put tokens into URLs.
Use this preview branch rather than the PoE1 `main` branch.

Alternatively, on GitHub select `codex/poe2-preview-alpha2`, then choose
**Code → Download ZIP**, or use the supplied alpha.2 source ZIP if available.

Right-click the ZIP → **Extract All**. Open the extracted project folder
containing `package.json`, `Cargo.toml`, `src` and `src-tauri`. Click File
Explorer's address bar, type `powershell`, and press Enter. This opens a
terminal in the correct folder. Keep the project somewhere writable, such
as your Documents folder; avoid Program Files or a network share.

Confirm that this is the preview:

```powershell
(Get-Content .\package.json -Raw | ConvertFrom-Json).version
```

Expected: `0.2.0-alpha.2`.

## 3. Install dependencies and launch

In that PowerShell window:

```powershell
npm.cmd ci
npm.cmd run tauri dev
```

The first native build can take several minutes and downloads Rust
packages. Wait for the app to launch; leave the terminal running. Later
launches use the same `npm.cmd run tauri dev` command and reuse the build.

Expected: a **PoE2 Campaign Copilot** tray icon and an overlay initially
reading **Waiting for Client.txt…**. Check Windows' hidden tray icons if
necessary. The overlay intentionally does not have a normal taskbar button.

## 4. Point it at PoE2

1. Start PoE2 using the **English client** and **windowed fullscreen**.
2. Find the game's installation folder. For Steam: Library → right-click
   Path of Exile 2 → Manage → Browse local files.
3. Inside it, locate `logs\Client.txt` — use the real PoE2 log, not the
   sample uploaded for development or the PoE1 log.
4. Right-click the Copilot tray icon → **Settings** → **Browse** and select
   that file. The selection saves automatically.
5. Enter another area in PoE2. The app starts reading at the end of the log,
   so it does not replay your earlier travels on first launch.

Expected: the overlay shows the area you entered. Supported Act 1 zones
also show advisory route objectives and an **Act 1 preview** note. Other
areas show their location with guidance unavailable. Entering an area does
not confirm that you killed its boss or collected its rewards.

## 5. Basic controls

| Shortcut | Action |
| --- | --- |
| Alt+Shift+O | Open settings |
| Alt+Shift+S | Toggle setup mode to move/resize the overlay |
| Alt+Shift+H | Hide/show the overlay |
| Alt+Shift+C | Toggle compact view |
| Alt+Shift+T | Pause/resume the timer |

Turn setup mode off after positioning it so clicks pass through to the
game. Tray → **Quit** stops the app. When running from source, you can also
stop the development process with Ctrl+C in its PowerShell window.

PoE1 settings are separate. Login/process changes intentionally start a new
route session; per-character route resume across game sessions is not yet
implemented. Reset progress is available for intentionally starting a new run.

## Optional: make a local installer

After confirming the development preview starts, stop it and run:

```powershell
npm.cmd run tauri build -- --bundles nsis
Get-ChildItem .\target\release\bundle\nsis\*.exe
```

Run the generated setup executable. It installs for your Windows user and
lets you launch the app without a development terminal. This is a local
preview installer, not a published/signed release. Automatic updates remain
disabled. Installer creation and installation still need Windows validation.

## Bug reports and diagnostics

Tray → **Report a bug** opens a plain issue form. No diagnostics are read
or attached to its URL. Use **Open logs** to review/redact a file locally,
then attach it manually if you choose to share it.

## If something fails

- **`npm.ps1` is blocked:** use `npm.cmd` exactly as above; no PowerShell
  execution-policy change is necessary.
- **`node`, `npm.cmd` or `cargo` is not found:** reopen PowerShell after the
  installers finish and check the tools were added to PATH.
- **`link.exe` or Windows SDK errors:** modify Visual Studio Build Tools
  to include Desktop development with C++ and a Windows SDK, then reopen
  the terminal. Run `rustup show` and confirm the active host is MSVC.
- **WebView2 missing / blank window:** install or repair the Evergreen
  Runtime, then restart the app. Report persistent blank-window failures.
- **Port 1420 is already in use:** stop the previous Copilot/Vite process
  you started before launching another development session.
- **Still waiting after traveling:** confirm Settings points to the PoE2
  log that is actually growing. If you have previously set developer log
  overrides, start a fresh terminal without them or remove only
  `POE_COPILOT_LOG` and `POE_COPILOT_LOG_REPLAY` before launching.
- **No overlay:** check the tray, press Alt+Shift+H once, and confirm
  windowed fullscreen rather than exclusive fullscreen.

If a build fails, send the final error block from PowerShell. For an
in-game tracking issue, use [the short validation checklist](POE2-WINDOWS-VALIDATION.md)
and include the game patch, failed step and a minimal sanitized log excerpt.
