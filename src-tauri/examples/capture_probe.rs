//! Explicit developer smoke check. Prints metadata only; pixels are dropped in memory.
#[cfg(target_os = "windows")]
fn main() {
    match screenqr_lib::capture::capture_primary() {
        Ok(frame) => println!("Captured primary display: {}x{}, scale {}, origin ({}, {}). Pixels released without saving.",
            frame.display.width, frame.display.height, frame.display.scale, frame.display.x, frame.display.y),
        Err(error) => { eprintln!("Capture probe failed: {error:?}"); std::process::exit(1); }
    }
}
#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("Windows only");
}
