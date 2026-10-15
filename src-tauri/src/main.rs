// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `clipper.exe --hag<N>-smoke` runs that slice's self-check and exits.
    for arg in std::env::args() {
        let code = match arg.as_str() {
            "--hag3-smoke" => Some(clipper_lib::selfcheck::run_hag3()),
            "--hag4-smoke" => Some(clipper_lib::selfcheck::run_hag4()),
            "--hag6-smoke" => Some(clipper_lib::selfcheck::run_hag6()),
            "--hag7-smoke" => Some(clipper_lib::selfcheck::run_hag7()),
            "--hag10-smoke" => Some(clipper_lib::selfcheck::run_hag10()),
            "--integration-smoke" => Some(clipper_lib::selfcheck::integration_api()),
            "--hag14-smoke" => Some(clipper_lib::selfcheck::run_hag14()),
            "--hag15-smoke" => Some(clipper_lib::selfcheck::run_hag15()),
            "--hag16-smoke" => Some(clipper_lib::selfcheck::run_hag16()),
            _ => None,
        };
        if let Some(code) = code {
            std::process::exit(code);
        }
    }

    clipper_lib::run()
}
