use screenqr_lib::{
    capture::crop::{crop_rgba, PixelRect},
    decoder::{self, types::ErrorCode},
    fixtures,
};

#[test]
fn native_crop_preserves_all_fixture_results_and_errors() {
    for id in [
        "url",
        "thai",
        "emoji",
        "multiline",
        "html",
        "multiple",
        "blank",
        "damaged",
        "binary",
    ] {
        let bytes = fixtures::image(id).unwrap();
        let image = image::load_from_memory(bytes).unwrap().into_rgba8();
        let mut desktop = image::RgbaImage::from_pixel(
            image.width() + 100,
            image.height() + 100,
            image::Rgba([255, 255, 255, 255]),
        );
        image::imageops::replace(&mut desktop, &image, 40, 30);
        let crop = crop_rgba(
            &desktop,
            PixelRect {
                x: 40,
                y: 30,
                width: image.width(),
                height: image.height(),
            },
        )
        .unwrap();
        assert_eq!(
            decoder::decode_rgba(crop),
            decoder::decode_image(bytes),
            "{id}"
        );
    }
}
#[test]
fn qr_outside_selected_region_is_not_decoded() {
    let image = image::load_from_memory(fixtures::image("url").unwrap())
        .unwrap()
        .into_rgba8();
    let mut desktop = image::RgbaImage::from_pixel(
        image.width() + 200,
        image.height(),
        image::Rgba([255, 255, 255, 255]),
    );
    image::imageops::replace(&mut desktop, &image, 200, 0);
    let crop = crop_rgba(
        &desktop,
        PixelRect {
            x: 0,
            y: 0,
            width: 150,
            height: desktop.height(),
        },
    )
    .unwrap();
    assert_eq!(
        decoder::decode_rgba(crop).unwrap_err().code,
        ErrorCode::NoQrDetected
    );
}
#[test]
fn native_decoder_rejects_empty_or_oversized_images() {
    assert_eq!(
        decoder::decode_rgba(image::RgbaImage::new(0, 1))
            .unwrap_err()
            .code,
        ErrorCode::InvalidImage
    );
    assert_eq!(
        decoder::decode_rgba(image::RgbaImage::new(4097, 1))
            .unwrap_err()
            .code,
        ErrorCode::ImageTooLarge
    );
    assert_eq!(
        decoder::validate_dimensions(3000, 3000).unwrap_err().code,
        ErrorCode::ImageTooLarge
    );
}
