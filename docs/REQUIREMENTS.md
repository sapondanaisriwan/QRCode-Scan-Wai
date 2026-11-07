# ScreenQR product requirements

ScreenQR lets a Windows user read a QR code already visible on their desktop. A user can select a code in a presentation, browser or paused video, inspect its contents, then copy text or open a web address. A scan must work with the network disconnected.

## 1. Scope and architecture

| Priority | Delivery boundary |
| --- | --- |
| P0 · MVP | On-demand capture of the primary display, rectangular selection, local decoding, result review, copy, safe URL opening and a global shortcut. |
| P1 · Version 1 | Tray controls, optional local history, settings and scanning on any one connected display. |
| P2 · Future | User-triggered automatic detection and image import from a file or clipboard. |

Target Windows 10 and 11 x64. Use Tauri v2 with a bundled React and strict TypeScript interface, Vite, npm and Tailwind CSS v4. Rust owns capture, crop, QR decoding, storage and Windows integration. React owns controls, feedback and presentation. Use small typed Tauri commands with structured errors. Keep full-resolution frames native where possible. The installed application serves bundled assets and requires no backend server, account, cloud decoder or development server.

## 2. Functional requirements and acceptance criteria

Each row is independently verifiable. P1 and P2 rows become release gates only when their milestone is scheduled.

| ID | Priority | Required behavior | Acceptance criterion |
| --- | --- | --- | --- |
| FR-01 | P0 | Start one scan from the main window. Hide ScreenQR surfaces and wait for the desktop to repaint before acquiring a single frame. | A code behind the main window can be decoded. Repeated triggers during selection or decoding create one session and one result batch at most. |
| FR-02 | P0 | Show a borderless, topmost selection overlay over the captured primary display. Show the frozen frame through the selection, dim its surroundings and accept dragging in any direction. | Releasing the pointer commits the visible rectangle. A zero-area selection stays in selection mode. Cropping excludes the overlay, dimming and selection border. |
| FR-03 | P0 | Escape cancels an active selection or pending scan. Clean up on cancellation, failure and completion. | Escape removes the overlay, restores prior focus where Windows permits, and produces no new result, clipboard write or history entry. A late decoder response is discarded. A new scan works immediately. |
| FR-04 | P0 | Decode standard QR symbols inside the crop in Rust. Preserve the original decoded text, including Thai, line breaks and whitespace. | A UTF-8 fixture matches its expected string exactly. A blank crop offers Retry. Undecodable data produces readable feedback. Multiple readable codes are listed in top-to-bottom, then left-to-right order for individual review. Keep duplicate symbols as separate entries. If only some codes can be read, show those results and counts of unreadable/unsupported candidates. Binary data with no supported text encoding produces an unsupported-content message. |
| FR-05 | P0 | Display successful results as a selectable list; show the selected result as literal text with its own content type and available actions. | Long content scrolls without hiding actions. HTML-like text appears verbatim. Switching results updates text and URL action targets together. Starting another scan clears stale results and action targets. |
| FR-06 | P0 | Copy the complete original text only when the user selects Copy. | A paste into a plain-text editor matches the fixture, including line breaks. If the clipboard is busy, the result stays visible and Copy can be retried. |
| FR-07 | P0 | Offer Open only for a validated absolute HTTP or HTTPS address. Display the destination before the user activates Open. Rust revalidates the same destination before invoking the default browser. | Scanning a URL makes no external request. One explicit Open action passes one validated URL to Windows. Opening failure preserves Copy. All cases in section 5 pass. |
| FR-08 | P0 | Register a global shortcut while ScreenQR is running. Display its binding and registration status in the main window. | From another focused app the shortcut starts selection once, including when held down. A conflict explains that Scan remains available. Exit releases registration. |
| FR-09 | P1 | Provide tray actions for Scan, Show ScreenQR and Quit. Explain close-to-tray behavior on first use. | Close hides the window while the shortcut works. Show restores it. Quit ends the process, releases the shortcut and removes the icon. |
| FR-10 | P1 | Offer opt-in, bounded, per-user local history with review, disable and clear controls. | Only successful scans performed while enabled are recorded. Entries survive restart. Disabling stops new writes. Confirmed Clear removes all entries and restart shows an empty list. |
| FR-11 | P1 | Persist history preference and shortcut settings. Validate a replacement shortcut before relinquishing the old one. | A valid rebind survives restart. A conflicting rebind explains the conflict and retains the previous working binding. |
| FR-12 | P1 | Support region selection on any attached display, one display per scan, including displays left of or above the primary. | A drag started on a secondary display decodes its own pixels at its own scale. Crossing its edge clamps to that display. |
| FR-13 | P2 | Auto-detect QR codes in one user-requested frame on the chosen display. | Zero matches offer manual selection. One match opens review. Multiple matches allow the user to select a result. Detection stops after that frame. |
| FR-14 | P2 | Decode a user-selected PNG or JPEG file or a clipboard image after an explicit import action. Apply ordinary result and URL rules. | Valid imports work offline. Cancel leaves the source untouched. Missing, corrupt, unsupported or oversized input yields a recoverable message before excessive allocation. |
| FR-15 | P0 (approved M3.5 extension) | After decoding a manually selected region, keep the frozen primary-display image and show a numbered result card associated with each readable QR. Preserve the existing main-window result list. | Two readable QR codes produce two correctly associated results, including duplicate payloads. Card/marker positions use crop offset and native overlay DPI. Cards stay within the viewport and do not overlap; dense layouts use a scrollable numbered list with matching QR markers. Full original text and validated destination remain reviewable as literal text. Escape, Close results or View in main window dismisses the frozen view, releases image buffers and restores main with the same batch. Before decoding finishes, Escape still cancels and publishes no result. No automatic URL action occurs. |

## 3. User flows and use cases

| Use case | Preconditions and user flow | Observable outcome |
| --- | --- | --- |
| Read a link in a slide | With the slide visible and ScreenQR running, press the shortcut, drag around the code, release, inspect the destination and choose Open. | Review appears before Windows receives a browser request. Dismissing review ends the flow without opening a browser. |
| Copy a Thai reference number | Open ScreenQR, select Scan, drag around the reference QR and choose Copy. Paste into the destination application. | The pasted text preserves the original characters and spacing. |
| Recover from a missed code | Select an empty area, read the feedback, choose Retry and select the full code including its white border. | Retry captures a fresh frame. Only the successful attempt exposes result actions. |
| Abandon a selection | Trigger from another application, begin a drag and press Escape before releasing. | The desktop becomes interactive again. The clipboard retains its previous value. |
| Revisit a result (P1) | Enable history, scan a code on a secondary display, open history from the main window and select the entry. | The saved text is available offline. The same validation and explicit action are required to open its URL. |

### M3.5 result-review behavior

This user-approved extension applies only to QR codes inside the user-selected crop. It does not enable FR-13/P2 full-frame automatic detection. The overlay transitions from selection to decoding to review; selection cannot restart over a retained old frame. Main remains the alternative list view, reached by dismissing review. Result numbers follow the decoder's spatial order and link cards to QR markers. Long text scrolls, keyboard focus reaches every card and the close/list controls, and partial-success counts remain visible. A non-overlapping numbered list is the fallback when cards cannot fit near all symbols. Errors with no readable results return to main with Retry.

Keep the frozen image in memory only while selecting, decoding or reviewing. Discard it on dismissal, failure, display/session invalidation or shutdown. Display topology/DPI changes invalidate review as well as selection. Future M4 Copy/Open actions must share the same explicit-user-action and Rust URL-validation rules in both views; they are not part of M3.5.

## 4. Windows capture and overlay contract

1. Treat the primary display bounds as the P0 capture area even when another display has focus. Identify the supported display in the scan UI. P1 extends the selectable area to additional monitors.
2. Use per-monitor DPI awareness. At capture time record the display identity, signed physical-pixel origin, native frame dimensions, orientation and effective scale. Keep monitor-local logical units, virtual-desktop pixels and frame-local pixels distinct.
3. Convert each pointer endpoint using the metrics of its overlay window. For logical coordinates relative to that window, multiply by its scale once. For physical desktop coordinates, subtract the captured display origin. Normalize reversed endpoints, round the lower edges down and upper edges up, then clamp to frame bounds. The crop includes the lower edges and excludes the upper edges.
4. For example, a display at desktop x = −1920 with a 150% scale maps overlay-local x = 100 to frame x = 150 and desktop x = −1770. Verify the crop against known pixel markers. Applying scaling to already-physical coordinates would select the wrong pixels.
5. Cover the captured display including the taskbar area. Acquire pointer input for the active drag and release it on every exit. Preserve alignment for portrait displays. If display topology, DPI, orientation or session access changes during selection, discard the frame, close the overlay and offer a fresh scan.
6. Respect secure desktop, protected surfaces and OS capture restrictions. Report capture denial when the API identifies it. Treat an indistinguishable blank protected frame as an ordinary no-match. Never attempt to circumvent protection.

## 5. Privacy, security and non-functional requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| NFR-01 | Capture and decode locally only after user initiation. Release frames and crops on every terminal path. Keep screenshots, QR payloads and destinations out of diagnostic logs and crash attachments. | Exercise success, cancellation and failure with a unique test string. Inspect app files, logs and traffic for the string and image artifacts. Scanning requires zero outbound app requests. |
| NFR-02 | Treat decoded text, imported images and IPC arguments as untrusted. Render escaped text, scope Tauri permissions by window and validate session ownership, bounds and argument sizes in Rust. Exclude generic shell execution, remote frontend assets and telemetry. | Malformed rectangles, expired sessions and unexpected commands fail without side effects. Script-like content cannot run or navigate the webview. |
| NFR-03 | Parse the entire payload as a single absolute URL with a nonempty host. Reject credentials, embedded controls, leading or trailing whitespace, ambiguous backslashes and malformed addresses. Display the full parsed destination, including an ASCII hostname for internationalized domains, before opening that same destination. Preserve original text for Copy. | Test mixed-case HTTPS, query strings and internationalized hosts. Plain domains, relative paths, command strings, file paths, javascript, data, file, mailto and custom schemes remain copy-only. No DNS lookup, preview or reputation request occurs during review. |
| NFR-04 | Target startup under 3 seconds, selection-to-result within 1 second for ordinary readable codes, and idle memory under 150 MB. Keep input responsive during decoding. | On a recorded x64 reference PC with SSD, 8 GB RAM and a 1080p display, report median over 20 release-build launches and scans of a 512 × 512 fixture. Measure combined private working set of app and associated WebView processes after 60 seconds idle. Record cold/warm conditions and exceptions. |
| NFR-05 | Support standard Windows accounts, system light/dark themes, visible keyboard focus, screen-reader names and keyboard access to Scan, Copy, Open, Retry and Cancel. | On both supported Windows families, check tab order, Enter activation, Escape during selection, Narrator feedback and readable controls at 200% scale. Region drawing assumes a pointing device. |
| NFR-06 | Keep scan state consistent through cancellation, retry, shutdown and capture failure. Surface actionable messages without raw native errors. | Run 100 scans alternating cancel, success and failure. Verify no stranded overlay, accumulating capture handles or stale result. Quit during selection and relaunch successfully. |

The Windows clipboard and default browser are explicit destinations outside ScreenQR. Clipboard history or sync is controlled by Windows. Explain this boundary in privacy information. P1 history may contain secrets. Disclose its local retention before opt-in and provide Clear even when recording is disabled.

## 6. Data model and retention

| Entity | Fields and invariants | Lifetime |
| --- | --- | --- |
| ScanSession | Unique session ID, state, display metrics, optional selection and native frame handle. Only the current ID may commit a result. | Memory only. Release raw capture/crop pixels after processing; retain only the frozen review image until review dismissal, cancellation, failure or display invalidation. |
| ScanBatch | Ordered successful ScanResult entries, unreadable and unsupported candidate counts; selected result index belongs to the UI. | Memory until dismissed, replaced or the app exits. |
| ScanResult | Original content string, optional crop-local physical QR bounds for spatial review, and type (url, text or other). A validated launch destination exists only for eligible URLs. | Memory until dismissed, replaced or the app exits. P0 persists no results. |
| HistoryEntry (P1) | Unique ID, scan time in UTC, content and type. Display time in local time. Revalidate content when opened. | Per Windows user, newest first. Proposed cap of 100 entries, evict oldest on insertion. Retain until eviction or user clear. |
| Settings (P1) | Schema version, history-enabled flag and shortcut binding. Default history off. | Per-user app data. Use versioned JSON or SQLite with atomic writes. Preserve valid settings across upgrade. |

P1 storage errors must leave scanning usable. On an unreadable history store, report the problem and offer an explicit reset. On settings corruption, use safe defaults with history off. On a clear or save failure, report the failure and allow retry. Persist neither screenshots nor capture geometry with a history entry.

## 7. Test matrix

Run P0 Windows integration cases on installed x64 release builds on Windows 10 and 11. Record OS build, display layout and scale, runtime version, build identifier, expected and actual outcomes. Use synthetic QR fixtures for evidence.

| Test | Layer and conditions | Pass condition | Coverage |
| --- | --- | --- | --- |
| T01 | Rust fixtures for HTTPS, Thai, emoji, multiline text, damaged symbols, blank images, binary and two symbols | Exact expected strings or documented recovery outcome | FR-04, FR-05 |
| T02 | Rust URL and IPC tests plus UI tests for unsafe content and action visibility | Only eligible destinations can reach the opener. Copy preserves source. Text renders literally. | FR-05–07, NFR-02–03 |
| T03 | Rust geometry tests and Windows visual checks at 100%, 125%, 150% and 200%, portrait and landscape, reverse drags and edge pixels | Frame bounds match expected coordinates. A code just outside the rectangle never decodes. | FR-01–02, FR-12 |
| T04 | Global shortcut from a focused browser, held key, occupied binding, Escape during drag and clean exit | One overlay, recoverable conflict, restored interaction and released binding | FR-03, FR-08 |
| T05 | Mixed 100%/150% displays, negative origins, drag across boundary and unplug during drag | P0 uses primary only. P1 selects either monitor. Stale frames are discarded. | FR-02–03, FR-12 |
| T06 | Offline install with prerequisites available, offline scan, locked clipboard, failed browser handoff, protected window and denied capture | Scan works offline. Each failure offers recovery with no unsolicited action or disclosure. | FR-01, FR-04, FR-06–07, NFR-01 |
| T07 | UI state tests and Windows lifecycle, accessibility, resource and timing checks in section 5 | All required controls and states are reachable. Record measurements and investigate regressions. | NFR-04–06 |
| T08 | P1 tray quit, opt-in, disable, clear, capacity eviction, corrupt store, rebind conflict and restart | Correct registration, retention and recovery across restart | FR-09–11 |
| T09 | P2 blank, single and multiple QR frames, valid PNG/JPEG and clipboard fixtures, invalid or oversized input | One-shot detection and explicit import produce expected review or recovery | FR-13–14 |

## 8. MVP Definition of Done

- [ ] All P0 functional criteria and NFR checks have recorded results. The primary-display flow passes on both Windows families with real capture, decoding, clipboard, browser handoff and global shortcut.
- [ ] Rust formatting, linting and tests pass. TypeScript strict checking, frontend tests and the production Tauri build pass.
- [ ] T01–T07 pass for their P0 scope. Record measured performance against targets and obtain a documented decision for any missed target. Resolve failures involving privacy, URL safety, wrong pixels or stuck overlays before release.
- [ ] An x64 installer passes clean install, launch, offline scan and uninstall on a standard-user test environment. Document runtime prerequisites, the shortcut, primary-display limit, cancellation, privacy and recovery steps.
- [ ] Attach the build identifier and test record to the release candidate. P1, P2 and public Store publication have separate gates.

## 9. Microsoft Store readiness

1. Prepare an x64 MSIX candidate with the reserved Partner Center identity and publisher, a monotonically increasing package version, app icons, supported OS range and justified capabilities. Check packaged capture, overlay, shortcut and external browser activation under a standard account.
2. Document WebView2 provisioning. Test first run on clean Windows 10 and 11 images with the runtime present and absent. Provide a supported installation or recovery path and state any prerequisite acquisition that requires connectivity. Test offline operation once prerequisites are installed.
3. Run the Windows App Certification Kit against the candidate. Retain its report, resolve applicable failures and test install, upgrade and uninstall. For P1, verify upgrade preserves settings and history choices. [Microsoft distribution guidance](https://learn.microsoft.com/en-us/windows/apps/distribute-through-store/how-to-distribute-your-win32-app-through-microsoft-store)
4. Prepare listing copy, screenshots from the actual build, privacy policy, support contact, applicable age-rating answers and reviewer steps using a synthetic QR code. Describe capture timing, local storage and the browser/clipboard boundary accurately.
5. Recheck current submission policies at release. Track readiness, submission and certification as separate statuses. Claim Store availability only after Partner Center confirms publication. [Microsoft certification process](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-certification-process)

## 10. Assumptions and exclusions

- The MVP assumes an interactive unlocked desktop, a pointing device, a visible readable standard QR code and supported WebView2 provisioning. Record the exact minimum OS build after clean-machine validation.
- Proposed defaults for implementation are primary-display-only P0 capture, multiple symbols per manual selection with individual review, Ctrl+Shift+Q as the initial shortcut, and a 100-entry opt-in history cap for P1. Validate shortcut availability at runtime. Confirm a bounded decoded-image size before scheduling P2 import.
- Excluded from this plan are camera scanning, QR generation, non-QR barcode formats, continuous monitoring, cross-display stitched crops, screen recording, screenshot export, automatic Wi-Fi joining, accounts, cloud sync, remote decoding and mobile or non-Windows clients.

### T10: M3.5 spatial result review

Test two separated and two adjacent QR codes, duplicate payloads, dense layouts, edge positions, partial decoding, Thai/emoji/multiline/HTML-like text, reverse selections and non-100% DPI. Verify each result number matches its symbol and the main list after dismissal. Exercise Escape during decoding versus review, Close results, View in main window, repeated scans, topology changes and shutdown. Confirm no screenshot persistence, stale overlay, automatic navigation or unrequested Copy/Open. Record actual Windows/DPI coverage; compilation alone does not close this gate.
