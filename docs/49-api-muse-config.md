# API: muse/config

- `GET /api/muse/config` — current `MuseCfg`.
- `POST /api/muse/config` — partial update. Unknown fields are
  rejected with `400`.

Fields: `skip_processes`, `smart_dedup`, `debounce_secs`,
`screenshots`, `record_calls`, `capture_whatsapp`.
