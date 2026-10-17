# Feature: self-improvement

At startup and once a day at 03:00 local time, the daemon
recomputes a small summary of the vault's statistics (clip count
by day, top hosts, top tags) and writes it to
`<vault>/.cache/stats.json`. The dashboard reads this for the
"this week" widget.

There is no ML in the loop. The summary is deterministic. We do
not want the index to drift because a model had a bad day.
