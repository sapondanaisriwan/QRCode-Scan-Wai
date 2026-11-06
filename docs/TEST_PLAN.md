# Validation record

## Milestone 1: complete for development foundation

Verified 2026-11-06 on the current x64 Windows development machine, OS build 26300.9711 (DisplayVersion 26H2). Node 24.11.1, npm 11.6.2, Rust/Cargo 1.99.1. This is not Windows 10/11 clean-install certification.

| Check                                     | Result                                                                                                                                 |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| npm install with updated lockfile         | PASS; ESLint 10.14.0, @eslint/js 10.0.1; plugin peer ranges support ESLint 10                                                          |
| npm audit                                 | PASS; zero reported vulnerabilities after minimatch 11.0.1 override                                                                    |
| npm run lint and typecheck                | PASS                                                                                                                                   |
| Linter compatibility probe                | PASS; TSX parses, conditional-hook misuse triggers react-hooks/rules-of-hooks, dist files are ignored                                  |
| cargo fmt --check                         | PASS                                                                                                                                   |
| cargo clippy --all-targets -- -D warnings | PASS                                                                                                                                   |
| cargo test                                | PASS; zero tests in this foundation, not behavior coverage                                                                             |
| npm run tauri -- build --no-bundle        | PASS; includes strict TypeScript and Vite production build                                                                             |
| Release executable launch                 | PASS; launched src-tauri/target/release/screenqr.exe directly                                                                          |
| Bundled frontend                          | PASS; accessibility document URL is http://tauri.localhost/ and all expected foundation text is present; no listener on Vite port 1420 |
| Native Close                              | PASS; clicked Close, window disappeared and screenqr process was absent                                                                |

Native window verification used the Computer Use plugin. The screenshot helper returned an unrelated foreground window even after activation; pixel-level visual review is therefore unverified. Accessibility evidence confirms the title, main landmark, headings and complete placeholder text, but does not prove layout, contrast or scaling. The user separately reported successful manual development launch. No disconnected-network test, performance measurement, Windows-family compatibility claim or installer test is included in M1.

The temporary minimatch override resolves GHSA-g3hv-75wg-m249 in development lint tooling. Track removal when upstream ranges permit fixed versions. Runtime dependencies were not changed. Both lockfiles are present. No scanning features are implemented and no P1/P2 work was added.

## Subsequent milestones

Follow REQUIREMENTS.md T01-T07 for P0. Add automated decoder, URL, geometry, session and UI tests with the corresponding implementation. Do not substitute placeholder tests for behavior coverage.

For Windows integration evidence record OS build, display layout/scale, WebView2 version, build identifier, synthetic fixture, expected/actual outcome and pass/fail. Exercise Windows 10 and 11 on installed x64 release builds. Record unexecuted cases as pending.

Measure performance using the specified reference hardware and 20-run protocol. Inspect files, logs and traffic on success, failure and cancellation. Verify no screenshot persistence or unsolicited external actions. P1/P2 acceptance cases are excluded.

## Milestone 2: complete, including multiple-result update (2026-11-06)

The user reported all original M2 automated checks and manual desktop fixture cases passed before this change. The user then authorized replacing the multiple-QR error with a selectable list; FR-04/FR-05 and session wording were updated accordingly.

Verified after the change:

- PASS: cargo fmt --check and cargo clippy --all-targets -- -D warnings.
- PASS: cargo test --offline: 12 tests (10 decoder/URL/fixture tests, one command/guard test, one capability configuration test).
- PASS: npm test: 5 React tests, including selection changes, partial failure feedback, literal text, stale URL removal and selection reset.
- PASS: npm run lint and npm run format:check.
- PASS: npm run tauri -- build --no-bundle, including strict TypeScript checking and Vite build. Artifact: src-tauri/target/release/screenqr.exe.

Rust image tests verify two distinct results, horizontal/vertical ordering, duplicate symbols, independent URL classification, readable + damaged + binary partial results, and deterministic error precedence when all candidates fail. Existing exact Unicode and negative-image tests still pass. No new dependencies, permissions, capture, automatic opening or P1/P2 features were added.

PASS (user-reported, 2026-11-06): after the updated release was provided, the user confirmed the multiple-QR workflow works, following the instructions to decode Two QR codes and switch QR code 1/2. The user previously reported all original M2 manual fixture cases and commands passed. The agent has not visually inspected the updated desktop UI; this record attributes manual verification to the user.

M2 acceptance is complete for the fixture-based decoder scope. Strict UTF-8, legacy/mixed encoding and detection limitations remain documented in ARCHITECTURE.md. This does not certify screen capture, an installed/offline scan workflow, performance targets or Windows 10/11 clean-machine compatibility. This was the M2 closure record; see the current M3 phase status below.

## Phase 3.1 verification (2026-11-06)

Passed: cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test (19 tests); npm lint; npm test (8 React tests); npm build (strict TypeScript and Vite). Tests cover hide-before-capture order, success/failure restoration, failed hide preventing capture, panic cleanup, duplicate permit rejection/release, controlled native errors, scoped IPC configuration, explicit UI activation and preview Blob cleanup/retry. Native capture probe captured 1920x1080 at 100% scale without saving pixels. This does not verify window occlusion or Windows release compatibility.

Manual checklist (retained for regression testing; user confirmed the Phase 3.1 workflow works, without reporting per-case results):

1. Run the desktop build. Put a distinctive synthetic QR image on the primary display, then cover it with ScreenQR. Choose **Test screen capture**. Confirm the preview shows the formerly covered image, without ScreenQR, and the app returns and accepts keyboard input.
2. Repeat ten times; clear each preview. Confirm no stranded hidden window, stale image or duplicate capture. The preview is a capture diagnostic, not a QR decoding result.
3. Test inaccessible/protected content where available. A native denial must be recoverable; an indistinguishable blank protected surface is valid. Never attempt to bypass Windows protection.
4. Repeat at 125%, 150% or 200% scale if available. Record OS build, display dimensions/scale and actual outcome. Display changes during capture should produce a retry error.
5. Check no screenshots are created. Inspect source/permissions and observe offline operation. Full traffic/resource profiling, secure desktop, protected-surface behavior, non-100% DPI and Windows 10/11 clean-machine checks remain unverified.

Phase 3.1 is accepted for development progression based on the user confirmation. M3 remains incomplete. No overlay, region selection, crop or screen decoder flow is claimed by these checks.

Windows release build: `npm.cmd run tauri -- build --no-bundle` passed after Phase 3.1 changes; executable: `src-tauri/target/release/screenqr.exe`. The user subsequently confirmed Phase 3.1 works after manual testing. This is user-reported evidence, not an agent-observed visual check. OS build, test repetition count and DPI of this manual run were not supplied. Protected content, secure desktop, alternate DPI and Windows 10/11 compatibility remain unverified unless separately recorded.
