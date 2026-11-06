#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if screenqr_lib::run().is_err() {
        // Never print native errors or potentially sensitive runtime data.
        std::process::exit(1);
    }
}
