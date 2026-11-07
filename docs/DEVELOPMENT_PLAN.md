# P0 development plan

REQUIREMENTS.md governs behavior and acceptance. Each milestone is implemented and reviewed separately. No P1/P2 features are scheduled.

## M1: Foundation (complete, 2026-11-06)

Files: package/configuration files, src entry points, src-tauri entry points and capabilities, README, and development/test documentation.

Dependencies: React for presentation; TypeScript for strict checking; Vite for bundled assets; Tailwind v4 and its Vite plugin for styling; Tauri v2 and tauri-build for the desktop host; ESLint, React Hooks rules and Prettier for code checks. No capture, storage, opener or shortcut plugins are needed yet.

Risks: missing Rust/MSVC/WebView2 prerequisites can block native verification. The application identity requires Store reconciliation. Empty capabilities must be expanded narrowly as commands are implemented. Frontend success cannot establish Windows compatibility.

Exit gate: frontend checks and native formatting, linting, tests, build and window-launch verification pass. Checks and native launch/close are recorded in TEST_PLAN.md. This closes the development foundation only, not the MVP or Windows release certification.

## M2: Local decoding and safe result model (complete, 2026-11-06)

Completed the standalone rqrr decoder, strict UTF-8 preservation, bounded PNG inputs, typed results/errors, HTTP(S) URL classification, scoped fixture-only Tauri IPC and selectable multiple-result review. Automated checks and release build passed; the user confirmed the original fixture workflow and updated multiple-QR desktop UI. See TEST_PLAN.md for evidence and ARCHITECTURE.md for encoding limitations. No file/clipboard import UI was added.

## M3: Primary-display capture and selection (user-verified)

Depends on M2. Implement FR-01 to FR-03 and the capture contract. Risks include repaint timing, DPI transforms, topology changes, cancellation races and resource cleanup. Add geometry/session tests and Windows T03/T05 checks.

## M3.5: Spatial QR result review (user-verified)

User-approved extension FR-15/T10, before M4. Add QR image bounds to successful decoder results, transform crop offsets/native DPI into overlay-local anchors, retain the frozen WebView through result review and preserve the original main-window list after dismissal. Numbered cards and connectors associate each payload with its QR; crowded screens use a numbered scrollable list. Escape before completion cancels; after decoding it dismisses review while retaining the batch. No new dependencies, automatic URL actions, Copy/Open, full-frame detection or P1/P2 features.

Files: decoder types/content/service; selection session, command registration and scoped capability; overlay review/layout components and types; geometry/decoder/UI/ACL tests; requirements and development documentation. Risks: coordinate alignment, duplicate association, dense/edge layouts, stale sessions, keyboard access and image retention. Formatting/type/targeted lint checks pass; The user subsequently reported M3.5 testing without issues. This is user-reported acceptance with no claim of exhaustive OS/DPI coverage.

## M4: Review and explicit actions (implementation ready; validation pending)

Depends on M3/M3.5. Implements shared Copy/Open controls, scoped result IDs, generation-based invalidation, Rust URL revalidation and controlled retry. Uses the existing windows crate for UTF-16 clipboard ownership and default-browser handoff; adds no package. Changed actions modules, decoder result metadata, command registration/capabilities, both result views, tests and docs. Risks: native clipboard ownership, Unicode/newline paste fidelity, default browser/focus handoff and stale results. Formatting, TypeScript and targeted ESLint pass; Rust/React tests, Clippy, release build and manual T02/T06 remain pending user-run checks. Do not mark M4 complete before those pass.

## M5: Shortcut and P0 validation

Depends on M4. Implement FR-08 with held-key suppression, conflict reporting and shutdown cleanup. Complete P0 T01-T07, accessibility, privacy, lifecycle and measured performance checks. Validate installed x64 builds on Windows 10 and 11. Store submission remains a separate approval gate.

## M3 phase status

Phase 3.1 is complete for the development phase: implementation and automated checks passed, and the user confirmed it works after manual testing. This is user-reported verification on their environment, not agent-observed inspection or certification of every Windows configuration. Capture probe succeeded at 1920x1080, 100% scale. Changed capture modules, scoped command/ACL, capture preview UI and tests. Dependencies: Windows-only xcap and windows API bindings. Phase 3.2 is user-verified after the missing Manager import was fixed: dynamic topmost borderless windows, frozen PNG transport, logical drag selection, Escape, single capture permit, display-change checks and terminal cleanup. Added Rust normalization/ACL tests and React interaction tests. No new dependency; enabled the existing windows keyboard API feature for Escape polling during an explicit session only. Phase 3.3 is user-verified following their no-issues test report. Phase 3.4 now integrates bounded native crops with the M2 decoder, shared result rendering, retry and cancellation during decoding. The user subsequently reported the 3.4 flow works without issues, closing the M3 development gate. This is user-reported testing, not certification of every Windows/DPI combination. P1/P2 remain out of scope.

## M2 user-approved scope update

Multiple QR symbols in one manually supplied image now return a selectable result list instead of an error. Preserve duplicates, sort by top edge then left edge, and retain readable results when other candidates fail. This does not add P2 full-frame auto-detection or image import. Updated implementation passes 12 Rust tests, 5 React tests, lint/format/type checks and the release build. The user confirmed the changed multiple-QR list works on the desktop release on 2026-11-06, closing the remaining M2 manual acceptance check. This is user-reported verification, not an agent-observed visual inspection.
