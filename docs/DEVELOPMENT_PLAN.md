# P0 development plan

REQUIREMENTS.md governs behavior and acceptance. Each milestone is implemented and reviewed separately. No P1/P2 features are scheduled.

## M1: Foundation (complete, 2026-11-06)

Files: package/configuration files, src entry points, src-tauri entry points and capabilities, README, and development/test documentation.

Dependencies: React for presentation; TypeScript for strict checking; Vite for bundled assets; Tailwind v4 and its Vite plugin for styling; Tauri v2 and tauri-build for the desktop host; ESLint, React Hooks rules and Prettier for code checks. No capture, storage, opener or shortcut plugins are needed yet.

Risks: missing Rust/MSVC/WebView2 prerequisites can block native verification. The application identity requires Store reconciliation. Empty capabilities must be expanded narrowly as commands are implemented. Frontend success cannot establish Windows compatibility.

Exit gate: frontend checks and native formatting, linting, tests, build and window-launch verification pass. Checks and native launch/close are recorded in TEST_PLAN.md. This closes the development foundation only, not the MVP or Windows release certification.

## M2: Local decoding and safe result model (complete, 2026-11-06)

Completed the standalone rqrr decoder, strict UTF-8 preservation, bounded PNG inputs, typed results/errors, HTTP(S) URL classification, scoped fixture-only Tauri IPC and selectable multiple-result review. Automated checks and release build passed; the user confirmed the original fixture workflow and updated multiple-QR desktop UI. See TEST_PLAN.md for evidence and ARCHITECTURE.md for encoding limitations. No file/clipboard import UI was added.

## M3: Primary-display capture and selection

Depends on M2. Implement FR-01 to FR-03 and the capture contract. Risks include repaint timing, DPI transforms, topology changes, cancellation races and resource cleanup. Add geometry/session tests and Windows T03/T05 checks.

## M4: Review and explicit actions

Depends on M3. Implement FR-05 to FR-07, literal selectable content, retry, clipboard error recovery and native URL revalidation. Test stale targets and action authorization (T02/T06).

## M5: Shortcut and P0 validation

Depends on M4. Implement FR-08 with held-key suppression, conflict reporting and shutdown cleanup. Complete P0 T01-T07, accessibility, privacy, lifecycle and measured performance checks. Validate installed x64 builds on Windows 10 and 11. Store submission remains a separate approval gate.

## M3 phase status

Phase 3.1 is complete for the development phase: implementation and automated checks passed, and the user confirmed it works after manual testing. This is user-reported verification on their environment, not agent-observed inspection or certification of every Windows configuration. Capture probe succeeded at 1920x1080, 100% scale. Changed capture modules, scoped command/ACL, capture preview UI and tests. Dependencies: Windows-only xcap and windows API bindings. Phase 3.2 overlay, 3.3 coordinates/crop and 3.4 decoder integration have not started. Phase 3.2 is next. M3 remains incomplete until selection, crop and decoder integration pass their Windows acceptance checks. P1/P2 remain out of scope.

## M2 user-approved scope update

Multiple QR symbols in one manually supplied image now return a selectable result list instead of an error. Preserve duplicates, sort by top edge then left edge, and retain readable results when other candidates fail. This does not add P2 full-frame auto-detection or image import. Updated implementation passes 12 Rust tests, 5 React tests, lint/format/type checks and the release build. The user confirmed the changed multiple-QR list works on the desktop release on 2026-11-06, closing the remaining M2 manual acceptance check. This is user-reported verification, not an agent-observed visual inspection.
