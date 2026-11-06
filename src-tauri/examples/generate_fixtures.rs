//! Reproducible synthetic test data only; never used by the application.
use image::{GrayImage, Luma};
use qrcodegen::{QrCode, QrCodeEcc, QrSegment};
use std::{error::Error, path::Path};

fn render(data: &[u8], eci: bool) -> Result<GrayImage, Box<dyn Error>> {
    let mut segments = Vec::new();
    if eci {
        segments.push(QrSegment::make_eci(26));
    }
    segments.push(QrSegment::make_bytes(data));
    let qr = QrCode::encode_segments(&segments, QrCodeEcc::Low)?;
    let size = (qr.size() + 8) as u32 * 8;
    Ok(GrayImage::from_fn(size, size, |x, y| {
        Luma([if qr.get_module((x / 8) as i32 - 4, (y / 8) as i32 - 4) {
            0
        } else {
            255
        }])
    }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    std::fs::create_dir_all(&dir)?;
    for (name, text, eci) in [
        (
            "url",
            "HTTPS://example.com/qr?value=1&lang=th#result",
            false,
        ),
        ("thai", "สวัสดี ชาวโลก เลขอ้างอิง ๑๒๓", true),
        ("emoji", "Hello 👋 🌏 🚀", true),
        (
            "multiline",
            "  first line\r\nบรรทัดสอง\nlast line\t \n",
            false,
        ),
        ("html", "<script>alert('QR')</script>", false),
    ] {
        render(text.as_bytes(), eci)?.save(dir.join(format!("{name}.png")))?;
    }
    render(&[0xff, 0xfe, 0x80, 0x00], false)?.save(dir.join("binary.png"))?;
    GrayImage::from_pixel(256, 256, Luma([255])).save(dir.join("blank.png"))?;
    let a = render(b"first", false)?;
    let b = render(b"second", false)?;
    let mut combined = GrayImage::from_pixel(
        a.width() + b.width() + 32,
        a.height().max(b.height()),
        Luma([255]),
    );
    image::imageops::replace(&mut combined, &a, 0, 0);
    image::imageops::replace(&mut combined, &b, i64::from(a.width() + 32), 0);
    combined.save(dir.join("multiple.png"))?;
    let mut damaged = render(b"Deliberately damaged QR payload for negative test", false)?;
    // Erase the interior payload while retaining all three finder patterns.
    for y in 104..168 {
        for x in 104..168 {
            damaged.put_pixel(x, y, Luma([255]));
        }
    }
    damaged.save(dir.join("damaged.png"))?;
    Ok(())
}
