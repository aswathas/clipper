//! Phone pairing: the session token, the LAN address, and the QR image.

use std::{
    net::UdpSocket,
    sync::{Arc, RwLock},
    time::Duration,
};

use crate::config::Config;

/// 32 characters from the OS random source.
pub fn new_token() -> String {
    use rand::Rng;

    const ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..32)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

/// The address a phone on the same network should dial.
///
/// Connecting a UDP socket transmits nothing; it only asks the OS which local
/// interface would be used to reach that address.
pub fn lan_ip() -> String {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local) = socket.local_addr() {
                if !local.ip().is_loopback() {
                    return local.ip().to_string();
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

/// Where the phone browser should be pointed.
pub fn pair_url(cfg: &Config, ip: &str, token: &str) -> String {
    format!("http://{ip}:{}/pair?token={token}", cfg.server.port)
}

/// A QR code for `data`, as PNG bytes.
pub fn qr_png(data: &str) -> Result<Vec<u8>, String> {
    use qrcode::{EcLevel, QrCode};

    let code = QrCode::with_error_correction_level(data.as_bytes(), EcLevel::M)
        .map_err(|e| e.to_string())?;

    let mut renderer = code.render::<image::Luma<u8>>();
    renderer.min_dimensions(320, 320);
    let image = renderer.build();

    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png)
}

/// Regenerate the session token once a day. Never logs the token itself.
pub async fn rotate_forever(token: Arc<RwLock<String>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        let fresh = new_token();
        *token.write().unwrap_or_else(|poisoned| poisoned.into_inner()) = fresh;
        println!("pairing: rotated the session token (24h)");
    }
}
