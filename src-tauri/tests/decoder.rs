use image::{GrayImage, ImageFormat, Luma};
use screenqr_lib::{
    decoder::{
        content::validated_destination,
        decode_image,
        types::{ContentType, DecodeResponse, ErrorCode},
        MAX_ENCODED_BYTES,
    },
    fixtures,
};
use std::io::Cursor;

#[test]
fn representative_images_preserve_exact_text() {
    for (id, expected) in [
        ("url", "HTTPS://example.com/qr?value=1&lang=th#result"),
        ("thai", "สวัสดี ชาวโลก เลขอ้างอิง ๑๒๓"),
        ("emoji", "Hello 👋 🌏 🚀"),
        ("multiline", "  first line\r\nบรรทัดสอง\nlast line\t \n"),
        ("html", "<script>alert('QR')</script>"),
    ] {
        let batch = decode_image(fixtures::image(id).unwrap()).unwrap();
        assert_eq!(batch.results.len(), 1);
        let result = &batch.results[0];
        assert_eq!(result.content, expected, "fixture {id}");
        assert_eq!(
            result.content_type,
            if id == "url" {
                ContentType::Url
            } else {
                ContentType::Text
            }
        );
    }
}

#[test]
fn negative_images_have_stable_errors() {
    for (id, expected) in [
        ("blank", ErrorCode::NoQrDetected),
        ("damaged", ErrorCode::MalformedQr),
        ("binary", ErrorCode::UnsupportedContent),
        ("invalid", ErrorCode::InvalidImage),
    ] {
        for _ in 0..2 {
            assert_eq!(
                decode_image(fixtures::image(id).unwrap()).unwrap_err().code,
                expected,
                "fixture {id}"
            );
        }
    }
}

#[test]
fn bounds_and_truncated_images_are_controlled() {
    assert_eq!(decode_image(&[]).unwrap_err().code, ErrorCode::InvalidImage);
    let png = fixtures::image("url").unwrap();
    assert_eq!(
        decode_image(&png[..png.len() / 2]).unwrap_err().code,
        ErrorCode::InvalidImage
    );
    assert_eq!(
        decode_image(&vec![0; MAX_ENCODED_BYTES + 1])
            .unwrap_err()
            .code,
        ErrorCode::ImageTooLarge
    );
    for (width, height) in [(4097, 1), (2100, 2100)] {
        let img = GrayImage::from_pixel(width, height, Luma([255]));
        let mut bytes = Cursor::new(Vec::new());
        img.write_to(&mut bytes, ImageFormat::Png).unwrap();
        assert_eq!(
            decode_image(bytes.get_ref()).unwrap_err().code,
            ErrorCode::ImageTooLarge
        );
    }
}

#[test]
fn duplicate_symbols_are_kept_as_separate_results() {
    let a = image::load_from_memory(fixtures::image("url").unwrap())
        .unwrap()
        .into_luma8();
    let mut pair = GrayImage::from_pixel(a.width() * 2 + 32, a.height(), Luma([255]));
    image::imageops::replace(&mut pair, &a, 0, 0);
    image::imageops::replace(&mut pair, &a, i64::from(a.width() + 32), 0);
    for img in [pair.clone(), image::imageops::rotate180(&pair)] {
        let mut bytes = Cursor::new(Vec::new());
        img.write_to(&mut bytes, ImageFormat::Png).unwrap();
        let batch = decode_image(bytes.get_ref()).unwrap();
        assert_eq!(batch.results.len(), 2);
        assert_eq!(batch.results[0], batch.results[1]);
    }
}

#[test]
fn urls_are_canonicalized_without_changing_source() {
    for (source, expected) in [
        (
            "HTTPS://Example.COM/a?q=1&x=2#part",
            "https://example.com/a?q=1&x=2#part",
        ),
        ("https://bücher.de/path", "https://xn--bcher-kva.de/path"),
        ("http://example.com", "http://example.com/"),
        ("https://[::1]:8080/a", "https://[::1]:8080/a"),
    ] {
        assert_eq!(validated_destination(source).as_deref(), Some(expected));
        let result = screenqr_lib::decoder::content::classify(source.to_owned());
        assert_eq!(result.content, source);
        assert_eq!(result.launch_destination.as_deref(), Some(expected));
    }
}

#[test]
fn unsafe_and_malformed_addresses_are_copy_only() {
    for source in [
        "",
        "example.com",
        "/relative",
        "//example.com",
        "javascript:alert(1)",
        "data:text/html,x",
        "file:///C:/secret",
        "mailto:a@example.com",
        "custom://host",
        "C:\\file",
        "cmd /c calc",
        " https://example.com",
        "https://example.com\n",
        "https://exa\tmple.com",
        "https://example.com/\u{7f}",
        "https:\\example.com",
        "https:/example.com",
        "https:example.com",
        "https:///example.com",
        "https://",
        "https://user:pass@example.com",
        "https://@example.com",
        "https://example.com:",
        "https://example.com:99999",
        "https://[bad]",
        "https://example.com/%zz",
        "https://example.com/%",
        "https://example.com/a b",
        "https://one.test https://two.test",
    ] {
        assert_eq!(validated_destination(source), None, "{source:?}");
    }
}

#[test]
fn fixture_allowlist_rejects_paths_and_oversized_ids() {
    for id in [
        "../url.png",
        "C:\\Windows\\win.ini",
        "https://example.com",
        "URL",
        "",
        &"x".repeat(10000),
    ] {
        assert_eq!(
            fixtures::image(id).unwrap_err().code,
            ErrorCode::InvalidFixture
        );
    }
}

#[test]
fn serialized_contract_has_tagged_success_and_error() {
    let response: DecodeResponse = decode_image(fixtures::image("url").unwrap()).into();
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(json["status"], "success");
    assert_eq!(json["result"]["results"][0]["type"], "url");
    assert_eq!(
        json["result"]["results"][0]["launchDestination"],
        "https://example.com/qr?value=1&lang=th#result"
    );
    let error: DecodeResponse = decode_image(b"invalid").into();
    let json = serde_json::to_value(error).unwrap();
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "invalid_image");
}

fn combined(ids: &[&str], vertical: bool) -> Vec<u8> {
    let images: Vec<_> = ids
        .iter()
        .map(|id| {
            image::load_from_memory(fixtures::image(id).unwrap())
                .unwrap()
                .into_luma8()
        })
        .collect();
    let width = if vertical {
        images.iter().map(|i| i.width()).max().unwrap()
    } else {
        images.iter().map(|i| i.width() + 32).sum()
    };
    let height = if vertical {
        images.iter().map(|i| i.height() + 32).sum()
    } else {
        images.iter().map(|i| i.height()).max().unwrap()
    };
    let mut canvas = GrayImage::from_pixel(width, height, Luma([255]));
    let mut offset = 0;
    for img in images {
        image::imageops::replace(
            &mut canvas,
            &img,
            if vertical { 0 } else { offset },
            if vertical { offset } else { 0 },
        );
        offset += i64::from(if vertical { img.height() } else { img.width() }) + 32;
    }
    let mut bytes = Cursor::new(Vec::new());
    canvas.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

#[test]
fn multiple_results_follow_spatial_order_and_classify_independently() {
    let batch = decode_image(fixtures::image("multiple").unwrap()).unwrap();
    assert_eq!(
        batch
            .results
            .iter()
            .map(|r| r.content.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    for vertical in [false, true] {
        let batch = decode_image(&combined(&["url", "html"], vertical)).unwrap();
        assert_eq!(batch.results.len(), 2);
        assert_eq!(batch.results[0].content_type, ContentType::Url);
        assert_eq!(batch.results[1].content_type, ContentType::Text);
        assert!(batch.results[1].launch_destination.is_none());
        assert_eq!((batch.unreadable_count, batch.unsupported_count), (0, 0));
    }
}

#[test]
fn partial_success_keeps_readable_results_and_counts_failures() {
    let batch = decode_image(&combined(&["url", "damaged", "binary"], false)).unwrap();
    assert_eq!(batch.results.len(), 1);
    assert_eq!(batch.results[0].content_type, ContentType::Url);
    assert_eq!((batch.unreadable_count, batch.unsupported_count), (1, 1));
    for ids in [["damaged", "binary"], ["binary", "damaged"]] {
        assert_eq!(
            decode_image(&combined(&ids, false)).unwrap_err().code,
            ErrorCode::MalformedQr
        );
    }
}
