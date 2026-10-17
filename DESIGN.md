# Clipper — Design Specification v3 ("The Library")

_Generated under the Impeccable skill, Operate mode. One user, one device, calm focus, library aesthetic._

---

## 0. North star

> "It should feel like walking into a personal library at the end of a long day."

Not a productivity app. Not a feed. A **place**. The user opens it to feel oriented — what they saved, what they care about, what they should look at now. The interface recedes; the **content** has presence.

Reference products (for *feeling*, not to copy pixels):
- **Heptabase** — calm white space, serif for headings, graph view makes connections feel physical
- **Reflect** — editorial typography, the AI sidebar recedes, white-on-warm
- **Readwise Reader** — typographic hierarchy, article-first density
- **Notion's quiet mode** — restrained chrome, content wins

What we are **not**: a SaaS dashboard (no neon charts), a feed (no infinite scroll), a social product (no engagement metrics for their own sake).

---

## 1. Design tokens

All tokens exposed as CSS custom properties on `:root` (light) and `[data-theme="dark"]` (dark). No hard-coded hex anywhere in component CSS.

### Color (light, "warm paper")
| Token | Value | Use |
|---|---|---|
| `--bg` | `#FAF8F4` | page background, warm off-white |
| `--surface` | `#FFFFFF` | cards, panels |
| `--surface-2` | `#F4F1EB` | nested surfaces, inputs |
| `--ink` | `#1C1816` | primary text, warm near-black |
| `--ink-2` | `#3A332E` | secondary headings |
| `--mute` | `#6B6660` | body, captions |
| `--mute-2` | `#9A948C` | tertiary, disabled |
| `--hairline` | `#E8E4DC` | borders, dividers |
| `--accent` | `#B45309` | primary action, links on hover, focus ring |
| `--accent-soft` | `#FEF3E2` | selected row, badge background |
| `--danger` | `#B91C1C` | destructive, urgent deadlines |

### Color (dark, "lamplit room")
| Token | Value |
|---|---|
| `--bg` | `#15110E` |
| `--surface` | `#1E1A16` |
| `--surface-2` | `#261F1A` |
| `--ink` | `#ECE6DC` |
| `--ink-2` | `#D8D1C4` |
| `--mute` | `#9A948C` |
| `--mute-2` | `#5E5852` |
| `--hairline` | `#2D2823` |
| `--accent` | `#E89150` |
| `--accent-soft` | `#2A1F12` |
| `--danger` | `#F87171` |

### Type colors (per content type — used for the accent bar, icon tint, and tag pill border; never for body text)
| Type | Color |
|---|---|
| `article` | `#2E5C8A` (blue) |
| `video` | `#C2410C` (rose) |
| `book` | `#B45309` (amber) |
| `repo` | `#4A5D3F` (gray-green) |
| `code-error` | `#B91C1C` (red) |
| `photo` | `#6B21A8` (violet) |
| `event` | `#0F766E` (teal) |
| `paper` | `#4338CA` (indigo) |

In dark mode, each gets a +10% lightness bump.

### Typography
- **Display (headings, hero, article titles)**: `"Spectral", "Source Serif 4", Georgia, serif` — gives the library warmth
- **Body**: `"Inter", -apple-system, "Segoe UI", sans-serif`
- **Mono (code, prompts, paths)**: `"JetBrains Mono", ui-monospace, "Cascadia Code", monospace`

Sizes (modular, 1.2 ratio):
```
--fs-xs:  12px
--fs-sm:  13px
--fs-base:15px
--fs-md:  17px
--fs-lg:  20px
--fs-xl:  24px
--fs-2xl: 32px
--fs-3xl: 44px
--fs-4xl: 60px
```

Weights: 400 body, 500 UI, 600 headings, 700 display only.

Line-height: 1.55 body, 1.25 headings, 1.7 long-form.

Letter-spacing: `-0.011em` on `--fs-xl` and above, `0` below.

### Spacing (4px base)
`--sp-1: 4 · --sp-2: 8 · --sp-3: 12 · --sp-4: 16 · --sp-5: 24 · --sp-6: 32 · --sp-7: 48 · --sp-8: 64 · --sp-9: 96`

### Radii
`--r-sm: 4 · --r-md: 8 · --r-lg: 12 · --r-xl: 20`

### Elevation (shadows, used sparingly)
```
--shadow-1: 0 1px 2px rgba(28,24,22,.04)
--shadow-2: 0 4px 12px rgba(28,24,22,.06)
--shadow-3: 0 16px 48px rgba(28,24,22,.10)
```

### Motion
- Hover: 80ms ease-out
- Transition: 180ms ease-out (cards, panels, modals)
- Slide-over: 240ms ease-out
- Stagger (list items entering): 40ms apart, max 6 items
- No spring, no bounce, no parallax
- Reduced-motion respected

### Icons
- 1.5px stroke, 18px default, 20px in headers
- Inline SVG only, no icon font
- Type-icon set: 8 hand-tuned glyphs (article, video, book, repo, code-error, photo, event, paper), each with the type color applied as `currentColor` so it inherits context

---

## 2. App shell

### Three-zone desktop layout (≥1200px)
```
┌──────────┬──────────────────────────────────────────────┬──────────────┐
│          │                                              │              │
│  240px   │           fluid, max 920px content           │   320px      │
│  rail    │           (centered in viewport)             │   context    │
│          │                                              │              │
└──────────┴──────────────────────────────────────────────┴──────────────┘
   left rail             main content (page)               right rail
```

- **Left rail** (240px, fixed): app name at top in display serif, then primary nav, then secondary nav, then a small user-pill at the bottom showing the current lifecycle of the user's "mood" (e.g. "reading: 3, learning: 11, idle: 2"). Subtle border-right in `--hairline`.
- **Main**: page-specific. Max content width 920px, centered with 32px gutters.
- **Right rail** (320px, contextual): appears on Today and Ask, hidden on Library/Inbox/Skills/Settings. Shows the contextual side panel (focus items, related, sources).

### Tablet (≥900px, <1200px)
- Two zones: left rail (collapsible, 64px collapsed / 240px expanded via hover) + main. No right rail; context moves inline.

### Mobile (<900px)
- Single column, no rail. Top bar with logo, search icon, theme toggle, "Save" FAB. **Bottom tab bar** (5 slots: Today · Library · Inbox · Ask · More) with safe-area insets.

### Top bar (in main, per page)
- Left: page title in display serif (`--fs-xl`), below it a one-line description in mute (`--fs-sm`).
- Right: contextual actions for the page (Today: nothing. Library: filter chips. Inbox: bulk-action toolbar. Ask: new conversation. Skills: + Add. Settings: save button if dirty).

### Theme
- Light + dark, system pref default, manual toggle in left rail bottom area.
- Toggle is a small sun/moon icon button, no dropdown, no menu.

---

## 3. Page: **Today** (landing)

**Vibe**: A morning paper for your own life. Calm, oriented, not noisy.

```
┌──────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   Good evening, Aswath.                                             │  ← display serif, --fs-2xl
│   Thursday, 16 October                                               │  ← mute, --fs-sm
│                                                                      │
│   ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│   THIS WEEK                                                          │  ← caption, uppercase, --fs-xs, --mute-2, tracking .08em
│                                                                      │
│   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐         │
│   │   23           │  │    12          │  │   5h 40m       │         │  ← big numbers, display serif, --fs-3xl
│   │   saved        │  │   read         │  │   focus        │         │  ← caption below
│   │   ↑ 4 vs last  │  │   =            │  │   ↑ 1h 10m     │         │  ← delta vs last week, small
│   └────────────────┘  └────────────────┘  └────────────────┘         │
│                                                                      │
│   ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│   FOR YOU TODAY                                       5 items →     │
│                                                                      │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  ──── article · #rust · saved 2d                            │    │  ← accent bar (article blue)
│   │                                                             │    │
│   │  Rust ownership patterns in 2026                            │    │  ← title, display serif, --fs-lg
│   │  Why: you saved 3 similar this month                        │    │  ← "why this surfaced" in mute
│   │                                                             │    │
│   │  Borrow-checker improvements have shifted how...            │    │  ← 2-line excerpt, body
│   │                                                             │    │
│   │  #rust #borrow-checker  ·  8 min                           │    │  ← tags + reading time
│   └────────────────────────────────────────────────────────────┘    │
│   (× 5 of these, ranked by user model)                              │
│                                                                      │
│   ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│   STALE & FORGOTTEN                                                 │
│                                                                      │
│   14 items saved over 30 days ago, never opened.       [Review →]    │
│                                                                      │
│   ─────────────────────────────────────────────────────────────────  │
│                                                                      │
│   ACTIVITY  ─── last 90 days ───                                     │
│                                                                      │
│   ▁▁▂▂▃▅▆▇▆▅▃▂▂▁▁▁▁▁▂▃▃▂▁▁▁▁  (custom SVG sparkline, --accent)    │
│   Today            Last week            90-day total: 184            │
│                                                                      │
│   ┌─ Topics you lean on ─┐  ┌─ Decay distribution ─────────┐        │
│   │  rust    ████████ 47 │  │  fresh  ███████████ 28        │        │
│   │  ai      ██████   31 │  │  warm   █████       12        │        │
│   │  cooking ██        8 │  │  stale  ███          7       │        │
│   │  rust     ...  (8 more)            [Show all]   │        │
│   └─────────────────────┘  └──────────────────────────────┘        │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘

Right rail (320px):
┌──────────────────────────┐
│  CURRENT FOCUS            │  ← from memory.md
│                          │
│  Learn Tokio internals.  │  ← quote from memory.md
│  Read 1 chapter a day.   │
│                          │
│  Active projects (3) →   │
│                          │
│  TODAY'S DIGEST           │  ← user-model-derived
│                          │
│  • Catch up on borrow-   │
│    checker article       │
│  • The async/await piece │
│    you flagged last week │
│  • Close out 2 stale     │
│    Go items              │
│                          │
│  [Edit focus] [Read all] │
└──────────────────────────┘
```

### Rules
- The greeting uses the user's `user.name` from `user.md` if set, else "you".
- The greeting is time-aware: "Good morning" (5–12), "Good afternoon" (12–17), "Good evening" (17–24).
- The date is locale-formatted (e.g. "Thursday, 16 October" for en-GB; "Thu, Oct 16" for en-US).
- "For you today" cards are ranked by `user_model` (interest × priority × deadline_urgency), max 5.
- Each card's "Why: …" line is generated by Claude from the user model + clip metadata; if Claude is offline, fall back to "matched your interest in `<topic>`".
- Stale & forgotten: items with `decay_score > 0.7` AND `status='inbox'` AND `last_opened IS NULL`. Limit 5, plus count.
- Activity sparkline: 90 days, one bar per day, height = `saves_count` (capped), color = `--accent` for new saves, `--mute-2` for null. Custom SVG, no library.
- The whole page should feel like reading a book, not a dashboard.

### Empty state
- "Your library is quiet. The first save is the hardest — try it now."
- A small "Save your first URL" input box inline, with placeholder.

---

## 4. Page: **Library**

**Vibe**: Browsing a personal archive. Calm density. Cards have presence. You can see the whole room at once.

### Layout

```
┌──────────────────────────────────────────────────────────────────────┐
│  Library                                       234 items   view ⊞ ☰ │
│  Everything you've saved.                                          │
│                                                                      │
│  [ All 234 ] [ Articles 47 ] [ Videos 23 ] [ Books 12 ] [ Repos 18 ] │  ← horizontal chip rail
│  [ Errors 4 ] [ Photos 22 ] [ Events 8 ]   +  Tags ▾                │
│                                                                      │
│  Sort: Recent ▾    [grid ⊞] [list ☰]                                │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐   │
│  │  ▬               │  │  ▬               │  │  ▬               │   │  ← 6px type-color accent bar
│  │  📄 article      │  │  🎥 video        │  │  📄 article      │   │  ← icon + type label
│  │                  │  │                  │  │                  │   │
│  │  Rust ownership  │  │  Build a tiny    │  │  Why TDD is dead │   │  ← title, display serif, --fs-md
│  │  patterns 2026   │  │  LLM from scratch│  │  (again)         │   │
│  │                  │  │                  │  │                  │   │
│  │  Borrow-checker  │  │  Andrej Karpathy │  │  The premise was │   │  ← 2-line excerpt
│  │  improvements... │  │  walks through   │  │  never about...  │   │
│  │                  │  │                  │  │                  │   │
│  │  #rust #borrow   │  │  #ai #llm        │  │  #opinion #tdd   │   │  ← tag pills
│  │                  │  │                  │  │                  │   │
│  │  ──────          │  │  ───             │  │  ─               │   │  ← decay bar (green→amber→red)
│  │  2d · 8 min      │  │  1w · 47 min     │  │  3w · 6 min      │   │  ← age + reading time
│  └──────────────────┘  └──────────────────┘  └──────────────────┘   │
│  (more rows below; lazy-load as you scroll)                         │
└──────────────────────────────────────────────────────────────────────┘
```

### Card anatomy (320–360px wide, ~280px tall)
- 6px top accent bar in the type's color
- 16px padding all around
- Type icon + type label (uppercase, tracking .08em, --fs-xs, --mute-2), top-left
- Title in display serif, --fs-md, max 2 lines, ellipsized
- 2-line excerpt in body, --fs-sm, --mute, ellipsized
- Tag pills (rounded, --surface-2 background, no border, max 3 visible, "+N" if more)
- Decay bar at bottom, 4px tall, full width, color green→amber→red based on `decay_score`
- Age + reading time at bottom-right, --fs-xs, --mute-2

### States
- **Hover**: card lifts (`translateY(-2px)`), shadow bumps from `--shadow-1` to `--shadow-2`, accent bar thickens to 8px
- **Click**: opens detail slide-over from the right
- **Selected** (bulk): ring of `--accent` at 2px

### List view (toggle)
- Single column, denser rows: 56px tall, type icon, title (single line, ellipsized), tags inline, decay dot, date. Bulk-select on left.

### Filters
- Horizontal chip rail, scrolls horizontally if overflow
- "All" is selected by default; selecting a chip replaces
- Tags dropdown: search box, multi-select pills
- Sort: Recent / Oldest / Most-read / Deadline / Decay (highest first)

### Empty state
- "Nothing saved yet. The library grows one item at a time."
- Inline "Paste a URL" box.

### Mobile
- Single column grid, 1 card per row, full-width. Tags wrap.

---

## 5. Page: **Inbox** (triage)

**Vibe**: The desk where new mail lands. Quick decisions, then to Library.

### Layout
```
┌──────────────────────────────────────────────────────────────────────┐
│  Inbox                                       7 unread    [mark all] │
│  New saves, untriaged.                                             │
│                                                                      │
│  [ All ] [ Untagged 4 ] [ Needs deadline 2 ] [ Recent ]             │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ ☐  📄  article  Rust ownership patterns 2026                    │ │  ← row
│  │         en.wikipedia.org · 2d · #rust                          │ │
│  │                                              [Archive] [Tag]   │ │  ← hover actions
│  └────────────────────────────────────────────────────────────────┘ │
│  (more rows)                                                         │
│                                                                      │
│  ─── 3 selected ───────────────────────────────────────────────────  │  ← bulk action bar
│  [ Archive all ] [ Add tag ] [ Set deadline ] [ Delete ]            │
└──────────────────────────────────────────────────────────────────────┘
```

### Row (compact list)
- 64px tall, left checkbox (8px circle)
- Type icon + type label
- Title (single line, serif, --fs-md)
- Source + age + tags (single line, --fs-sm, --mute, ellipsized)
- Hover: action chips appear on the right (Archive / Tag / Open in Obsidian)

### Keyboard
- `j`/`k` move selection
- `a` archive
- `t` tag (opens tag input)
- `Enter` open detail
- `Shift+Click` for multi-select
- `/` focus search

### Mobile
- Rows full-width, action chips on tap-and-hold.

---

## 6. Page: **Ask** (chat)

**Vibe**: A long conversation with someone who knows you. Quiet, focused, citations matter.

### Layout
```
┌──────────────────┬──────────────────────────────────────────────────┐
│ Today            │  Ask your brain                                  │
│ Yesterday        │                                                  │
│                  │  ┌────────────────────────────────────────────┐   │
│ ▾ E0382 · ●      │  │ You: what did I save about Rust async?    │   │
│ Karpathy ·       │  │                                            │   │
│ TDD is dead      │  │ Clipper: based on 4 saved items, your     │   │
│ Astro catalog    │  │  Rust async interest centers on:           │   │
│ ...              │  │                                            │   │
│                  │  │  • "Rust ownership patterns" (8 min)      │   │
│ + new chat       │  │  • "Tokio in 100 lines" (47 min)          │   │
│                  │  │  • "Async without tokio" (article)         │   │
│                  │  │                                            │   │
│                  │  │ The borrow-checker shift toward ...        │   │
│                  │  │                                            │   │
│                  │  │ Sources (4): ▸                            │   │
│                  │  └────────────────────────────────────────────┘   │
│                  │                                                  │
│                  │  [ Ask anything…                          ] [Ask] │
└──────────────────┴──────────────────────────────────────────────────┘
```

### Chat thread (centered, max-width 720px)
- User messages: right-aligned, `--surface-2` bubble, --fs-base, no border
- Assistant messages: full-width, no bubble, --fs-base, body text reads like an essay
- Code blocks: mono, --surface-2, soft scroll, copy-on-hover
- Sources below each assistant message: collapsible, list of clip cards (title, type, link)
- Streaming: text appears word-by-word with a subtle cursor (1px, --accent), no "typing…" indicator

### Composer (sticky bottom)
- Auto-growing textarea, max 6 rows
- Send button on right, disabled when empty
- `Enter` sends, `Shift+Enter` newline
- Stop button replaces Send during streaming (cancels the in-flight request)

### Left rail (chat history)
- Recent conversations grouped: Today, Yesterday, This week, Older
- Each: clip title (if chat was scoped) or first user message (truncated)
- "New chat" button at top
- Click loads that conversation

### Mobile
- Left rail becomes a slide-in drawer from the left
- Right rail (sources) becomes a bottom sheet

### Empty state
- Centered: "Ask me anything about what you've saved. I'll find it, summarize it, or connect it."
- Three suggestion pills: "What's on my plate today?" / "What did I save about <topic>?" / "Summarize my last 5 reads"

---

## 7. Page: **Skills**

**Vibe**: A workbench. Each skill is a tool. Edit with care.

### Layout
- Card list, one per skill
- Each card: name (serif, --fs-md), one-line description, trigger badges, "Last used 3h ago, 47 times" stat
- Right side: `[Test]` `[Edit]` `[Disable]` buttons
- "+ Add skill" button at top-right
- Click "Edit" → slide-over with split view: YAML form (top) + body markdown editor (bottom) + live preview (right, sanitized via DOMPurify)
- "Test skill" button: paste a URL, run extraction, show the resulting markdown in a preview pane

---

## 8. Page: **Settings**

**Vibe**: Forms done with care. Type, save, done.

### Tabs (left rail of settings)
- **Soul** — persona/tone/avoid, markdown editor + live preview
- **Memory** — user.md + memory.md editors, "Regenerate" button (asks Claude to propose updates)
- **Self** — user_topics cloud, learned patterns grid, "Forget this" per pattern, "Recompute now" button
- **Providers** — list of CLI providers (active = claude), `[Test]` per provider, config.toml editor
- **Pairing** — LAN IP, current QR, TTL countdown, "Regenerate" confirm, devices list with revoke
- **Devices** — paired phones/tablets, last seen, revoke

### Form pattern
- Each tab is a single form with sections (h3, --fs-md, --ink-2)
- Each control: label (--fs-sm, --mute), input (--surface, --fs-base), help text below (--fs-xs, --mute-2)
- Save button sticky bottom-right, disabled until dirty
- "Saved" toast on success, inline error on failure

---

## 9. Detail slide-over

Right-side panel, 520px wide, slides in 240ms. Used everywhere: click any clip in Library, Inbox, or Today → opens here.

### Anatomy
```
┌────────────────────────────────────────────┐
│  [×]                                       │  ← close, top-left
│  ── article                                │  ← type label
│                                            │
│  Rust ownership patterns in 2026           │  ← title, display serif, --fs-2xl
│                                            │
│  en.wikipedia.org  ·  2 days ago           │  ← source + date, --fs-sm, --mute
│                                            │
│  #rust  #borrow-checker  #ownership  +add  │  ← tag pills, click to add/remove
│                                            │
│  ──────────────────────────────────────    │
│                                            │
│  [Body — rendered markdown in display     │
│  typography. Code blocks in mono, soft    │
│  scroll. Pull quotes, lists, etc.         │
│  Long-form reading at its own pace.]      │
│                                            │
│  ──────────────────────────────────────    │
│                                            │
│  ACTIONS                                   │
│  [Archive]  [Set deadline]  [Open in       │
│                              Obsidian]    │
│                                            │
│  ──────────────────────────────────────    │
│                                            │
│  RELATED                       (4)         │  ← 4 cards inline
│  ┌────────┐  ┌────────┐  ┌────────┐       │
│  │ ...    │  │ ...    │  │ ...    │       │
│  └────────┘  └────────┘  └────────┘       │
│                                            │
│  ──────────────────────────────────────    │
│                                            │
│  CHAT ABOUT THIS                  [Open ▾] │  ← expands inline chat
│                                            │
└────────────────────────────────────────────┘
```

- Background: `--surface`
- Shadow: `--shadow-3` to lift it
- Backdrop: page content dims to 50%, slight blur (4px)
- Esc closes
- Click outside closes
- Chat section: toggles open inline, full thread below the related list

---

## 10. Empty / loading / error states

### Empty (universal)
- Centered in the page area, max-width 480px
- Serif headline (--fs-xl), short body sentence, primary action button or inline input
- Never "No data" or "Nothing to show"

### Loading
- **Page-level**: full-page skeleton matching the layout, pulsing (1.5s ease-in-out, opacity 0.4 ↔ 0.7)
- **List-level**: 5 skeleton rows of the actual row shape, no spinner
- **Inline action**: button shows a 16px spinner, no text change

### Error
- Inline below the failed control: red border on the input, red --fs-xs message
- Page-level: full-page card with the error, "Try again" button, "Copy details" for support
- Network: toast bottom-right, "Connection lost — retrying…" with auto-retry every 10s, dismissable

---

## 11. Data visualizations

All custom SVG. No chart library. No d3. No chart.js.

### Sparkline (90-day activity on Today)
- 90 bars, each 4px wide, 1px gap
- Height proportional to daily save count, capped at 20
- Bar color: `--accent` if saves > 0, `--mute-2` (12% opacity) if 0
- Inline SVG, 720×60, viewBox 0 0 720 60

### Decay distribution (small bar list)
- 3 horizontal bars: Fresh / Warm / Stale
- Each bar full-width, height 8px, rounded
- Color: green / amber / red, with the actual proportion visible
- Count number to the right

### Topic distribution (horizontal bar list, top 8)
- 8 rows, each `topic — bar — count`
- Bar proportional to count, max 200px wide
- Topic in --ink, bar in --accent-soft with --accent fill at 40% width

### Calendar heatmap (Settings → Self, optional)
- 12 weeks × 7 days
- Each cell 12px square, 2px gap
- Color: --mute-2 with opacity from 0.1 (no saves) to 0.8 (5+ saves)
- Hover: tooltip with date + count

### Decay dot (used in list view, detail, etc.)
- 8px circle
- Color: green (≤10d), amber (≤20d), red (>20d)

### All charts are calm. None of them move. None of them have animation. The data should feel like it sits still, like a page in a book.

---

## 12. Motion

- **Hover**: `transition: background-color 80ms ease-out, border-color 80ms ease-out, transform 80ms ease-out;` — only on things that respond to hover
- **Cards**: `transition: transform 180ms ease-out, box-shadow 180ms ease-out;`
- **Slide-over**: `transition: transform 240ms ease-out, opacity 240ms ease-out;`
- **Modal**: `transition: opacity 180ms ease-out;` (no scale)
- **Tab change**: instant (no animation — calm, fast)
- **Streamed chat text**: 1px cursor blinks at 1s interval during streaming
- **Reduced motion**: `prefers-reduced-motion: reduce` → kill all transitions, instant

---

## 13. Accessibility

- All interactive elements are real `<button>`, `<a>`, `<input>`, or `role="button"` with `tabindex="0"`
- Focus ring: 2px solid `--accent`, 2px offset, never removed
- Color contrast: 4.5:1 minimum for body, 3:1 for large text. All tokens above meet this.
- Skip link: hidden until focused, jumps to main content
- Headings: one h1 per page (the page title), h2 for sections, h3 for cards
- Live regions: chat streaming uses `aria-live="polite"`, toasts `aria-live="assertive"`
- No information conveyed by color alone (decay dot has shape, urgency has text)
- All icons have `aria-label` or are wrapped in a label

---

## 14. Mobile-specific rules

- Single column, full width, 16px gutters
- Bottom tab bar (5 items, fixed, safe-area-inset-bottom)
- FAB for primary action (Save) bottom-right
- Top bar: logo + search + theme toggle (max 3 icons)
- Slide-overs become full-page sheets
- No hover states (replace with active states)
- Touch targets ≥ 44px
- No two-finger gestures required

---

## 15. Performance budget

- Initial dashboard render: <100ms (data + skeleton swap)
- Route change: <50ms (Alpine show/hide, no page reload)
- Slide-over: <300ms total
- Search debounce: 200ms
- Sparkline renders: <10ms (pure SVG, no JS lib)
- Total JS: ≤ 80KB gzipped (Alpine + Pico + app)
- Total CSS: ≤ 40KB gzipped
- Lighthouse perf score target: ≥ 90

---

## 16. Build order (for the implementation agent)

1. **Tokens** (CSS file) — every custom property, both themes, full set
2. **Fonts** — Google Fonts link, preconnect, swap
3. **App shell** — left rail, main, right rail, top bar, mobile bottom nav
4. **Type icons** — 8 inline SVGs, single component
5. **Today page** — greeting, week stats, For You, stale, activity, topics
6. **Library page** — chip rail, grid + list toggle, card component, filters
7. **Inbox page** — bulk select, action bar
8. **Ask page** — left rail history, thread, composer
9. **Detail slide-over** — used by all
10. **Skills + Settings pages** — last
11. **Empty / loading / error states** — universal component, applied throughout
12. **Reduced-motion + a11y pass**

Each step is one commit. Stop and test the page in a browser after each step. Don't batch.

---

## 17. Anti-patterns (do not ship)

- ❌ No emoji as icons (use the inline SVGs)
- ❌ No chart library
- ❌ No React, no Vue, no Svelte (Alpine only)
- ❌ No CSS framework beyond Pico (and even that we may outgrow)
- ❌ No infinite scroll (use pagination or "load more")
- ❌ No dark patterns, no gamification, no streaks
- ❌ No notification badges except unread inbox
- ❌ No "AI suggested you also…" feeds
- ❌ No loading spinners (skeletons only)
- ❌ No hard-coded hex (use tokens)
- ❌ No `git add -A` (scoped adds only)
- ❌ No `git add .` (scoped adds only)

---

## 18. North star (re-stated)

> It should feel like a personal library — calm, editorial, content-forward. The user opens it to feel oriented, not busy. Every pixel earns its place.

Build that.
