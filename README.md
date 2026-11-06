# ScreenQR

codex --sandbox workspace-write --ask-for-approval on-request --add-dir "$env:USERPROFILE\.cargo" --add-dir "$env:USERPROFILE\.rustup" resume --last

A Windows 10/11 x64 desktop QR reader built with Tauri v2, React, strict TypeScript, Vite and Tailwind CSS v4.

## Current status

Milestones 1 and 2 are complete as of 2026-11-06, including user-confirmed multiple-QR desktop review. Milestone 2 provides a standalone Rust PNG/QR decoder and a bundled synthetic-image test panel. Run the desktop app, select a test image, then choose **Decode test image**. Results and errors return through scoped Tauri IPC; no link opens. The plain Vite browser preview cannot invoke Rust and displays a recoverable message.

Milestone 3 is in progress. Phase 3.1 implements one-shot primary-display capture with a temporary preview. Choose **Test screen capture**: ScreenQR hides, waits for repaint, captures once in memory and restores its window. Check the preview, then choose **Clear preview**. Screenshots are never written to disk. The user confirmed Phase 3.1 works after manual testing; region selection and screen QR decoding are not implemented yet.

Region selection, arbitrary image import, clipboard actions, browser opening, shortcuts, tray and history are not implemented. REQUIREMENTS.md is the product source of truth; P1/P2 are not scheduled. See [architecture and encoding limits](docs/ARCHITECTURE.md) and [validation](docs/TEST_PLAN.md).

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

The installed build loads bundled assets; Vite is development-only. Native commands decode allowlisted bundled fixtures or capture a temporary primary-display preview. Both are scoped to the local main window. No remote assets or scan persistence are added. Future P0 scanning targets only the primary display. Clipboard history/sync belongs to Windows, and explicit browser opening hands the destination to an external application.

Packaging is disabled in the foundation. The identifier `app.screenqr.desktop` is the development application identity; Store identity and publisher must be reconciled with Partner Center before packaging. Native launch, offline behavior, installation and OS compatibility require Windows verification; a frontend build does not establish them.

## Lint dependency maintenance

ESLint 10 is used with compatible TypeScript and React Hooks plugins. A temporary `minimatch: 11.0.1` npm override addresses [GHSA-g3hv-75wg-m249](https://github.com/isaacs/minimatch/security/advisories/GHSA-g3hv-75wg-m249) in the lint dependency chain. Remove the override once upstream dependencies select a fixed version themselves, then rerun audit, lint and the TSX/ignore checks. The override does not ship in the application runtime. Retain both package-lock.json and src-tauri/Cargo.lock.

For multiple QR codes, choose **Two QR codes** and decode: select **QR code 1** or **QR code 2** to inspect each result. Results are ordered by top edge then left edge, duplicates stay separate, and readable results remain available when other detected codes fail. URL destinations belong to the selected result and never open automatically.
