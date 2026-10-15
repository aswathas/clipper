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
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path><line x1="8" y1="7" x2="16" y2="7"></line><line x1="8" y1="11" x2="14" y2="11"></line></svg>`
  },
  video: {
    label: "Video",
    colorVar: "var(--type-video)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="4" width="15" height="16" rx="2"></rect><polygon points="22 8 17 12 22 16 22 8"></polygon></svg>`
  },
  book: {
    label: "Book",
    colorVar: "var(--type-book)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path></svg>`
  },
  repo: {
    label: "Repo",
    colorVar: "var(--type-repo)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="4"></circle><line x1="1.05" y1="12" x2="7" y2="12"></line><line x1="17.01" y1="12" x2="22.96" y2="12"></line></svg>`
  },
  "code-error": {
    label: "Code Error",
    colorVar: "var(--type-code-error)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>`
  },
  photo: {
    label: "Photo",
    colorVar: "var(--type-photo)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>`
  },
  event: {
    label: "Event",
    colorVar: "var(--type-event)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="3" y="4" width="18" height="18" rx="2"></rect><line x1="16" y1="2" x2="16" y2="6"></line><line x1="8" y1="2" x2="8" y2="6"></line><line x1="3" y1="10" x2="21" y2="10"></line></svg>`
  },
  paper: {
    label: "Paper",
    colorVar: "var(--type-paper)",
    svg: `<svg class="type-glyph" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>`
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

async function api(path, opts = {}) {
  const isMock = Boolean(
    window.__MOCK_API__ ||
    (typeof location !== "undefined" && location.search.includes("mock")) ||
    (typeof navigator !== "undefined" && !navigator.onLine)
  );

  if (isMock) {
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
    if (err.status) throw err;
    console.warn(`[Clipper API] Network call failed for ${path}, falling back to mock:`, err);
    return mockApiHandler(path, opts);
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

    // 90-day sparkline bars: 90 entries, each with x, height, color, opacity
    sparklineBars: [],

    async init() {
      this.updateGreeting();
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

    generateSparkline() {
      // 90 days of activity. ViewBox 0 0 720 60.
      // Bar width: 6px, gap: 2px (90 bars * 8px = 720px width).
      const bars = [];
      const now = Date.now();
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
    },

    async loadItems() {
      this.loading = true;
      try {
        const res = await api("/api/today");
        this.items = (res.items || []).slice(0, 5);
      } catch (err) {
        console.warn("Failed loading today items:", err);
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
    }
  };
}

function pageLibrary() {
  return {
    title: "Library",
    items: [],
    loading: false,
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
      try {
        const res = await api("/api/items?status=all");
        this.items = res.items || [];
      } catch (err) {
        console.warn("Failed loading library items:", err);
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
    filterMode: "all", // 'all' | 'untagged' | 'needs-deadline' | 'recent'
    selectedIds: [],
    focusedIndex: 0,

    async init() {
      await this.loadItems();
      this.initKeybindings();
    },

    async loadItems() {
      this.loading = true;
      try {
        const res = await api("/api/items?status=inbox");
        const all = res.items || [];
        this.items = all.filter(i => (i.status || "inbox") === "inbox");
      } catch (err) {
        console.warn("Failed to load inbox items:", err);
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

    openDetail(item) {
      this.activeDetailItem = item;
      this.detailChatOpen = false;
      this.detailChatMessages = [
        {
          role: "assistant",
          text: `Ask me anything about "${item.title}". I can synthesize concepts, extract code snippets, or compare it with other clips in your archive.`
        }
      ];
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

    init() {
      ThemeManager.init();
      this.initRouter();
      this.initKeybindings();
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
