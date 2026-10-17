# Feature: screenshots

`Ctrl+Shift+X` reads a clipboard image, writes a PNG to
`<vault>/Library/<id>.png`, and queues a row that links the
markdown stub. The stub is small — just the path, the size, the
source window title.

Muse also captures per-window screenshots during the watch, at a
configurable rate. The PNG goes to
`<vault>/.cache/screenshots/<window>__<ts>.png` and the path is
attached to the activity row.
