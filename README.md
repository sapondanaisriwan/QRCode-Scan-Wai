# ScreenQR

codex --sandbox workspace-write --ask-for-approval on-request --add-dir "$env:USERPROFILE\.cargo" --add-dir "$env:USERPROFILE\.rustup" resume --last

A Windows 10/11 x64 desktop QR reader built with Tauri v2, React, strict TypeScript, Vite and Tailwind CSS v4.

## Current status

Milestones 1 and 2 are complete as of 2026-11-06, including user-confirmed multiple-QR desktop review. Milestone 2 provides a standalone Rust PNG/QR decoder and a bundled synthetic-image test panel. Run the desktop app, select a test image, then choose **Decode test image**. Results and errors return through scoped Tauri IPC; no link opens. The plain Vite browser preview cannot invoke Rust and displays a recoverable message.

The user confirmed the M3 capture -> select -> crop -> decode -> display flow works. The user confirmed M3.5 spatial review works. M4 Copy/Open is implemented but awaits automated and Windows verification. Choose **Scan screen**, select one or more QR codes, and inspect numbered cards on the frozen screenshot. Each marker/card refers to the same QR; dense layouts use a numbered scrollable list. **Close results**, **View in main window**, or Escape closes review and shows the same batch in the original main-window list. Escape before decoding completes cancels without a new result. Choose **Copy** for the full original text, or **Open URL** for the displayed HTTP(S) destination. Neither action runs automatically. After Open from an overlay, its results remain accessible from the taskbar while the browser can appear in front. The separate capture/fixture panels remain development diagnostics.

Arbitrary image import, shortcuts, tray and history are not implemented. REQUIREMENTS.md is the product source of truth; P1/P2 are not scheduled. See [architecture and encoding limits](docs/ARCHITECTURE.md) and [validation](docs/TEST_PLAN.md).

## Development

Install Node.js 22.13+ (22.x) or 24+ (Node 24 recommended), npm, Rust with the Windows MSVC toolchain, Microsoft C++ Build Tools with the Desktop development with C++ workload, and WebView2. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
npm.cmd ci
npm.cmd run dev
npm.cmd run tauri -- dev
```

`dev` previews only the frontend. `tauri -- dev` launches the native desktop application. Using `npm.cmd` avoids PowerShell script execution-policy restrictions. If the default npm cache is not writable, append `--cache .npm-cache` to installation commands.

## Validation

```powershell
npm.cmd test
npm.cmd run typecheck
npm.cmd run lint
npm.cmd run format:check
npm.cmd run build
npm.cmd run tauri -- build --no-bundle
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Use `npm.cmd run format` to format frontend/configuration/documentation files and `cargo fmt` for Rust. Source requirements and AGENTS.md are excluded from automated formatting. Rust fixture/URL/IPC tests and React state tests cover the M2 test surface. See src-tauri/tests/fixtures/README.md for fixture regeneration.

## Boundaries

The installed build loads bundled assets; Vite is development-only. Native commands decode allowlisted bundled fixtures or capture a temporary primary-display preview. Capture starts are scoped to the local main window; each overlay can read and finish only its active session. No remote assets or scan persistence are added. Future P0 scanning targets only the primary display. Clipboard history/sync belongs to Windows, and explicit browser opening hands the destination to an external application.

Packaging is disabled in the foundation. The identifier `app.screenqr.desktop` is the development application identity; Store identity and publisher must be reconciled with Partner Center before packaging. Native launch, offline behavior, installation and OS compatibility require Windows verification; a frontend build does not establish them.

## Lint dependency maintenance

ESLint 10 is used with compatible TypeScript and React Hooks plugins. A temporary `minimatch: 11.0.1` npm override addresses [GHSA-g3hv-75wg-m249](https://github.com/isaacs/minimatch/security/advisories/GHSA-g3hv-75wg-m249) in the lint dependency chain. Remove the override once upstream dependencies select a fixed version themselves, then rerun audit, lint and the TSX/ignore checks. The override does not ship in the application runtime. Retain both package-lock.json and src-tauri/Cargo.lock.

For multiple QR codes, choose **Two QR codes** and decode: select **QR code 1** or **QR code 2** to inspect each result. Results are ordered by top edge then left edge, duplicates stay separate, and readable results remain available when other detected codes fail. URL destinations belong to the selected result and never open automatically.

## Copy and Open privacy

Copy explicitly writes the original text to the Windows clipboard. Windows clipboard history/sync and the application receiving a paste may retain that text; ScreenQR does not control them. Embedded NUL text is rejected rather than silently truncated. If the clipboard is busy, keep the result and retry. Open URL passes only the displayed, revalidated HTTP(S) destination to the default browser, which may then contact that site. Scanning alone makes no such request. A new scan or fixture decode expires previous action IDs; old diagnostic results must be decoded again before acting.
