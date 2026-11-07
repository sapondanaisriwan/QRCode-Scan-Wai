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

## Phase 3.2 handoff (2026-11-07, not accepted)

Implementation adds Rust tests for four drag directions, clamping/zero-area/non-finite input, viewport mismatch and overlay ACL isolation. React tests cover logical geometry, reverse drag submission, Escape/zero-area behavior, Blob cleanup and retry feedback. These new tests and the Rust build have NOT been executed by the agent; the user requested that lengthy validation commands be handed off. Do not carry over Phase 3.1 pass counts as evidence for this change.

Run from the repository root:

```powershell
npm.cmd run lint
npm.cmd test
npm.cmd run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd run tauri -- build --no-bundle
```

Then run the newly built executable and choose **Test region selection**. Confirm frozen pixels cover the primary monitor including the taskbar, with dimming outside a clear rectangle. Test all four drag directions, zero-area click, pointer leaving/re-entering the screen, Escape before/during drag, Alt+F4, repeated selection/cancellation, main-window restoration/focus and display changes during selection. Verify at a non-100% DPI when possible. Selection currently reports logical dimensions only; no crop or QR result is expected. Record each actual result before accepting 3.2. 3.3/3.4 and full M3 acceptance remain pending.

## Phase 3.2 user confirmation and Phase 3.3 handoff (2026-11-07)

After fixing the missing `tauri::Manager` import, the user reported testing with no issues. This closes the 3.2 development gate using user-reported evidence; it does not establish specific OS/DPI coverage or individual test counts.

3.3 adds tests for four drag directions at 100/125/150/200%, negative monitor origins, fractional rounding, zero area before rounding, out-of-bounds clamping, NaN/infinite/overflow inputs, portrait and landscape marker-pixel crops, half-open edges and crop overflow. React tests check cropped preview transport and cleanup on a cancelled retry. These new tests, Clippy and release build are handed to the user and are not claimed passed.

Run the validation commands in the 3.2 handoff above. In the newly built app, choose **Test region selection** and inspect the crop: it must show exactly the selected desktop content, without dimming or selection borders. Test reverse drags, all screen edges, zero-area clicks and repeated Escape/retry. Compare known pixel markers at 100% and non-100% DPI if available. Change display settings during selection and confirm recovery instead of a stale crop. Clear the preview when finished. Record actual results before closing 3.3; QR decoding remains 3.4.

## Phase 3.3 confirmation and Phase 3.4 handoff (2026-11-07)

The user reported testing 3.3 with no issues; this is user-reported acceptance, not evidence of every Windows/DPI configuration. Phase 3.4 remains pending.

Agent checks for 3.4: formatting, TypeScript and targeted ESLint passed. Added but not executed by the agent: crop-to-decoder fixture integration (URL, Thai, emoji, multiline, literal HTML, multiple, blank, damaged and binary); exclusion of a QR outside the crop; native decoder bounds; React multiple-result selection, controlled retry, duplicate-trigger suppression and late-result rejection after cancellation. Existing M2 decoder tests exercise the shared logic. Do not assume old test counts apply.

Run from repository root:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd run lint
npm.cmd test
npm.cmd run tauri -- build --no-bundle
```

Using the new executable, verify **Scan screen -> Select -> Crop -> Decode -> Display** with QR codes in a browser and another application. Check Thai, emoji, multiline, URL and multiple-result review. Select blank space, a damaged code and a region excluding a nearby QR; confirm controlled errors/retry. Exercise Escape during selection and Cancel/Escape during decoding, then rescan and confirm no stale content appears. Include reverse drags, screen edges and non-100% scale if available. No URL should open or make a network request. Record each tested configuration and outcome before marking 3.4/M3 complete. Protected desktop, Windows-family compatibility and broader performance/privacy gates remain unverified unless explicitly recorded.

## M3 confirmation and M3.5 handoff

The user confirmed the 3.4 screen-decoding workflow works without issues. M3 is accepted for development progression using that user report; exact OS/DPI and per-case coverage were not provided. M3.5 is a separately approved FR-15 extension and is not yet accepted.

M3.5 changes add tests for crop-offset/inverse-DPI mapping, invalid bounds, duplicate payloads with distinct positions, layout collisions/viewport bounds/dense fallback, literal card text and dismissal, and frozen-image retention/release across overlay review. Formatting, TypeScript and targeted ESLint passed. The agent has NOT run the new Rust/React tests or release build; the user requested heavier commands be handed off.

Run from root:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd run lint
npm.cmd test
npm.cmd run tauri -- build --no-bundle
```

Native T10: select two distinct QR codes in one region, then two identical codes. Verify each numbered card matches its marked symbol and the main list retains both after closing. Check adjacent/crowded and screen-edge codes, fallback list, Thai/emoji/multiline/HTML-like content, partial successes and keyboard access. Escape during decoding must publish no new result; Escape during review and both close/list buttons must dismiss the frozen image and retain the completed batch. Repeat scans/cancellation, change display settings during review, and verify fresh captures and no stranded overlay. Test at non-100% DPI if available. No Copy/Open or automatic navigation is expected. Record actual outcomes before closing M3.5 or moving to M4.

## M3.5 confirmation and M4 handoff

The user reported M3.5 testing without issues. M4 now adds explicit Copy/Open in main, fixture review and spatial cards. Agent checks: Rust formatting, TypeScript and targeted ESLint passed. New tests have been written but NOT executed: result/window ownership, generation expiry and late-result rejection, duplicate native permits, exact-destination revalidation, UTF-16 preservation/NUL rejection, no action on render, Copy retry, failed Open retaining Copy, duplicate clicks and switching action targets. ACL expectations now include only the two narrow action commands in main/overlay.

Run from repository root:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd run lint
npm.cmd test
npm.cmd run tauri -- build --no-bundle
```

Manual acceptance (pending):

- Copy Thai, emoji, multiline/whitespace and long content from both a spatial card and the main list; paste into a text editor and compare exact text. No copy occurs before clicking.
- Copy while the clipboard is occupied by another application if feasible; verify a controlled error and successful retry after release. Text/results must remain available.
- Scan two different URLs, switch cards/results and check the full destination before clicking Open. Exactly the selected destination should reach the default browser, once. No browser/network request should occur from scanning/review alone.
- Verify javascript/file/mailto/plain domains/credentials/invalid URLs remain Copy-only. Mixed-case HTTP(S) and IDN display the canonical destination. Existing Rust URL cases must pass.
- In spatial review, verify the browser appears in front and the results remain reachable on the taskbar. Return to review and close it; test failure/retry where feasible. Escape in the browser must not dismiss background review.
- Start another scan or fixture decode, then try an older diagnostic result: actions must be expired. Repeat cancellation/scan and ensure no wrong-target action or stale feedback.

Do not close M4 until automated checks/build and these core Windows interactions are verified. Clipboard contention, browser-association failure, Windows-family compatibility, performance and configurations not actually exercised remain unverified. Windows clipboard history/sync and external-browser traffic are explicit user-action privacy boundaries documented in README.
