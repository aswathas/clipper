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
    }
  };
}

// Global exports
window.clipperShell = clipperShell;
window.pageToday = pageToday;
window.api = api;
window.toast = toast;
