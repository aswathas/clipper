// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `clipper.exe --hag3-smoke` runs the HAG-3 self-check and exits.
    if std::env::args().any(|arg| arg == "--hag3-smoke") {
        std::process::exit(clipper_lib::selfcheck::run());
    }

    clipper_lib::run()
}
