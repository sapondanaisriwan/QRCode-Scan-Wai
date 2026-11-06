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

Responses are `{status:'success', result:{results:[{content,type,launchDestination}],unreadableCount,unsupportedCount}}` or `{status:'error',error:{code,message}}`. Success always contains at least one result. React defaults to the first item and provides keyboard-accessible radio selection; changing the image or decoding again clears the old selection and results. No Copy/Open action is implemented yet.

## URL safety

Original content is retained separately. Only entire absolute HTTP(S) addresses with explicit `://` and a nonempty authority qualify. Reject whitespace, controls, backslashes, credentials (including empty userinfo), malformed percent escapes and invalid host/port syntax before/after parsing. Return the parsed canonical destination with ASCII IDN hostname. No DNS, reputation check, preview or browser handoff occurs. The React test panel renders literal text and a destination paragraph, with no active link or Open action. Future M4 must revalidate that same destination immediately before opening it.

## Scope and limitations

Only synthetic bundled fixtures can be decoded from React. Phase 3.1 also provides a native capture preview, but this is not a working screen QR scanner. Crop coordinates, session ownership/cancellation, copy/open controls, shortcuts, tray, history and general image import are absent. Rotation, perspective, inverted symbols, legacy encodings and real-world damaged-code tolerance are not release-certified. Performance targets and Windows 10/11 clean-machine certification remain later P0 gates.

## Phase 3.1: native capture (user-verified)

`capture/native.rs` uses Windows-only `xcap 0.9.8` with default features disabled. Its GDI backend provides a one-shot RGBA frame without a recording session or WGC feature. This avoids writing another unsafe GDI resource-management layer. A small `windows 0.62` wrapper establishes/restores thread-local per-monitor DPI awareness and calls DwmFlush. Sources: [xcap](https://github.com/nashaofu/xcap), [BitBlt](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-bitblt).

Capture records monitor ID, signed physical origin, dimensions, orientation and scale. It checks metadata before and after capture and rejects changed displays or dimensions. The 32-megapixel cap bounds a raw RGBA frame to 128 MiB; encoding and IPC copies are additional. This is not a total memory/performance guarantee. Protected surfaces may be blank; only an explicit access-denied native error is labelled denial. No protection bypass or blank-image guessing is attempted.

`capture/workflow.rs` manages hide/capture/restore independently of Tauri and decoding. The main-window-only `capture_preview` command acquires a non-queuing permit, runs on a blocking worker, hides the window and checks visibility before waiting for compositor repaint plus 120 ms. Restoration runs after success/failure and during unwinding; restoration failure gets a controlled error with one best-effort retry. Window focus remains subject to Windows policy.

The temporary phase-test UI receives one PNG via binary IPC, not a JSON byte array, and keeps a Blob URL in memory. Native frame/encoding buffers leave scope after the response; React revokes the Blob URL on clear, replacement, failure or unmount. CSP allows blob images only; no filesystem, remote URL or generic window permissions are granted. No capture runs without an explicit button click. This diagnostic preview should be replaced by session-owned frozen-frame transport in 3.2. Escape/session cancellation, overlay selection and cropping belong to later phases and are absent here.
