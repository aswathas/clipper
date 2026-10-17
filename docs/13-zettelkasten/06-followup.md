# What we did not borrow

Luhmann's slip-box used a fixed physical address per note. We use
content-addressed ids (UUIDs) so the address is stable across
renames.

We did not borrow the idea of a "branch" — Luhmann split a note
into multiple notes when it had two ideas. Our drainer writes
atomic notes from the start.

We did not borrow the idea of a "sequence" — Luhmann numbered
notes to chain arguments. The user can do this with `[[id1]] →
[[id2]]` in the body if they want.
