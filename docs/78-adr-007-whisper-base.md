# ADR 007: local Whisper, base.en

**Status:** accepted.

**Context.** The audio capture feature needs a speech-to-text
model. Candidates are `whisper-rs` with `ggml-base.en`, Vosk,
and a cloud API.

**Decision.** `ggml-base.en` via `whisper-rs`.

**Consequences.** 141 MB on disk, 376 MB peak RSS, WER 2.07%
on librispeech-asr clean. CPU floor at idle is 0. RTF 0.24 on a
Ryzen 7 5700X. No network. MIT license.

**Alternatives.** Vosk (faster, weaker on accented speech), a
cloud API (privacy, latency, cost).
