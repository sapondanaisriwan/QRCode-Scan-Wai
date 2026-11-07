# ScreenQR architecture

## Boundaries

`decoder::decode_image(&[u8])` is synchronous, standalone Rust: it reads PNG bytes, bounds dimensions/allocation, converts to grayscale, detects QR candidates, decodes each candidate and classifies its text independently. It imports no Tauri, filesystem, clipboard, browser or network API.

`commands::decode_fixture` is the M2 IPC entry point. It accepts a synthetic fixture identifier, not a path, URL or image buffer. A native allowlist resolves that identifier to `include_bytes!` assets. This is a milestone test surface, not P2 image import. It should be removed/replaced with native session-owned capture data in M3. No image bytes cross the decoder fixture IPC.

The command runs on `spawn_blocking` and a single atomic permit bounds concurrent work; an RAII guard releases it on success, failure or panic. A busy response does not queue another job. Tauri command ACL enrollment in `build.rs` plus `main.json` grants `allow-decode-fixture` only to the local main window. No filesystem, shell, opener, HTTP or clipboard plugin is installed.

## Dependencies and evaluation

- `rqrr 0.11`: maintained Rust decoder, MSRV 1.85.1, `(MIT OR Apache-2.0) AND ISC`. Detection is distinct from decoding, its byte writer preserves payload bytes, and decoding errors are explicit. [Upstream](https://github.com/WanzenBug/rqrr), [API](https://docs.rs/rqrr/0.11.0/rqrr/).
- `image 0.25`: PNG codec and grayscale conversion; default features disabled, PNG only. No JPEG/file-picker/import feature is provided.
- `serde`: tagged IPC serialization; `url`: absolute URL parsing and IDNA normalization. Both are already present transitively through Tauri, now explicitly declared for direct use.
- `qrcodegen 1.8`: dev-only deterministic fixture generation. `serde_json` and Tauri's test feature are dev-only test helpers. None introduces QR generation in the product.
- `@tauri-apps/api`: typed invoke wrapper. Vitest, jsdom and Testing Library are dev-only tests for IPC-driven React states, literal text and retry.

`quircs 0.10.3` was evaluated but not selected: its source loses unknown ECI assignments and stops on unknown data modes. It does not solve the full encoding problem. rqrr 0.11 also skips ECI rather than exposing segment encoding metadata; this milestone explicitly supports UTF-8 payload bytes, with or without ECI 26, and ASCII numeric/alphanumeric text. It does not implement legacy encoding transcoding or mixed ECI interpretation. Non-UTF-8 bytes fail strictly (no replacement characters). A legacy-encoded payload whose bytes also happen to be valid UTF-8 cannot be distinguished by this API. Revisit the decoder/segment API before claiming broader encoding support.

## Deterministic outcomes

1. Reject encoded input above 4 MiB; reject invalid/non-PNG data.
2. Read dimensions before full decoding. Reject zero dimensions, either side above 4096, or more than 4,194,304 pixels. Apply a 64 MiB image-decoder allocation budget. Detector allocations are additional but bounded by the pixel cap; this is not a total-process memory guarantee.
3. Zero candidates: `no_qr_detected`. Sort candidate bounding-box minimum y, then minimum x; decode independently and preserve duplicate payloads as separate entries.
4. Return every readable UTF-8 result in spatial order, plus `unreadableCount` and `unsupportedCount`. A partially readable image is successful. Counts apply only to detected candidates; destroyed finder patterns may not be detected.
5. If no result is readable, return `malformed_qr` when any candidate failed correction/parsing, otherwise `unsupported_content`. This precedence is independent of spatial order.

Responses are `{status:'success', result:{results:[{content,type,launchDestination}],unreadableCount,unsupportedCount}}` or `{status:'error',error:{code,message}}`. Success always contains at least one result. React defaults to the first item and provides keyboard-accessible radio selection; changing the image or decoding again clears the old selection and results. M4 supplies shared explicit Copy/Open actions below.

## URL safety

Original content is retained separately. Only entire absolute HTTP(S) addresses with explicit `://` and a nonempty authority qualify. Reject whitespace, controls, backslashes, credentials (including empty userinfo), malformed percent escapes and invalid host/port syntax before/after parsing. Return the parsed canonical destination with ASCII IDN hostname. Classification performs no DNS, reputation check, preview or browser handoff. M4 displays the destination as literal text and revalidates it immediately before an explicit Open action.

## Scope and limitations

Both synthetic fixtures and manually selected native screen crops can be decoded. M4 adds Copy/Open controls; shortcuts, tray, history and general image import remain absent. Rotation, perspective, inverted symbols, legacy encodings and real-world damaged-code tolerance are not release-certified. Performance targets and Windows 10/11 clean-machine certification remain later P0 gates.

## Phase 3.1: native capture (user-verified)

`capture/native.rs` uses Windows-only `xcap 0.9.8` with default features disabled. Its GDI backend provides a one-shot RGBA frame without a recording session or WGC feature. This avoids writing another unsafe GDI resource-management layer. A small `windows 0.62` wrapper establishes/restores thread-local per-monitor DPI awareness and calls DwmFlush. Sources: [xcap](https://github.com/nashaofu/xcap), [BitBlt](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt).

Capture records monitor ID, signed physical origin, dimensions, orientation and scale. It checks metadata before and after capture and rejects changed displays or dimensions. The 32-megapixel cap bounds a raw RGBA frame to 128 MiB; encoding and IPC copies are additional. This is not a total memory/performance guarantee. Protected surfaces may be blank; only an explicit access-denied native error is labelled denial. No protection bypass or blank-image guessing is attempted.

`capture/workflow.rs` manages hide/capture/restore independently of Tauri and decoding. The main-window-only `capture_preview` command acquires a non-queuing permit, runs on a blocking worker, hides the window and checks visibility before waiting for compositor repaint plus 120 ms. Restoration runs after success/failure and during unwinding; restoration failure gets a controlled error with one best-effort retry. Window focus remains subject to Windows policy.

The temporary phase-test UI receives one PNG via binary IPC, not a JSON byte array, and keeps a Blob URL in memory. Native frame/encoding buffers leave scope after the response; React revokes the Blob URL on clear, replacement, failure or unmount. CSP allows blob images only; no filesystem, remote URL or generic window permissions are granted. No capture runs without an explicit button click. This diagnostic preview should be replaced by session-owned frozen-frame transport in 3.2. The separate Phase 3.2 test implements overlay selection and session cancellation; cropping remains absent.

## Phase 3.2: region selection (user-verified)

`start_selection` shares the native capture permit with the 3.1 preview. A blocking worker hides the main window, captures one frame and owns the native frame until termination. Each session creates a fresh `selection-N` WebView, initially hidden. After its PNG loads, Rust positions and sizes the borderless, topmost window in physical monitor pixels, verifies its client dimensions and focuses it. No full-screen auto-monitor selection or screenshot files are used.

Only the owning window label can retrieve its PNG, announce readiness, submit a logical rectangle or cancel. `selection-*` capability grants these operations and the session-owned `selection_review` query only; main alone may start capture. The command waits for the session outcome, preventing late results from a prior session. Escape is handled by the overlay and by GetAsyncKeyState polling only while the worker is waiting for this explicit selection; no shortcut is registered. Close requests, image-load failure, setup timeout, worker unwind and display-topology changes clean up the session and attempt main-window restoration. A frozen frame can remain in memory while the user selects; all buffers are dropped on termination and the overlay WebView is destroyed.

Pointer capture supports reverse drags and releases on completion, cancellation or component cleanup. Zero-area rectangles remain in selection mode. Rust rejects non-finite input and mismatched viewport dimensions, normalizes/clamps logical bounds and reports them for this phase test. Phase 3.3 adds physical crop conversion below; Phase 3.4 invokes the shared decoder as described below. Protected/secure desktop, taskbar coverage, actual DPI alignment and focus behavior still require Windows verification. Escape during the synchronous initial capture/encoding is processed when the worker reaches its selection wait; immediate preemption of those native operations is not provided.

## Phase 3.3: coordinate conversion and crop (user-verified)

`capture/crop.rs` is a pure module with no IPC or decoder dependency. Overlay-local endpoints multiply by the actual native overlay scale once; physical desktop endpoints instead subtract the signed monitor origin without scaling. Normalize direction, clamp to the frame, reject empty bounds before rounding, floor lower edges and ceil upper edges. PixelRect describes half-open physical bounds. The checked crop rejects overflow/out-of-frame rectangles before image::crop_imm allocates a new image. No resampling or rotation is applied: display-oriented capture pixels are used as-is.

The overlay records its scale when ready. Commit verifies that scale, client size and physical position still match the captured monitor; viewport size must match the native logical size. The worker crops the original RGBA frame, never the dimmed WebView rendering, and rechecks topology/cancellation before publishing. The phase-3.3 crop preview was a temporary diagnostic, now replaced by native decoding in 3.4. Its command, capability and retained PNG state have been removed. No file/network access or dependency is added.

Negative origins, portrait dimensions, rounding and common DPI scales have synthetic tests; real secondary-monitor scanning is still P1 and is not claimed. The user reported no issues testing 3.3; exact OS/DPI coverage was not supplied.

## Phase 3.4: native crop to QR results (user-verified)

`decoder::decode_rgba` accepts an owned native crop, validates the existing 4096-side/4,194,304-pixel limits, then uses the same grayscale rqrr path as fixture PNG decoding. The selection worker checks limits before copying pixels. No crop PNG encoding or image transfer to React occurs. Error handling, strict UTF-8, ordering, duplicate preservation and partial-success counts remain the M2 behavior.

M3.5 extends the original 3.4 lifecycle: the overlay remains visible during scoped decoding and result review. Raw full-frame pixels and the native PNG copy are released before decoding; the frozen image remains in the overlay WebView until dismissal. The native coordinator polls Escape and topology, while main has a narrowly scoped `cancel_scan` command. An atomic cancellation flag takes precedence over queued selection, and a cancelled job's decoded output is discarded. rqrr itself is not interruptible: the coordinator waits for the bounded worker to finish before releasing the single scan permit, preventing an accumulation of abandoned jobs. The UI remains responsive and disables another scan until that completion. No result is published if the display changed.

`start_selection` resolves to `{status:'cancelled'}` or `{status:'decoded',response:DecodeResponse}`; capture/lifecycle failures reject with the existing structured CaptureFailure. React clears old results before invoking and also discards late output after a local cancellation request. `ResultReview` is shared with fixture review, renders literal text and keeps each validated destination tied to its selected entry. No Copy/Open, shortcut, tray or history feature is introduced.

## M3.5: spatial result review (user-verified)

Successful decoded entries now include optional `bounds` (x/y/width/height in decoder-image physical pixels). The pure text classifier leaves bounds absent; actual QR detection sets clamped axis-aligned bounds from rqrr corners. Duplicate text retains separate entries and separate bounds. Payload/type/destination ordering is unchanged.

Rust maps each readable bound by `(crop offset + QR coordinate) / pinned native overlay scale`. Negative desktop origins are already removed by frame-local crop coordinates and must not be added again. The active session stores the batch and aligned logical anchors. Only its owning overlay can query `selection_review`; polling starts after commit and stops once the review arrives. Repeated selection input is disabled during decoding/review. No extra capture, resampling, network request or general image import occurs.

The overlay shows numbered markers, non-overlapping nearby cards with connectors, full scrollable literal content and validated destination text. A deterministic layout checks card/symbol collisions and viewport margins; dense/small screens fall back to a numbered scrollable list. Keyboard users can tab through cards and dismissal controls. Main receives the identical ordered batch when review closes. Partial-success counts remain visible. No unreadable candidate is falsely given a decoded payload.

Escape/Close during review releases the frozen WebView and restores main with the completed batch. Before review publication cancellation discards the decoded batch. Topology/DPI invalidation clears the overlay and returns a controlled error. Image buffers are never persisted; screenshot retention lasts only through this explicit review. M4 must reuse the same action validation in both views. Real Windows alignment, focus, dense layouts and lifecycle remain unverified until the T10 manual gate passes.

## M4: result-scoped Copy/Open (verification pending)

`actions` keeps an in-memory registry of results issued by Rust. Each successful result receives an `actionId`; main may act on current results, and an overlay may act only on entries issued to its exact window label. A new scan/fixture decode clears the registry and advances a generation; a late decoder cannot re-register obsolete results. Failed/cancelled scans invalidate their own generation without erasing a newer one. No IPC accepts arbitrary clipboard text or an arbitrary launch URL.

`copy_result(actionId)` retrieves original content and encodes unchanged UTF-16 plus a terminator. Embedded NUL or over-limit text yields a controlled error. `open_result(actionId, displayedDestination)` re-parses the stored original with the existing URL validator and requires equality with both the stored canonical target and the displayed destination. Only after all checks does Windows receive an open request. A native action permit and UI pending guards prevent overlapping duplicate work. Errors contain stable codes/messages, not payloads or native details; retry retains the result.

Windows support uses the existing windows crate with DataExchange/Memory/COM/Shell features. Clipboard allocation uses GMEM_MOVEABLE and RAII cleanup; ownership transfers to Windows only after SetClipboardData succeeds, and clipboard access is always closed. The browser request uses a fixed open verb with no command parameters or working directory, on a COM STA thread. References: [SetClipboardData ownership](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setclipboarddata), [ShellExecuteW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew). No generic shell/clipboard plugin is exposed.

`ResultActions` is shared by the main list and spatial cards. Destination text remains visible; long card content scrolls while action controls stay outside that scroll area. Selection changes remount the actions to prevent late feedback attaching to another result. Opening from a spatial card removes topmost status and exposes the retained review on the taskbar so the browser can appear above it; failures attempt to restore overlay flags. Escape polling during review applies only while that review window has focus, avoiding accidental dismissal when Escape is used in the browser. Actual browser foreground behavior and clipboard paste fidelity require manual Windows testing.
