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
      vault_path: "Inbox/Is TDD Dead.md"
    }
  ],
  stats: {
    total: 3,
    stale_30d: 0,
    by_status: { inbox: 3, library: 0, archived: 0 },
    by_type: { article: 2, video: 1 }
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

    async init() {
      this.updateGreeting();
      await this.loadItems();
    },

    updateGreeting() {
      const hour = new Date().getHours();
      let timeGreeting = "Good evening";
      if (hour >= 5 && hour < 12) timeGreeting = "Good morning";
      else if (hour >= 12 && hour < 17) timeGreeting = "Good afternoon";
      this.greeting = `${timeGreeting}, Aswath.`;
    },

    async loadItems() {
      this.loading = true;
      try {
        const res = await api("/api/today");
        this.items = res.items || [];
      } catch (err) {
        console.warn("Failed loading today items:", err);
      } finally {
        this.loading = false;
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
