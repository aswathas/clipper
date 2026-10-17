# Roadmap

## Next (v0.7.0)
- Audio capture: local Whisper `ggml-base.en`, WASAPI loopback + mic
- Calls summary that lands in `<vault>/Calls/<date>.md`
- VAD-driven windowing to skip silence

## v0.8.0
- Inline preview pane in the Library
- Bulk action engine (regex rename, tag-from-rule, archive)
- Per-clip recallability score (model-based, on-device)

## v0.9.0
- Local embeddings via `all-MiniLM-L6-v2` ONNX
- Hybrid FTS + vector search in `/api/search`
- Auto-generated "weekly review" in `/today`

## v1.0
- Stable public API with versioned routes
- Optional encrypted vault (SQLite + markdown in a single `age`
  envelope)
- Plugin surface for third-party skill authors

## Will not do
- Cloud sync. Social. Mobile-first rewrite.
