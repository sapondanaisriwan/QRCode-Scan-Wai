//! Pure geometry and in-memory crop; independent of Tauri, capture APIs and QR decoding.
use image::RgbaImage;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropError {
    InvalidCoordinates,
    EmptySelection,
    OutOfBounds,
}

/// Overlay-local CSS/logical points: apply the native overlay scale exactly once.
pub fn from_overlay(
    a: Point,
    b: Point,
    scale: f64,
    width: u32,
    height: u32,
) -> Result<PixelRect, CropError> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(CropError::InvalidCoordinates);
    }
    pixel_bounds(
        Point {
            x: a.x * scale,
            y: a.y * scale,
        },
        Point {
            x: b.x * scale,
            y: b.y * scale,
        },
        width,
        height,
    )
}
/// Physical desktop points: subtract the signed monitor origin; never apply DPI again.
pub fn from_desktop(
    a: Point,
    b: Point,
    origin: (i32, i32),
    width: u32,
    height: u32,
) -> Result<PixelRect, CropError> {
    pixel_bounds(
        Point {
            x: a.x - f64::from(origin.0),
            y: a.y - f64::from(origin.1),
        },
        Point {
            x: b.x - f64::from(origin.0),
            y: b.y - f64::from(origin.1),
        },
        width,
        height,
    )
}
fn pixel_bounds(a: Point, b: Point, width: u32, height: u32) -> Result<PixelRect, CropError> {
    if width == 0 || height == 0 || ![a.x, a.y, b.x, b.y].iter().all(|v| v.is_finite()) {
        return Err(CropError::InvalidCoordinates);
    }
    let left = a.x.min(b.x).clamp(0.0, f64::from(width));
    let right = a.x.max(b.x).clamp(0.0, f64::from(width));
    let top = a.y.min(b.y).clamp(0.0, f64::from(height));
    let bottom = a.y.max(b.y).clamp(0.0, f64::from(height));
    // Check before rounding: a fractional zero-width line is still empty.
    if right <= left || bottom <= top {
        return Err(CropError::EmptySelection);
    }
    let x = left.floor() as u32;
    let y = top.floor() as u32;
    Ok(PixelRect {
        x,
        y,
        width: right.ceil() as u32 - x,
        height: bottom.ceil() as u32 - y,
    })
}
pub fn crop_rgba(frame: &RgbaImage, rect: PixelRect) -> Result<RgbaImage, CropError> {
    if rect.width == 0 || rect.height == 0 {
        return Err(CropError::EmptySelection);
    }
    if rect
        .x
        .checked_add(rect.width)
        .is_none_or(|right| right > frame.width())
        || rect
            .y
            .checked_add(rect.height)
            .is_none_or(|bottom| bottom > frame.height())
    {
        return Err(CropError::OutOfBounds);
    }
    Ok(image::imageops::crop_imm(frame, rect.x, rect.y, rect.width, rect.height).to_image())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_directions_and_dpi_scales() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            for (x1, x2) in [(10.0, 30.0), (30.0, 10.0)] {
                for (y1, y2) in [(20.0, 40.0), (40.0, 20.0)] {
                    let rect = from_overlay(
                        Point { x: x1, y: y1 },
                        Point { x: x2, y: y2 },
                        scale,
                        200,
                        200,
                    )
                    .unwrap();
                    assert_eq!(rect.x, (10.0 * scale).floor() as u32);
                    assert_eq!(rect.y, (20.0 * scale).floor() as u32);
                    assert_eq!(rect.x + rect.width, (30.0 * scale).ceil() as u32);
                    assert_eq!(rect.y + rect.height, (40.0 * scale).ceil() as u32);
                }
            }
        }
    }
    #[test]
    fn negative_origins_are_subtracted_without_scaling_physical_coordinates() {
        let local = from_overlay(
            Point { x: 100.0, y: 20.0 },
            Point { x: 200.0, y: 40.0 },
            1.5,
            1920,
            1080,
        )
        .unwrap();
        let desktop = from_desktop(
            Point {
                x: -1770.0,
                y: -1050.0,
            },
            Point {
                x: -1620.0,
                y: -1020.0,
            },
            (-1920, -1080),
            1920,
            1080,
        )
        .unwrap();
        assert_eq!(local, desktop);
    }
    #[test]
    fn fractional_edges_clamp_and_empty_is_checked_before_rounding() {
        assert_eq!(
            from_overlay(
                Point { x: -5.0, y: 1.1 },
                Point { x: 200.0, y: 2.1 },
                1.0,
                100,
                50
            ),
            Ok(PixelRect {
                x: 0,
                y: 1,
                width: 100,
                height: 2
            })
        );
        assert_eq!(
            from_overlay(
                Point { x: 1.2, y: 1.0 },
                Point { x: 1.2, y: 9.0 },
                1.5,
                100,
                50
            ),
            Err(CropError::EmptySelection)
        );
        assert_eq!(
            from_overlay(
                Point { x: 101.0, y: 1.0 },
                Point { x: 110.0, y: 9.0 },
                1.0,
                100,
                50
            ),
            Err(CropError::EmptySelection)
        );
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                from_overlay(
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 1.0, y: 1.0 },
                    scale,
                    100,
                    50
                ),
                Err(CropError::InvalidCoordinates)
            );
        }
        assert_eq!(
            from_overlay(
                Point {
                    x: f64::NAN,
                    y: 0.0
                },
                Point { x: 1.0, y: 1.0 },
                1.0,
                100,
                50
            ),
            Err(CropError::InvalidCoordinates)
        );
        assert_eq!(
            from_overlay(
                Point {
                    x: f64::MAX,
                    y: 0.0
                },
                Point { x: 1.0, y: 1.0 },
                2.0,
                100,
                50
            ),
            Err(CropError::InvalidCoordinates)
        );
    }
    #[test]
    fn marker_pixels_verify_half_open_crop_in_landscape_and_portrait() {
        for (width, height) in [(8, 12), (12, 8)] {
            let frame = RgbaImage::from_fn(width, height, |x, y| {
                image::Rgba([x as u8, y as u8, 42, 255])
            });
            let rect = from_overlay(
                Point { x: 1.0, y: 2.0 },
                Point { x: 3.0, y: 4.0 },
                1.5,
                width,
                height,
            )
            .unwrap();
            let crop = crop_rgba(&frame, rect).unwrap();
            assert_eq!(crop.dimensions(), (4, 3));
            assert_eq!(*crop.get_pixel(0, 0), image::Rgba([1, 3, 42, 255]));
            assert_eq!(*crop.get_pixel(3, 2), image::Rgba([4, 5, 42, 255]));
            assert_eq!(
                crop_rgba(
                    &frame,
                    PixelRect {
                        x: 0,
                        y: 0,
                        width,
                        height
                    }
                )
                .unwrap(),
                frame
            );
            assert!(matches!(
                crop_rgba(
                    &frame,
                    PixelRect {
                        x: u32::MAX,
                        y: 0,
                        width: 2,
                        height: 1
                    }
                ),
                Err(CropError::OutOfBounds)
            ));
            assert!(matches!(
                crop_rgba(
                    &frame,
                    PixelRect {
                        x: width,
                        y: 0,
                        width: 1,
                        height: 1
                    }
                ),
                Err(CropError::OutOfBounds)
            ));
        }
    }
}
