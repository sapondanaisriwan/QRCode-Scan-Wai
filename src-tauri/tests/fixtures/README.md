# Synthetic QR fixtures

All PNGs are generated locally from synthetic, non-secret data by the dev-only Rust example:

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --example generate_fixtures --offline
cargo test --manifest-path src-tauri/Cargo.toml --offline
```

`qrcodegen 1.8.0` generates byte-mode symbols at low ECC with 8 pixels/module and a 4-module white border. Thai and emoji include ECI 26 (UTF-8); other text uses UTF-8 bytes without ECI. The generator is deterministic and no network/service creates the images. Tests consume committed PNG bytes; they do not regenerate images during a normal run.

| Fixture       | Expected result                                                                    |
| ------------- | ---------------------------------------------------------------------------------- |
| url.png       | Exact `HTTPS://example.com/qr?value=1&lang=th#result`; canonical HTTPS destination |
| thai.png      | Exact `สวัสดี ชาวโลก เลขอ้างอิง ๑๒๓`                                               |
| emoji.png     | Exact `Hello 👋 🌏 🚀`                                                             |
| multiline.png | `  first line\r\nบรรทัดสอง\nlast line\t \n`, preserving every character            |
| html.png      | Literal `<script>alert('QR')</script>`                                             |
| binary.png    | Bytes FF FE 80 00; unsupported_content                                             |
| blank.png     | White image; no_qr_detected                                                        |
| damaged.png   | Payload region erased while retaining finder patterns; malformed_qr                |
| multiple.png  | Separate first/second symbols; ordered results `first`, `second`                   |

Invalid bytes and truncated PNGs are supplied directly in tests. Oversized dimensions, duplicate symbols and rotated pairs are constructed in memory. No user screenshots or payloads are stored here.
