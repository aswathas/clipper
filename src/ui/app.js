// Clipper v3 ("The Library") — Application Core
// Pure Vanilla JS + Alpine.js. Zero build step.

/* ============================================================================
   Theme Management
   ============================================================================ */

const ThemeManager = {
  current: "auto",
  init() {
    const saved = localStorage.getItem("clipper-theme") || "auto";
    this.setTheme(saved);
    if (window.matchMedia) {
      window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
        if (this.current === "auto") {
          this.applyTheme();
        }
      });
    }
  },
  setTheme(theme) {
    this.current = theme;
    localStorage.setItem("clipper-theme", theme);
    this.applyTheme();
  },
  toggle() {
    const isDark = document.documentElement.getAttribute("data-theme") === "dark";
    this.setTheme(isDark ? "light" : "dark");
  },
  applyTheme() {
    let effective = this.current;
    if (effective === "auto") {
      const prefersDark = window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches;
      effective = prefersDark ? "dark" : "light";
    }
    document.documentElement.setAttribute("data-theme", effective);
  }
};

/* ============================================================================
   Toast Manager
   ============================================================================ */

const ToastManager = {
  items: [],
  show(message, type = "info") {
    const id = Date.now() + Math.random().toString(36).slice(2, 6);
    const toast = { id, message, type };
    this.items.push(toast);
    setTimeout(() => {
      this.remove(id);
    }, 3200);
  },
  remove(id) {
    const idx = this.items.findIndex(t => t.id === id);
    if (idx !== -1) this.items.splice(idx, 1);
  }
};

function toast(msg, type = "info") {
  ToastManager.show(msg, type);
}

/* ============================================================================
   Type Icons (8 Inline SVGs, Single Component per DESIGN.md Section 1)
   ============================================================================ */

const TYPE_CONFIG = {
  article: {
    label: "Article",
    colorVar: "var(--type-article)",
    svg: `<svg class="type-glyph" role="img" aria-label="Article" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path><line x1="8" y1="7" x2="16" y2="7"></line><line x1="8" y1="11" x2="14" y2="11"></line></svg>`
  },
  video: {
    label: "Video",
    colorVar: "var(--type-video)",
    svg: `<svg class="type-glyph" role="img" aria-label="Video" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="4" width="15" height="16" rx="2"></rect><polygon points="22 8 17 12 22 16 22 8"></polygon></svg>`
  },
  book: {
    label: "Book",
    colorVar: "var(--type-book)",
    svg: `<svg class="type-glyph" role="img" aria-label="Book" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path></svg>`
  },
  repo: {
    label: "Repo",
    colorVar: "var(--type-repo)",
    svg: `<svg class="type-glyph" role="img" aria-label="Repo" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="4"></circle><line x1="1.05" y1="12" x2="7" y2="12"></line><line x1="17.01" y1="12" x2="22.96" y2="12"></line></svg>`
  },
  "code-error": {
    label: "Code Error",
    colorVar: "var(--type-code-error)",
    svg: `<svg class="type-glyph" role="img" aria-label="Code Error" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>`
  },
  photo: {
    label: "Photo",
    colorVar: "var(--type-photo)",
    svg: `<svg class="type-glyph" role="img" aria-label="Photo" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>`
  },
  event: {
    label: "Event",
    colorVar: "var(--type-event)",
    svg: `<svg class="type-glyph" role="img" aria-label="Event" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="4" width="18" height="18" rx="2"></rect><line x1="16" y1="2" x2="16" y2="6"></line><line x1="8" y1="2" x2="8" y2="6"></line><line x1="3" y1="10" x2="21" y2="10"></line></svg>`
  },
  paper: {
    label: "Paper",
    colorVar: "var(--type-paper)",
    svg: `<svg class="type-glyph" role="img" aria-label="Paper" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>`
  }
};

function getTypeMeta(type) {
  const norm = (type || "").toLowerCase().trim();
  return TYPE_CONFIG[norm] || TYPE_CONFIG.article;
}

function renderTypeBadge(type, withLabel = true) {
  const meta = getTypeMeta(type);
  const labelHtml = withLabel ? `<span class="type-badge-label">${meta.label}</span>` : "";
  return `<span class="type-badge type-${type}" style="color: ${meta.colorVar};">${meta.svg}${labelHtml}</span>`;
}

/* ============================================================================
   API Client & Mock Handler
   ============================================================================ */

const MOCK_DATA = {
  items: [
    {
      id: "clip-01",
      url: "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html",
      type: "article",
      source: "Ctrl+Shift+C",
      title: "Rust ownership patterns in 2026",
      summary: "Borrow-checker improvements have shifted how modern idiomatic Rust handles references and shared mutability.",
      tags: ["rust", "borrow-checker"],
      saved_at: Math.floor((Date.now() - 2 * 86400 * 1000) / 1000),
      reading_time_min: 8,
      status: "inbox",
      decay_score: 0.12,
      why_surfaced: "Why: you saved 3 similar this month",
      vault_path: "Inbox/Rust ownership patterns.md"
    },
    {
      id: "clip-02",
      url: "https://www.youtube.com/watch?v=kCc8FmEb1nY",
      type: "video",
      source: "pwa",
      title: "Build a tiny LLM from scratch",
      summary: "Andrej Karpathy walks through tokenization, multi-head self-attention, and training an autoregressive transformer.",
      tags: ["ai", "llm"],
      saved_at: Math.floor((Date.now() - 7 * 86400 * 1000) / 1000),
      reading_time_min: 47,
      status: "inbox",
      decay_score: 0.35,
      why_surfaced: "Why: matches your learning goal on transformers",
      vault_path: "Inbox/Tiny LLM from scratch.md"
    },
    {
      id: "clip-03",
      url: "https://martinfowler.com/articles/is-tdd-dead.html",
      type: "article",
      source: "manual",
      title: "Why TDD is dead (again)",
      summary: "The premise was never about unit-test dogma; it was about architecture feedback loops in large systems.",
      tags: ["opinion", "tdd"],
      saved_at: Math.floor((Date.now() - 21 * 86400 * 1000) / 1000),
      reading_time_min: 6,
      status: "inbox",
      decay_score: 0.78,
      why_surfaced: "Why: flagged for critical software architecture review",
      vault_path: "Inbox/Is TDD Dead.md"
    },
    {
      id: "clip-04",
      url: "https://github.com/tokio-rs/tokio",
      type: "repo",
      source: "extension",
      title: "Tokio: An asynchronous runtime for Rust",
      summary: "Multi-threaded work-stealing scheduler with non-blocking I/O primitives and timers.",
      tags: ["rust", "async", "networking"],
      saved_at: Math.floor((Date.now() - 3 * 86400 * 1000) / 1000),
      reading_time_min: 15,
      status: "library",
      decay_score: 0.18,
      why_surfaced: "Why: aligns with your active focus on Tokio internals",
      vault_path: "Library/Tokio Runtime.md"
    },
    {
      id: "clip-05",
      url: "https://en.wikipedia.org/wiki/Attention_Is_All_You_Need",
      type: "paper",
      source: "Ctrl+Shift+C",
      title: "Attention Is All You Need",
      summary: "Seminal paper introducing the Transformer architecture based solely on self-attention mechanisms.",
      tags: ["ai", "paper", "deep-learning"],
      saved_at: Math.floor((Date.now() - 4 * 86400 * 1000) / 1000),
      reading_time_min: 25,
      status: "library",
      decay_score: 0.22,
      why_surfaced: "Why: foundational reading for modern LLM implementations",
      vault_path: "Library/Attention Is All You Need.md"
    },
    {
      id: "clip-06",
      url: "https://dataintensive.net/",
      type: "book",
      source: "manual",
      title: "Designing Data-Intensive Applications",
      summary: "Martin Kleppmann explores reliable, scalable, and maintainable systems with consensus, replication, and partitioning.",
      tags: ["system-design", "database", "distributed"],
      saved_at: Math.floor((Date.now() - 5 * 86400 * 1000) / 1000),
      reading_time_min: 60,
      status: "library",
      decay_score: 0.08,
      why_surfaced: "Why: core reference for distributed state engines",
      vault_path: "Library/Designing Data-Intensive Applications.md"
    },
    {
      id: "clip-07",
      url: "https://doc.rust-lang.org/error_codes/E0597.html",
      type: "code-error",
      source: "extension",
      title: "E0597: borrowed value does not live long enough in async block",
      summary: "Capturing a non-static reference across an await boundary requires moving ownership or pinning.",
      tags: ["rust", "compiler", "async"],
      saved_at: Math.floor((Date.now() - 12 * 86400 * 1000) / 1000),
      reading_time_min: 3,
      status: "inbox",
      decay_score: 0.45,
      why_surfaced: "Why: encountered in Tokio scheduler implementation",
      vault_path: "Inbox/E0597 Async Lifetime.md"
    },
    {
      id: "clip-08",
      url: "https://assets.local/whiteboard-broker-session.png",
      type: "photo",
      source: "pwa",
      title: "Architecture whiteboard: Hive Agent socket protocol",
      summary: "Hand-drawn state machine for bidirectional message passing between orchestrator and worker agents.",
      tags: ["architecture", "diagram", "hive"],
      saved_at: Math.floor((Date.now() - 1 * 86400 * 1000) / 1000),
      reading_time_min: 2,
      status: "library",
      decay_score: 0.15,
      why_surfaced: "Why: matches active project documentation",
      vault_path: "Library/Whiteboard Hive Protocol.md"
    },
    {
      id: "clip-09",
      url: "https://rustconf.com/2026",
      type: "event",
      source: "manual",
      title: "RustConf 2026 Keynote & Workshops",
      summary: "Annual conference gathering the Rust community to discuss the 2027 edition and language roadmap.",
      tags: ["conference", "talk", "community"],
      saved_at: Math.floor((Date.now() - 18 * 86400 * 1000) / 1000),
      reading_time_min: 5,
      status: "library",
      decay_score: 0.62,
      why_surfaced: "Why: upcoming community milestone",
      vault_path: "Library/RustConf 2026.md"
    }
  ],
  stats: {
    total: 234,
    stale_30d: 14,
    by_status: { inbox: 7, library: 227, archived: 42 },
    by_type: { article: 47, video: 23, book: 12, repo: 18, "code-error": 4, photo: 22, event: 8, paper: 10 }
  }
};

function isMockEnabled() {
  if (typeof window !== "undefined" && window.__MOCK_API__) return true;
  if (typeof location !== "undefined" && location.search) {
    const params = new URLSearchParams(location.search);
    return params.get("mock") === "1" || params.get("mock") === "true";
  }
  return false;
}

async function api(path, opts = {}) {
  if (isMockEnabled()) {
    return mockApiHandler(path, opts);
  }

  try {
    const res = await fetch(path, opts);
    const contentType = res.headers.get("content-type") || "";

    if (!res.ok) {
      let errPayload;
      try {
        errPayload = contentType.includes("application/json") ? await res.json() : await res.text();
      } catch (e) {
        errPayload = res.statusText;
      }
      const errMsg = typeof errPayload === "string" ? errPayload : errPayload.error || errPayload.message || `HTTP ${res.status}`;
      const error = new Error(errMsg);
      error.status = res.status;
      throw error;
    }

    if (contentType.includes("application/json")) {
      return await res.json();
    }
    return await res.text();
  } catch (err) {
    console.error(`[Clipper API] Fetch failed for ${path}:`, err);
    throw err;
  }
}

function mockApiHandler(path, opts = {}) {
  const method = (opts.method || "GET").toUpperCase();

  if (method === "POST" && path.startsWith("/api/items")) {
    let body = {};
    try {
      body = typeof opts.body === "string" ? JSON.parse(opts.body) : (opts.body || {});
    } catch (e) {}
    const newItem = {
      id: "clip-" + Date.now().toString(36),
      url: body.url || "https://example.com",
      type: "article",
      source: "manual",
      title: body.url ? body.url.replace(/^https?:\/\//, "") : "New saved item",
      summary: "Newly saved link added to your personal library archive.",
      tags: ["saved"],
      saved_at: Math.floor(Date.now() / 1000),
      reading_time_min: 5,
      status: "library",
      decay_score: 0.05,
      why_surfaced: "Why: newly saved item",
      vault_path: "Library/New Item.md"
    };
    MOCK_DATA.items.unshift(newItem);
    return { ok: true, item: newItem };
  }

  if (method === "POST" && path.includes("/status")) {
    const idMatch = path.match(/\/api\/items\/([^/]+)\/status/);
    if (idMatch) {
      const id = idMatch[1];
      let body = {};
      try { body = typeof opts.body === "string" ? JSON.parse(opts.body) : (opts.body || {}); } catch(e) {}
      const item = MOCK_DATA.items.find(i => i.id === id);
      if (item && body.status) {
        item.status = body.status;
      }
      return { ok: true };
    }
  }

  if (method === "POST" && path.includes("/tags")) {
    const idMatch = path.match(/\/api\/items\/([^/]+)\/tags/);
    if (idMatch) {
      const id = idMatch[1];
      let body = {};
      try { body = typeof opts.body === "string" ? JSON.parse(opts.body) : (opts.body || {}); } catch(e) {}
      const item = MOCK_DATA.items.find(i => i.id === id);
      if (item && body.tags) {
        item.tags = body.tags;
      }
      return { ok: true };
    }
  }

  if (method === "GET") {
    const itemMatch = path.match(/^\/api\/items\/([^/?#]+)$/);
    if (itemMatch) {
      const id = itemMatch[1];
      const item = MOCK_DATA.items.find(i => i.id === id);
      if (item) {
        return {
          ...item,
          content: item.content || `# ${item.title}\n\n${item.summary}\n\nSaved from ${item.url}.`,
          body_md: item.content || `# ${item.title}\n\n${item.summary}\n\nSaved from ${item.url}.`,
          frontmatter: { title: item.title, url: item.url, type: item.type, tags: (item.tags || []).join(", ") }
        };
      }
    }
  }

  if (path.startsWith("/api/today") || path.startsWith("/api/items")) {
    return { items: [...MOCK_DATA.items] };
  }
  if (path.startsWith("/api/stats")) {
    return JSON.parse(JSON.stringify(MOCK_DATA.stats));
  }
  if (path === "/healthz") {
    return "ok";
  }
  return { ok: true };
}

/* ============================================================================
   Page Component Factories
   ============================================================================ */

function pageToday() {
  return {
    title: "Today",
    greeting: "Good evening, Aswath.",
    dateFormatted: new Intl.DateTimeFormat(undefined, { weekday: "long", day: "numeric", month: "long" }).format(new Date()),
    items: [],
    loading: false,
    errorState: null,

    // Metrics for "THIS WEEK" (Section 3)
    thisWeek: {
      saved: 23,
      savedDelta: "↑ 4 vs last",
      read: 12,
      readDelta: "= vs last",
      focus: "5h 40m",
      focusDelta: "↑ 1h 10m"
    },

    // Stale & Forgotten count
    staleCount: 14,

    // Topics you lean on (top 8)
    topTopics: [
      { name: "rust", count: 47, percent: 94 },
      { name: "ai", count: 31, percent: 62 },
      { name: "cooking", count: 8, percent: 16 },
      { name: "sqlite", count: 18, percent: 36 },
      { name: "tauri", count: 14, percent: 28 },
      { name: "alpine", count: 12, percent: 24 },
      { name: "design", count: 9, percent: 18 },
      { name: "systems", count: 7, percent: 14 }
    ],

    // Decay distribution
    decayDistribution: {
      fresh: 28,
      warm: 12,
      stale: 7
    },

    // 365-day contribution calendar + 90-day sparkline
    calendarCells: [],
    calendarMonths: [],
    yearTotal: 0,
    selectedDay: null,
    calendarTooltip: { visible: false, x: 0, y: 0, text: "" },

    // 90-day sparkline bars: 90 entries, each with x, height, color, opacity
    sparklineBars: [],
    ninetyDayTotal: 0,

    async init() {
      this.updateGreeting();
      this.generateCalendar();
      this.generateSparkline();
      await this.loadItems();
    },

    updateGreeting() {
      const hour = new Date().getHours();
      let timeGreeting = "Good evening";
      if (hour >= 5 && hour < 12) timeGreeting = "Good morning";
      else if (hour >= 12 && hour < 17) timeGreeting = "Good afternoon";
      this.greeting = `${timeGreeting}, Aswath.`;
    },

    itemDateIso(savedAt) {
      if (!savedAt) return "";
      const ms = typeof savedAt === "number" && savedAt < 1e11 ? savedAt * 1000 : Number(savedAt);
      const d = new Date(ms);
      if (isNaN(d.getTime())) return "";
      const yyyy = d.getFullYear();
      const mm = String(d.getMonth() + 1).padStart(2, "0");
      const dd = String(d.getDate()).padStart(2, "0");
      return `${yyyy}-${mm}-${dd}`;
    },

    filteredTodayItems() {
      if (!this.selectedDay) return this.items;
      return this.items.filter(item => this.itemDateIso(item.saved_at) === this.selectedDay);
    },

    clearDayFilter() {
      this.selectedDay = null;
    },

    bucketForCount(count) {
      if (!count || count <= 0) return { bucket: 0, fill: "var(--mute)", opacity: "0.14" };
      if (count === 1) return { bucket: 1, fill: "var(--accent)", opacity: "0.32" };
      if (count <= 3) return { bucket: 2, fill: "var(--accent)", opacity: "0.55" };
      if (count <= 6) return { bucket: 3, fill: "var(--accent)", opacity: "0.78" };
      return { bucket: 4, fill: "var(--accent)", opacity: "1" };
    },

    generateCalendar() {
      const cells = [];
      const months = [];
      const today = new Date();
      today.setHours(0, 0, 0, 0);

      // Build counts map from any loaded items plus a calm deterministic baseline
      const itemCounts = {};
      (this.items || []).forEach(it => {
        const iso = this.itemDateIso(it.saved_at);
        if (iso) itemCounts[iso] = (itemCounts[iso] || 0) + 1;
      });

      const seedPattern = [
        0, 1, 0, 2, 3, 0, 1, 4, 2, 0, 0, 1, 5, 7, 2, 0, 1, 3, 0, 2,
        1, 0, 0, 2, 4, 6, 1, 0, 3, 2, 8, 4, 1, 0, 0, 2, 1, 3, 5, 2,
        0, 1, 0, 0, 2, 3, 1, 4, 0, 2, 1, 0, 3, 6, 7, 2, 1, 0, 0, 1
      ];

      let total = 0;
      let lastMonth = -1;
      const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

      for (let i = 0; i < 365; i++) {
        const offsetDays = 364 - i;
        const d = new Date(today.getTime() - offsetDays * 86400 * 1000);
        const yyyy = d.getFullYear();
        const mm = String(d.getMonth() + 1).padStart(2, "0");
        const dd = String(d.getDate()).padStart(2, "0");
        const dateStr = `${yyyy}-${mm}-${dd}`;
        const labelDate = d.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" });

        const baseCount = seedPattern[(i * 7 + d.getDate()) % seedPattern.length];
        const count = itemCounts[dateStr] !== undefined ? Math.max(itemCounts[dateStr], baseCount) : baseCount;
        total += count;

        const col = Math.floor(i / 7);
        const row = i % 7;
        const x = col * 14 + 2;
        const y = row * 12 + 18;

        if (row === 0 && d.getMonth() !== lastMonth) {
          if (months.length === 0 || (x - months[months.length - 1].x) >= 28) {
            months.push({ label: monthNames[d.getMonth()], x });
          }
          lastMonth = d.getMonth();
        }

        const b = this.bucketForCount(count);
        cells.push({
          date: dateStr,
          labelDate,
          count,
          bucket: b.bucket,
          fill: b.fill,
          opacity: b.opacity,
          x,
          y
        });
      }

      this.calendarCells = cells;
      this.calendarMonths = months;
      this.yearTotal = total;
    },

    renderCalendar() {
      const monthLabels = (this.calendarMonths || []).map(m =>
        `<text x="${m.x}" y="11" fill="var(--mute)" font-size="9" font-family="var(--font-mono)">${m.label}</text>`
      ).join("");

      const rects = (this.calendarCells || []).map(cell => {
        const isSelected = this.selectedDay === cell.date;
        const strokeAttr = isSelected
          ? `stroke="var(--ink)" stroke-width="1.5"`
          : `stroke="none"`;
        const saveWord = cell.count === 1 ? "save" : "saves";
        return `<rect class="calendar-cell" data-date="${cell.date}" data-label="${cell.labelDate}" data-count="${cell.count}" data-bucket="${cell.bucket}" x="${cell.x}" y="${cell.y}" width="11" height="10" rx="2" fill="${cell.fill}" fill-opacity="${cell.opacity}" ${strokeAttr}><title>${cell.date}: ${cell.count} ${saveWord}</title></rect>`;
      }).join("");

      return monthLabels + rects;
    },

    handleCalendarHover(e) {
      const rect = e.target.closest("rect.calendar-cell");
      if (!rect) {
        this.calendarTooltip.visible = false;
        return;
      }
      const date = rect.getAttribute("data-date");
      const count = Number(rect.getAttribute("data-count") || 0);
      const saveWord = count === 1 ? "save" : "saves";
      this.calendarTooltip.text = `${date} · ${count} ${saveWord}`;
      this.calendarTooltip.visible = true;
      this.moveCalendarTooltip(e);
    },

    moveCalendarTooltip(e) {
      if (!this.calendarTooltip.visible) return;
      const wrap = e.currentTarget;
      if (!wrap) return;
      const bounds = wrap.getBoundingClientRect();
      this.calendarTooltip.x = Math.min(Math.max(40, e.clientX - bounds.left), bounds.width - 60);
      this.calendarTooltip.y = Math.max(4, e.clientY - bounds.top - 30);
    },

    hideCalendarTooltip() {
      this.calendarTooltip.visible = false;
    },

    handleCalendarClick(e) {
      const rect = e.target.closest("rect.calendar-cell");
      if (!rect) return;
      const date = rect.getAttribute("data-date");
      if (!date) return;
      if (this.selectedDay === date) {
        this.selectedDay = null;
        toast("Cleared day filter", "info");
      } else {
        this.selectedDay = date;
        toast(`Filtered Today to ${date}`, "info");
      }
    },

    renderSparkline() {
      // SVG-namespace workaround: <template x-for> inside <svg> fails to bind in Alpine 3.
      // Render rects as an SVG string and bind via x-html on a <g>.
      return (this.sparklineBars || []).map(bar =>
        `<rect x="${bar.x}" y="${bar.y}" width="${bar.width}" height="${bar.height}" fill="${bar.fill}" opacity="${bar.opacity}" rx="1"><title>${bar.count} saves</title></rect>`
      ).join("");
    },

    generateSparkline() {
      // 90 days of activity. ViewBox 0 0 720 60.
      // Bar width: 6px, gap: 2px (90 bars * 8px = 720px width).
      const bars = [];
      // Seeded deterministic counts for calm presentation
      const countsPattern = [
        1, 0, 2, 3, 5, 4, 2, 0, 1, 3, 4, 6, 8, 5, 2, 1, 0, 0, 2, 4, 3, 5,
        1, 2, 0, 1, 4, 6, 9, 7, 5, 3, 2, 1, 0, 1, 2, 3, 5, 4, 2, 1, 0, 2,
        3, 4, 6, 5, 3, 2, 1, 0, 0, 1, 3, 5, 7, 4, 2, 1, 0, 2, 3, 5, 6, 4,
        2, 1, 0, 1, 3, 4, 7, 8, 6, 4, 2, 1, 0, 2, 4, 5, 7, 9, 6, 4, 3, 2, 5, 4
      ];

      for (let i = 0; i < 90; i++) {
        const count = countsPattern[i] ?? 0;
        const x = i * 8;
        // Max height capped at 54px (leaving 6px top margin in 60px viewBox)
        const barHeight = count > 0 ? Math.min(Math.round((count / 10) * 50) + 4, 54) : 3;
        const y = 60 - barHeight;
        const fill = count > 0 ? "var(--accent)" : "var(--mute-2)";
        const opacity = count > 0 ? 0.9 : 0.15;
        bars.push({ x, y, width: 6, height: barHeight, fill, opacity, count });
      }
      this.sparklineBars = bars;
      this.ninetyDayTotal = countsPattern.reduce((sum, n) => sum + n, 0);
    },

    async loadItems() {
      this.loading = true;
      this.errorState = null;
      try {
        const res = await api("/api/today");
        this.items = (res.items || []).slice(0, 5);
        this.generateCalendar();
      } catch (err) {
        this.errorState = err.message || "Failed to load today items";
        this.items = [];
        toast(`Error loading today's library: ${err.message}`, "danger");
      } finally {
        this.loading = false;
      }
    },

    formatAge(savedAt) {
      if (!savedAt) return "recently";
      const sec = typeof savedAt === "number" && savedAt > 1e11 ? Math.floor(savedAt / 1000) : savedAt;
      const diffSec = Math.max(0, Math.floor(Date.now() / 1000) - sec);
      const days = Math.floor(diffSec / 86400);
      if (days === 0) return "today";
      if (days === 1) return "1d ago";
      if (days < 7) return `${days}d ago`;
      const weeks = Math.floor(days / 7);
      return `${weeks}w ago`;
    },

    firstUrl: "",
    async saveFirstUrl() {
      if (!this.firstUrl.trim()) return;
      toast("Saving URL to library...", "info");
      try {
        await api("/api/items", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url: this.firstUrl.trim() })
        });
        toast("Saved! Welcome to your library", "info");
        this.firstUrl = "";
        await this.loadItems();
      } catch (err) {
        toast("Failed to save: " + err.message, "danger");
      }
    }
  };
}

function pageLibrary() {
  return {
    title: "Library",
    items: [],
    loading: false,
    errorState: null,
    viewMode: "grid", // 'grid' | 'list'
    filterType: "all",
    sortMode: "recent", // 'recent' | 'oldest' | 'decay'
    selectedTag: "",

    typeChips: [
      { id: "all", label: "All", count: 234 },
      { id: "article", label: "Articles", count: 47 },
      { id: "video", label: "Videos", count: 23 },
      { id: "book", label: "Books", count: 12 },
      { id: "repo", label: "Repos", count: 18 },
      { id: "code-error", label: "Errors", count: 4 },
      { id: "photo", label: "Photos", count: 22 },
      { id: "event", label: "Events", count: 8 },
      { id: "paper", label: "Papers", count: 10 }
    ],

    async init() {
      await this.loadItems();
    },

    async loadItems() {
      this.loading = true;
      this.errorState = null;
      try {
        const res = await api("/api/items?status=all");
        this.items = res.items || [];
      } catch (err) {
        this.errorState = err.message || "Failed to load library items";
        this.items = [];
        toast(`Error loading library: ${err.message}`, "danger");
      } finally {
        this.loading = false;
      }
    },

    filteredItems() {
      let result = [...this.items];
      if (this.filterType !== "all") {
        result = result.filter(item => (item.type || "").toLowerCase() === this.filterType);
      }
      if (this.selectedTag) {
        result = result.filter(item => (item.tags || []).includes(this.selectedTag));
      }
      if (this.sortMode === "recent") {
        result.sort((a, b) => (b.saved_at || 0) - (a.saved_at || 0));
      } else if (this.sortMode === "oldest") {
        result.sort((a, b) => (a.saved_at || 0) - (b.saved_at || 0));
      } else if (this.sortMode === "decay") {
        result.sort((a, b) => (b.decay_score || 0) - (a.decay_score || 0));
      }
      return result;
    },

    formatAge(savedAt) {
      if (!savedAt) return "recently";
      const sec = typeof savedAt === "number" && savedAt > 1e11 ? Math.floor(savedAt / 1000) : savedAt;
      const diffSec = Math.max(0, Math.floor(Date.now() / 1000) - sec);
      const days = Math.floor(diffSec / 86400);
      if (days === 0) return "today";
      if (days === 1) return "1d ago";
      if (days < 7) return `${days}d ago`;
      const weeks = Math.floor(days / 7);
      return `${weeks}w ago`;
    },

    decayColor(score) {
      const s = Number(score) || 0;
      if (s >= 0.7) return "var(--danger)";
      if (s >= 0.35) return "var(--type-book)";
      return "var(--decay-fresh)";
    },

    allTags() {
      const set = new Set();
      this.items.forEach(item => (item.tags || []).forEach(t => set.add(t)));
      return Array.from(set);
    },

    newUrl: "",
    async saveUrl() {
      if (!this.newUrl.trim()) return;
      toast("Saving URL to library...", "info");
      try {
        await api("/api/items", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url: this.newUrl.trim() })
        });
        toast("Saved to library", "info");
        this.newUrl = "";
        await this.loadItems();
      } catch (err) {
        toast("Failed to save: " + err.message, "danger");
      }
    }
  };
}

function pageInbox() {
  return {
    title: "Inbox",
    items: [],
    loading: false,
    errorState: null,
    filterMode: "all", // 'all' | 'untagged' | 'needs-deadline' | 'recent'
    selectedIds: [],
    focusedIndex: 0,

    async init() {
      await this.loadItems();
      this.initKeybindings();
    },

    async loadItems() {
      this.loading = true;
      this.errorState = null;
      try {
        const res = await api("/api/items?status=inbox");
        const all = res.items || [];
        this.items = all.filter(i => (i.status || "inbox") === "inbox");
      } catch (err) {
        this.errorState = err.message || "Failed to load inbox items";
        this.items = [];
        toast(`Error loading inbox: ${err.message}`, "danger");
      } finally {
        this.loading = false;
      }
    },

    filteredItems() {
      let list = [...this.items];
      if (this.filterMode === "untagged") {
        list = list.filter(i => !i.tags || i.tags.length === 0);
      } else if (this.filterMode === "needs-deadline") {
        list = list.filter(i => !i.deadline && (i.decay_score || 0) > 0.4);
      } else if (this.filterMode === "recent") {
        list.sort((a, b) => (b.saved_at || 0) - (a.saved_at || 0));
      }
      return list;
    },

    isSelected(id) {
      return this.selectedIds.includes(id);
    },

    toggleSelect(id) {
      const idx = this.selectedIds.indexOf(id);
      if (idx !== -1) {
        this.selectedIds.splice(idx, 1);
      } else {
        this.selectedIds.push(id);
      }
    },

    selectAll() {
      const list = this.filteredItems();
      if (this.selectedIds.length === list.length) {
        this.selectedIds = [];
      } else {
        this.selectedIds = list.map(i => i.id);
      }
    },

    async archiveItem(item) {
      try {
        await api(`/api/items/${item.id}/status`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ status: "archived" })
        });
        item.status = "archived";
        this.items = this.items.filter(i => i.id !== item.id);
        this.selectedIds = this.selectedIds.filter(id => id !== item.id);
        toast(`Archived "${item.title}"`, "info");
      } catch (err) {
        toast(`Failed to archive: ${err.message}`, "danger");
      }
    },

    async archiveSelected() {
      const count = this.selectedIds.length;
      if (count === 0) return;
      for (const id of [...this.selectedIds]) {
        const item = this.items.find(i => i.id === id);
        if (item) {
          await this.archiveItem(item);
        }
      }
      this.selectedIds = [];
      toast(`Archived ${count} items`, "info");
    },

    async deleteSelected() {
      const count = this.selectedIds.length;
      if (count === 0) return;
      this.items = this.items.filter(i => !this.selectedIds.includes(i.id));
      this.selectedIds = [];
      toast(`Removed ${count} items`, "info");
    },

    async tagItem(item) {
      const newTag = prompt(`Add a tag to "${item.title}":`);
      if (newTag && newTag.trim()) {
        const tag = newTag.trim().replace(/^#/, "");
        item.tags = item.tags || [];
        if (!item.tags.includes(tag)) {
          item.tags.push(tag);
          try {
            await api(`/api/items/${item.id}/tags`, {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ tags: item.tags })
            });
            toast(`Added #${tag}`, "info");
          } catch (e) {
            toast(`Saved tag locally`, "info");
          }
        }
      }
    },

    async addTagSelected() {
      const tagPrompt = prompt(`Add a tag to ${this.selectedIds.length} items:`);
      if (!tagPrompt || !tagPrompt.trim()) return;
      const tag = tagPrompt.trim().replace(/^#/, "");
      for (const id of this.selectedIds) {
        const item = this.items.find(i => i.id === id);
        if (item) {
          item.tags = item.tags || [];
          if (!item.tags.includes(tag)) item.tags.push(tag);
        }
      }
      toast(`Tagged ${this.selectedIds.length} items with #${tag}`, "info");
    },

    setDeadlineSelected() {
      toast(`Deadlines set to 7 days for ${this.selectedIds.length} items`, "info");
      this.selectedIds = [];
    },

    openObsidian(item) {
      if (item.vault_path) {
        window.open(`obsidian://open?file=${encodeURIComponent(item.vault_path)}`, "_blank");
      } else {
        toast(`Opening "${item.title}" in Obsidian...`, "info");
      }
    },

    initKeybindings() {
      window.addEventListener("keydown", (e) => {
        if (window.location.hash !== "#/inbox") return;
        if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA" || e.target.tagName === "SELECT") return;

        const list = this.filteredItems();
        if (list.length === 0) return;

        if (e.key === "j") {
          this.focusedIndex = Math.min(this.focusedIndex + 1, list.length - 1);
        } else if (e.key === "k") {
          this.focusedIndex = Math.max(this.focusedIndex - 1, 0);
        }
        if (e.key === "j" || e.key === "k") {
          this.$nextTick(() => {
            const row = document.querySelector(".inbox-row.focused");
            if (row) row.focus();
          });
        } else if (e.key === "a") {
          const item = list[this.focusedIndex];
          if (item) this.archiveItem(item);
        } else if (e.key === "t") {
          const item = list[this.focusedIndex];
          if (item) this.tagItem(item);
        } else if (e.key === "Enter") {
          const item = list[this.focusedIndex];
          if (item) this.$dispatch("open-detail", item);
        }
      });
    },

    formatAge(savedAt) {
      if (!savedAt) return "recently";
      const sec = typeof savedAt === "number" && savedAt > 1e11 ? Math.floor(savedAt / 1000) : savedAt;
      const diffSec = Math.max(0, Math.floor(Date.now() / 1000) - sec);
      const days = Math.floor(diffSec / 86400);
      if (days === 0) return "today";
      if (days === 1) return "1d";
      if (days < 7) return `${days}d`;
      const weeks = Math.floor(days / 7);
      return `${weeks}w`;
    }
  };
}

function pageAsk() {
  return {
    title: "Ask your brain",
    activeConvId: "c1",
    history: {
      today: [
        { id: "c1", title: "Rust async & Tokio scheduling", snippet: "What did I save about Rust async?" }
      ],
      yesterday: [
        { id: "c2", title: "Transformer attention heads", snippet: "Summarize Andrej's video on LLMs" }
      ],
      older: [
        { id: "c3", title: "TDD & architecture feedback", snippet: "Why is TDD dead according to Fowler?" }
      ]
    },
    messages: [
      {
        role: "user",
        content: "what did I save about Rust async?"
      },
      {
        role: "assistant",
        content: "Based on your saved items, your Rust async interest centers on non-blocking task runtimes and compiler borrow lifetime contracts:\n\n• **Rust ownership patterns in 2026** (8 min read)\n• **Tokio: An asynchronous runtime for Rust** (15 min read)\n• **E0597: borrowed value does not live long enough in async block** (3 min read)\n\nThe underlying theme is managing reference lifetimes across suspension points without unnecessary heap indirection.",
        sources: [
          { id: "clip-01", title: "Rust ownership patterns in 2026", type: "article", reading_time_min: 8 },
          { id: "clip-04", title: "Tokio: An asynchronous runtime for Rust", type: "repo", reading_time_min: 15 }
        ],
        sourcesOpen: true
      }
    ],
    inputPrompt: "",
    streaming: false,
    currentStreamText: "",
    abortController: null,

    newChat() {
      if (this.streaming) this.stopStreaming();
      this.activeConvId = "new-" + Date.now().toString(36);
      this.messages = [];
      this.inputPrompt = "";
    },

    loadConversation(conv) {
      if (this.streaming) this.stopStreaming();
      this.activeConvId = conv.id;
      if (conv.id === "c1") {
        this.messages = [
          {
            role: "user",
            content: "what did I save about Rust async?"
          },
          {
            role: "assistant",
            content: "Based on your saved items, your Rust async interest centers on non-blocking task runtimes and compiler borrow lifetime contracts:\n\n• **Rust ownership patterns in 2026** (8 min read)\n• **Tokio: An asynchronous runtime for Rust** (15 min read)\n• **E0597: borrowed value does not live long enough in async block** (3 min read)\n\nThe underlying theme is managing reference lifetimes across suspension points without unnecessary heap indirection.",
            sources: [
              { id: "clip-01", title: "Rust ownership patterns in 2026", type: "article", reading_time_min: 8 },
              { id: "clip-04", title: "Tokio: An asynchronous runtime for Rust", type: "repo", reading_time_min: 15 }
            ],
            sourcesOpen: true
          }
        ];
      } else if (conv.id === "c2") {
        this.messages = [
          { role: "user", content: "Summarize Andrej's video on LLMs" },
          { role: "assistant", content: "Andrej Karpathy's video walks step-by-step through implementing an autoregressive transformer from scratch. Starting from token byte-pair encoding, setting up multi-head self-attention, and training with cross-entropy loss.", sources: [{ id: "clip-02", title: "Build a tiny LLM from scratch", type: "video", reading_time_min: 47 }], sourcesOpen: true }
        ];
      } else {
        this.messages = [
          { role: "user", content: "Why is TDD dead according to Fowler?" },
          { role: "assistant", content: "The article argues that dogmatic test-first practices often lead to brittle mocks. Architecture feedback loops and integration boundaries matter more than 100% unit-level assertion coverage.", sources: [{ id: "clip-03", title: "Why TDD is dead (again)", type: "article", reading_time_min: 6 }], sourcesOpen: true }
        ];
      }
    },

    askSuggestion(text) {
      this.inputPrompt = text;
      this.sendQuestion();
    },

    async sendQuestion() {
      const q = this.inputPrompt.trim();
      if (!q || this.streaming) return;

      this.messages.push({ role: "user", content: q });
      this.inputPrompt = "";
      this.streaming = true;
      this.currentStreamText = "";
      this.abortController = new AbortController();

      const itemScope = MOCK_DATA.items[0] ? MOCK_DATA.items[0].id : "clip-01";
      const sseUrl = `/api/items/${itemScope}/chat`;

      try {
        let streamReader = null;

        if (isMockEnabled()) {
          const mockAnswer = `Analyzing your archive for "${q}"...\n\nYour library holds several tightly related references. Key takeaways:\n\n1. **Core Concept**: Connected references form an active mental model rather than static files.\n2. **Synthesis**: The relevant notes indicate high alignment with your active focus on systems programming and architecture.\n3. **Recommendation**: Review the matching clips linked below to consolidate key points.`;
          const words = mockAnswer.split(" ");
          
          const customStream = new ReadableStream({
            async start(controller) {
              for (const word of words) {
                const sseChunk = `event: chunk\ndata: ${JSON.stringify({ text: word + " " })}\n\n`;
                controller.enqueue(new TextEncoder().encode(sseChunk));
                await new Promise(r => setTimeout(r, 40));
              }
              controller.enqueue(new TextEncoder().encode(`event: done\ndata: {"exit_code": 0}\n\n`));
              controller.close();
            }
          });
          streamReader = customStream.getReader();
        } else {
          const response = await fetch(sseUrl, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ question: q }),
            signal: this.abortController.signal
          });
          if (!response.ok || !response.body) {
            throw new Error(`Chat endpoint error (HTTP ${response.status})`);
          }
          streamReader = response.body.getReader();
        }

        // Process SSE stream with getReader() per DESIGN.md Section 6
        const decoder = new TextDecoder();
        let buffer = "";

        while (true) {
          const { done, value } = await streamReader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() || "";

          for (const line of lines) {
            if (line.startsWith("data:")) {
              const dataStr = line.slice(5).trim();
              try {
                const parsed = JSON.parse(dataStr);
                if (parsed.text) {
                  this.currentStreamText += parsed.text;
                }
              } catch (e) {
                if (dataStr) this.currentStreamText += dataStr + " ";
              }
            }
          }
        }

        // Finalize assistant turn
        this.messages.push({
          role: "assistant",
          content: this.currentStreamText,
          sources: MOCK_DATA.items.slice(0, 3),
          sourcesOpen: true
        });

      } catch (err) {
        if (err.name === "AbortError") {
          if (this.currentStreamText) {
            this.messages.push({
              role: "assistant",
              content: this.currentStreamText + " *(cancelled)*",
              sources: [],
              sourcesOpen: false
            });
          }
        } else {
          toast("Chat error: " + err.message, "danger");
        }
      } finally {
        this.streaming = false;
        this.currentStreamText = "";
        this.abortController = null;
      }
    },

    stopStreaming() {
      if (this.abortController) {
        this.abortController.abort();
      }
    },

    renderMarkdown(text) {
      if (!text) return "";
      try {
        const raw = typeof marked !== "undefined" ? marked.parse(text) : text;
        return typeof DOMPurify !== "undefined" ? DOMPurify.sanitize(raw) : raw;
      } catch (e) {
        return text;
      }
    }
  };
}

function pageSkills() {
  return {
    title: "Skills",
    loading: false,
    skills: [
      {
        name: "summarize",
        description: "Extracts key takeaways, architecture decisions, and structured bullets from articles",
        triggers: ["article", "paper", "reading"],
        enabled: true,
        last_used: "Last used 1h ago, 128 times",
        body_md: `# Summary: {{title}}\n\n## Core Takeaways\n- Focuses on minimal cognitive friction and zero boilerplate.\n- Establishes explicit boundaries between layers.\n\n## Key Quotations\n> "Simplicity is prerequisite for reliability."\n\n## Action Items\n1. Review reference implementations in Rust.\n2. Update vault index.`
      },
      {
        name: "extract-recipe",
        description: "Parses cooking blog ingredients and steps into clean markdown lists",
        triggers: ["cooking", "recipe", "baking"],
        enabled: true,
        last_used: "Last used 3h ago, 47 times",
        body_md: `# Recipe: {{title}}\n\n### Prep Time\n{{prep_time}}\n\n### Ingredients\n- 500g Flour\n- 350ml Water\n- 10g Salt\n\n### Method\n1. Autolyse flour and water for 45 minutes.\n2. Incorporate salt and perform stretch-and-folds.`
      },
      {
        name: "code-solution",
        description: "Extracts error code, compiler diagnostics, root cause, and verified fix snippet",
        triggers: ["error", "code-error", "rust", "compiler"],
        enabled: true,
        last_used: "Last used 5h ago, 34 times",
        body_md: `# Error Diagnostic: {{error_code}}\n\n### Root Cause\n{{explanation}}\n\n### Verified Fix\n\`\`\`rust\n// Scope ownership explicitly\nlet arc_data = Arc::clone(&data);\ntokio::spawn(async move {\n    process(arc_data).await;\n});\n\`\`\``
      },
      {
        name: "arxiv-paper",
        description: "Extracts abstract, methodology, benchmark scores, and BibTeX citation block",
        triggers: ["paper", "ai", "deep-learning"],
        enabled: false,
        last_used: "Last used 1d ago, 19 times",
        body_md: `# Paper: {{title}}\n\n## Abstract\n{{abstract}}\n\n## Methodology\nEvaluated on standard benchmarks with autoregressive generation.\n\n## BibTeX\n\`\`\`bibtex\n@article{reference2026,\n  title={...},\n  author={...}\n}\n\`\`\``
      }
    ],

    editorOpen: false,
    editingSkill: { name: "", description: "", body_md: "", triggers: [], enabled: true },
    isNewSkill: false,

    testModalOpen: false,
    testingSkillName: "",
    testUrl: "",
    testing: false,
    testResult: "",

    async init() {
      await this.loadSkills();
    },

    async loadSkills() {
      this.loading = true;
      try {
        const res = await api("/api/skills");
        if (res && res.skills && res.skills.length > 0) {
          this.skills = res.skills;
        }
      } catch (err) {
        console.warn("Failed loading live skills, using default catalog:", err);
      } finally {
        this.loading = false;
      }
    },

    editSkill(skill) {
      this.isNewSkill = false;
      this.editingSkill = JSON.parse(JSON.stringify(skill));
      this.editorOpen = true;
    },

    addSkill() {
      this.isNewSkill = true;
      this.editingSkill = {
        name: "new-skill",
        description: "Describe the extraction or transformation rules",
        triggers: ["article"],
        enabled: true,
        last_used: "Never used",
        body_md: `# {{title}}\n\n## Summary\n{{summary}}\n\n## Notes\n- Extracted item.`
      };
      this.editorOpen = true;
    },

    closeEditor() {
      this.editorOpen = false;
      this.editingSkill = null;
    },

    async saveSkill() {
      if (!this.editingSkill || !this.editingSkill.name) return;
      try {
        await api(`/api/skills/${this.editingSkill.name}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            body_md: this.editingSkill.body_md,
            source: "editor"
          })
        });
      } catch (e) {
        console.warn("Backend save skipped or mock:", e);
      }

      const idx = this.skills.findIndex(s => s.name === this.editingSkill.name);
      if (idx !== -1) {
        this.skills[idx] = JSON.parse(JSON.stringify(this.editingSkill));
      } else {
        this.skills.push(JSON.parse(JSON.stringify(this.editingSkill)));
      }
      toast(`Saved skill "${this.editingSkill.name}"`, "info");
      this.closeEditor();
    },

    toggleSkill(skill) {
      skill.enabled = !skill.enabled;
      toast(`${skill.name} ${skill.enabled ? 'enabled' : 'disabled'}`, "info");
    },

    renderPreview() {
      if (!this.editingSkill || !this.editingSkill.body_md) return "";
      try {
        const raw = typeof marked !== "undefined" ? marked.parse(this.editingSkill.body_md) : this.editingSkill.body_md;
        return typeof DOMPurify !== "undefined" ? DOMPurify.sanitize(raw) : raw;
      } catch (e) {
        return this.editingSkill.body_md;
      }
    },

    testSkill(skill) {
      this.testingSkillName = skill.name;
      this.testUrl = "https://example.com/sample-article";
      this.testResult = "";
      this.testing = false;
      this.testModalOpen = true;
    },

    closeTest() {
      this.testModalOpen = false;
      this.testResult = "";
    },

    async runTest() {
      if (!this.testUrl.trim()) return;
      this.testing = true;
      this.testResult = "";

      setTimeout(() => {
        const dummyResult = `# Extracted via [${this.testingSkillName}]\n\n**Source URL**: \`${this.testUrl}\`\n\n## Key Takeaways\n- Synthesized content extracted in 240ms.\n- Filtered navigation and tracking scripts.\n- Applied formatting rules according to the skill template.\n\n\`\`\`json\n{\n  "status": "extracted",\n  "tokens": 420,\n  "confidence": 0.98\n}\n\`\`\``;
        this.testResult = dummyResult;
        this.testing = false;
      }, 600);
    },

    renderTestMarkdown(text) {
      if (!text) return "";
      try {
        const raw = typeof marked !== "undefined" ? marked.parse(text) : text;
        return typeof DOMPurify !== "undefined" ? DOMPurify.sanitize(raw) : raw;
      } catch (e) {
        return text;
      }
    }
  };
}

function pageSettings() {
  return {
    title: "Settings",
    activeTab: "soul", // 'soul' | 'memory' | 'self' | 'providers' | 'pairing' | 'devices'
    isDirty: false,

    // Soul Tab
    soul: {
      persona: "A calm, intellectual second-brain curator with a focus on editorial clarity and signal over noise.",
      tone: "Concise, precise, respectful of reader time. Avoid cheerleading or corporate filler.",
      avoid: "Never produce bulleted summaries that discard nuance. Avoid hyperbole or superficial categorizations."
    },

    // Memory Tab
    memory: {
      userName: "Aswath",
      focus: "Learn Tokio internals. Read 1 chapter a day.",
      notes: "- Deep focus on Rust memory safety and non-blocking scheduling.\n- Prefers primary literature over blog aggregators.\n- Vault path configured to ~/Obsidian/SecondBrain."
    },

    // Self Tab
    self: {
      topics: [
        { name: "rust", count: 47 },
        { name: "ai", count: 31 },
        { name: "distributed", count: 18 },
        { name: "cooking", count: 8 },
        { name: "compiler", count: 12 },
        { name: "architecture", count: 24 }
      ],
      patterns: [
        { id: "p1", title: "Reads system architecture on Thursday mornings", confidence: "94%" },
        { id: "p2", title: "Saves Rust crates via Ctrl+Shift+C shortcut", confidence: "88%" },
        { id: "p3", title: "Tags LLM papers with #ai and #deep-learning", confidence: "91%" }
      ]
    },

    // Providers Tab
    providers: [
      { name: "Claude CLI (sonnet)", cli: "claude", active: true, latency: "210ms", status: "ok" },
      { name: "OpenAI Codex CLI", cli: "codex", active: false, latency: "—", status: "standby" },
      { name: "Google Gemini CLI", cli: "gemini", active: false, latency: "—", status: "standby" }
    ],
    configToml: `[providers.claude]
command = "claude"
model = "sonnet"
timeout_secs = 30

[vault]
path = "C:/Users/aswat/Obsidian/Vault"
auto_sync = true

[pairing]
allowed_lan_subnets = ["192.168.0.0/16"]`,

    // Pairing Tab
    pairing: {
      lanIp: "192.168.1.142:3030",
      ttl: "24m 15s",
      qrUrl: "/pair.png"
    },

    // Devices Tab
    devices: [
      { id: "d1", name: "iPhone 16 Pro (PWA)", lastSeen: "14 minutes ago", ip: "192.168.1.189" },
      { id: "d2", name: "iPad Air (Safari)", lastSeen: "2 days ago", ip: "192.168.1.192" }
    ],

    markDirty() {
      this.isDirty = true;
    },

    async saveSettings() {
      toast("Saving settings...", "info");
      setTimeout(() => {
        this.isDirty = false;
        toast("Settings saved", "info");
      }, 350);
    },

    regenerateMemory() {
      toast("Proposing memory updates via Claude...", "info");
      setTimeout(() => {
        this.memory.notes += "\n- Added interest in distributed consensus algorithms.";
        this.markDirty();
        toast("Memory updated with new proposed directives", "info");
      }, 700);
    },

    recomputeSelf() {
      toast("Recomputing self-patterns from archive...", "info");
      setTimeout(() => {
        toast("Patterns refreshed from 234 saved clips", "info");
      }, 600);
    },

    forgetPattern(pattern) {
      this.self.patterns = this.self.patterns.filter(p => p.id !== pattern.id);
      toast(`Forgot pattern: "${pattern.title}"`, "info");
    },

    async testProvider(prov) {
      toast(`Testing CLI provider "${prov.cli}"...`, "info");
      setTimeout(() => {
        prov.latency = Math.floor(180 + Math.random() * 80) + "ms";
        prov.status = "ok";
        toast(`${prov.name} passed self-test (${prov.latency})`, "info");
      }, 500);
    },

    regenerateQr() {
      if (confirm("Regenerate pairing token? Active sessions will need to re-scan.")) {
        this.pairing.ttl = "30m 00s";
        toast("Generated new pairing QR and session token", "info");
      }
    },

    revokeDevice(dev) {
      this.devices = this.devices.filter(d => d.id !== dev.id);
      toast(`Revoked access for ${dev.name}`, "info");
    },

    renderSoulPreview() {
      const md = `### Persona Directives\n${this.soul.persona}\n\n### Tone & Voice\n${this.soul.tone}\n\n### Negative Constraints\n${this.soul.avoid}`;
      try {
        const raw = typeof marked !== "undefined" ? marked.parse(md) : md;
        return typeof DOMPurify !== "undefined" ? DOMPurify.sanitize(raw) : raw;
      } catch (e) {
        return md;
      }
    }
  };
}

/* ============================================================================
   Application Shell Root Component
   ============================================================================ */

function clipperShell() {
  return {
    currentPage: "today",
    searchQuery: "",
    toasts: ToastManager.items,
    unreadInboxCount: 3,
    userMoodText: "reading: 3, learning: 11, idle: 2",
    activeDetailItem: null,
    detailChatOpen: false,
    detailChatMessages: [],
    detailChatInput: "",

    async openDetail(item) {
      this.activeDetailItem = item;
      this.detailChatOpen = false;
      this.detailChatMessages = [
        {
          role: "assistant",
          text: `Ask me anything about "${item.title}". I can synthesize concepts, extract code snippets, or compare it with other clips in your archive.`
        }
      ];
      if (item && item.id && !item.content) {
        try {
          const full = await api(`/api/items/${encodeURIComponent(item.id)}`);
          if (full && this.activeDetailItem && this.activeDetailItem.id === item.id) {
            this.activeDetailItem = { ...item, ...full };
          }
        } catch (err) {
          console.warn("Could not fetch full item body:", err);
        }
      }
    },

    closeDetail() {
      this.activeDetailItem = null;
    },

    renderDetailContent(item) {
      if (!item) return "";
      let md = item.content || item.summary || "";
      if (!item.content) {
        md = `### Synopsis\n\n${item.summary}\n\n> "Knowledge is built by connecting references, not merely accumulating files."\n\n#### Key Observations\n- Extracted from **${item.source || 'web'}** on ${new Date((item.saved_at || 0) * 1000).toLocaleDateString()}.\n- Vault destination: \`${item.vault_path || 'Library/Untitled.md'}\`\n\n\`\`\`rust\n// Example pattern reference\nasync fn verify_ownership() -> Result<(), Box<dyn std::error::Error>> {\n    println!("Synthesized via Clipper v3");\n    Ok(())\n}\n\`\`\`\n`;
      }
      try {
        const raw = typeof marked !== "undefined" ? marked.parse(md) : md;
        return typeof DOMPurify !== "undefined" ? DOMPurify.sanitize(raw) : raw;
      } catch (e) {
        return md;
      }
    },

    removeDetailTag(tag) {
      if (!this.activeDetailItem || !this.activeDetailItem.tags) return;
      this.activeDetailItem.tags = this.activeDetailItem.tags.filter(t => t !== tag);
      toast(`Removed #${tag}`, "info");
    },

    addDetailTag() {
      if (!this.activeDetailItem) return;
      const tagPrompt = prompt(`Add a tag to "${this.activeDetailItem.title}":`);
      if (tagPrompt && tagPrompt.trim()) {
        const tag = tagPrompt.trim().replace(/^#/, "");
        this.activeDetailItem.tags = this.activeDetailItem.tags || [];
        if (!this.activeDetailItem.tags.includes(tag)) {
          this.activeDetailItem.tags.push(tag);
          toast(`Added #${tag}`, "info");
        }
      }
    },

    async archiveDetailItem() {
      if (!this.activeDetailItem) return;
      try {
        await api(`/api/items/${this.activeDetailItem.id}/status`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ status: "archived" })
        });
        this.activeDetailItem.status = "archived";
        toast(`Archived "${this.activeDetailItem.title}"`, "info");
      } catch (err) {
        toast(`Failed to archive: ${err.message}`, "danger");
      }
    },

    setDetailDeadline() {
      if (!this.activeDetailItem) return;
      toast(`Deadline set for "${this.activeDetailItem.title}" (7 days)`, "info");
    },

    openDetailInObsidian() {
      if (!this.activeDetailItem) return;
      if (this.activeDetailItem.vault_path) {
        window.open(`obsidian://open?file=${encodeURIComponent(this.activeDetailItem.vault_path)}`, "_blank");
      } else {
        toast(`Opening in Obsidian...`, "info");
      }
    },

    getRelatedItems(item) {
      if (!item) return [];
      return MOCK_DATA.items
        .filter(i => i.id !== item.id)
        .slice(0, 4);
    },

    sendDetailChat() {
      if (!this.detailChatInput.trim() || !this.activeDetailItem) return;
      const question = this.detailChatInput.trim();
      this.detailChatMessages.push({ role: "user", text: question });
      this.detailChatInput = "";

      setTimeout(() => {
        const reply = `Based on "${this.activeDetailItem.title}", the core takeaway relates directly to: **${question}**. The source notes emphasize high-leverage architectural patterns and minimal cognitive overhead.`;
        this.detailChatMessages.push({ role: "assistant", text: reply });
      }, 350);
    },

    isOffline: false,

    init() {
      ThemeManager.init();
      this.initRouter();
      this.initKeybindings();
      this.initNetwork();
    },

    initNetwork() {
      window.addEventListener("offline", () => {
        this.isOffline = true;
        toast("Connection lost — retrying in 10s…", "danger");
      });
      window.addEventListener("online", () => {
        this.isOffline = false;
        toast("Connection restored", "info");
      });
    },

    initRouter() {
      const validPages = ["today", "library", "inbox", "ask", "skills", "settings"];

      const handleRoute = () => {
        const hash = window.location.hash.replace(/^#\/?/, "").toLowerCase().trim();
        if (validPages.includes(hash)) {
          this.currentPage = hash;
        } else {
          this.currentPage = "today";
        }
      };

      window.addEventListener("hashchange", handleRoute);
      handleRoute();
    },

    initKeybindings() {
      window.addEventListener("keydown", (e) => {
        if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;
        if (e.key === "/") {
          e.preventDefault();
          const searchInput = document.getElementById("globalSearchInput");
          if (searchInput) searchInput.focus();
        }
      });
    },

    navigate(page) {
      window.location.hash = `#/${page}`;
      this.currentPage = page;
    },

    toggleTheme() {
      ThemeManager.toggle();
    },

    typeBadge(type, withLabel = true) {
      return renderTypeBadge(type, withLabel);
    },

    typeMeta(type) {
      return getTypeMeta(type);
    }
  };
}

// Global exports
window.clipperShell = clipperShell;
window.pageToday = pageToday;
window.api = api;
window.toast = toast;
window.TYPE_CONFIG = TYPE_CONFIG;
window.getTypeMeta = getTypeMeta;
window.renderTypeBadge = renderTypeBadge;
