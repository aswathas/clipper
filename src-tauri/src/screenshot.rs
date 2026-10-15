//! Screenshot capture: clipboard image -> PNG in staging -> queue entry.

use std::io;

use crate::{
    clock,
    config::Config,
    queue::{self, QueueEntry},
};

/// The clipboard image, as raw RGBA8, if it holds one.
pub fn clipboard_image() -> Option<(usize, usize, Vec<u8>)> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let image = clipboard.get_image().ok()?;
    Some((
        image.width,
        image.height,
        image.bytes.into_owned(),
    ))
}

/// Encode an RGBA buffer to `~/.clipper/staging/img_<unix_ts>.png` and queue it.
///
/// Returns the queue entry id, or `None` when the buffer is not a valid image.
pub fn save_and_queue(
    cfg: &Config,
    width: usize,
    height: usize,
    bytes: Vec<u8>,
    source: &str,
) -> io::Result<Option<String>> {
    if width == 0 || height == 0 {
        return Ok(None);
    }
    let Some(buffer) = image::RgbaImage::from_raw(width as u32, height as u32, bytes) else {
        return Ok(None);
    };

    cfg.ensure_dirs()?;
    let id = uuid::Uuid::new_v4().to_string();

    // `img_<unix_ts>.png`, with a suffix only if that second is already taken.
    let stamp = clock::now_unix();
    let mut path = cfg.staging_dir().join(format!("img_{stamp}.png"));
    if path.exists() {
        let short = id.split('-').next().unwrap_or("dup");
        path = cfg.staging_dir().join(format!("img_{stamp}-{short}.png"));
    }

    buffer.save(&path).map_err(io::Error::other)?;

    let entry = QueueEntry {
        id: id.clone(),
        url: None,
        path: Some(path.to_string_lossy().into_owned()),
        source: source.to_string(),
        kind: "image".to_string(),
        saved_at: clock::now_iso8601(),
    };
    queue::append_json_line(&cfg.inbox_jsonl(), &entry)?;

    Ok(Some(id))
}
