# Feature: memory

A single markdown file at `<vault>/Memory/user.md` plus a
machine-readable view in the `memory` table. The file is the
source of truth. The table is a denormalized view that the Ask
surface reads at runtime.

Memory holds three sections:
- **People** — names + relationships + last interaction
- **Projects** — names + status + next action
- **Preferences** — tools, habits, "I never want to..."

Editing the file in Obsidian updates the table on the next index
pass.
