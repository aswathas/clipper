# Feature: call capture (v0.7)

Toggle `record_calls` in Muse settings. When a known call
process (Zoom, Teams, Slack huddle, Discord) is foregrounded,
the watcher opens a WASAPI loopback + mic stream, downsamples
to 16 kHz mono, and buffers 30 s windows. Each window is
transcribed by `ggml-base.en` (local, on-device) and appended
to `<vault>/Calls/<date>.md`.

The transcription is local. No audio ever leaves the machine.
