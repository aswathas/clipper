/**
 * ============================================================================
 * Clipper Dashboard v0.2 — Application Shell & Router
 * ============================================================================
 */

/* ============================================================================
   lib/ — Shared utilities, API client, Type Icons, Decay Bar, Toasts & Modals
   ============================================================================ */

window.__MOCK_API__ = window.__MOCK_API__ || false;

// Toast store / manager
const ToastManager = {
  items: [],
  add(msg, kind = "info") {
    const id = Date.now() + Math.random().toString(36).substring(2, 7);
    const item = { id, msg, kind };
    this.items.push(item);
    setTimeout(() => {
      this.remove(id);
    }, 3000);
  },
  remove(id) {
    const idx = this.items.findIndex((t) => t.id === id);
    if (idx !== -1) this.items.splice(idx, 1);
  }
};

function toast(msg, kind = "info") {
  ToastManager.add(msg, kind);
}

// API Client
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
      if (res.status === 401 || res.status === 403 || res.status >= 500) {
        toast(`[Error ${res.status}] ${errMsg}`, "error");
      }
      const error = new Error(errMsg);
      error.status = res.status;
      error.payload = errPayload;
      throw error;
    }

    if (contentType.includes("application/json")) {
      return await res.json();
    }
    return await res.text();
  } catch (err) {
    if (err.status) throw err;
    // Network / offline error fallback
    console.warn(`[Clipper API] Network call failed for ${path}:`, err);
    return mockApiHandler(path, opts);
  }
}

// Mock handler for offline UI testing and early slicing
function mockApiHandler(path, opts = {}) {
  const method = (opts.method || "GET").toUpperCase();
  if (path.startsWith("/api/today")) {
    return { items: [] };
  }
  if (path.startsWith("/api/items")) {
    return { items: [], total: 0 };
  }
  if (path.startsWith("/api/stats")) {
    return { total: 0, by_type: {}, by_status: {}, stale_30d: 0 };
  }
  if (path === "/healthz") {
    return "ok";
  }
  return { ok: true };
}

// Type Icons — SVG + Color mapping for 8 types
const TYPE_CONFIG = {
  article: {
    colorVar: "var(--type-article)",
    label: "Article",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path></svg>`
  },
  video: {
    colorVar: "var(--type-video)",
    label: "Video",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="23 7 16 12 23 17 23 7"></polygon><rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect></svg>`
  },
  book: {
    colorVar: "var(--type-book)",
    label: "Book",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path></svg>`
  },
  repo: {
    colorVar: "var(--type-repo)",
    label: "Repo",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="4"></circle><line x1="1.05" y1="12" x2="7" y2="12"></line><line x1="17.01" y1="12" x2="22.96" y2="12"></line></svg>`
  },
  "code-error": {
    colorVar: "var(--type-code-error)",
    label: "Code Error",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>`
  },
  photo: {
    colorVar: "var(--type-photo)",
    label: "Photo",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>`
  },
  event: {
    colorVar: "var(--type-event)",
    label: "Event",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect><line x1="16" y1="2" x2="16" y2="6"></line><line x1="8" y1="2" x2="8" y2="6"></line><line x1="3" y1="10" x2="21" y2="10"></line></svg>`
  },
  paper: {
    colorVar: "var(--type-paper)",
    label: "Paper",
    svg: `<svg class="type-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>`
  }
};

function typeIcon(type) {
  const norm = (type || "").toLowerCase().trim();
  return TYPE_CONFIG[norm] || TYPE_CONFIG.article;
}

// Decay Bar — Thin bar: green < 10d, amber < 20d, red >= 20d
function decayBar(daysSinceSave) {
  const days = Math.max(0, Number(daysSinceSave) || 0);
  const widthPercent = Math.min(Math.round((days / 30) * 100), 100);
  let colorClass = "green";
  if (days >= 20) {
    colorClass = "red";
  } else if (days >= 10) {
    colorClass = "amber";
  }
  return {
    days,
    widthPercent,
    colorClass
  };
}

/* ============================================================================
   Theme Management
   ============================================================================ */

const ThemeManager = {
  current: "auto",
  init() {
    const saved = localStorage.getItem("clipper-theme") || "auto";
    this.setTheme(saved);
    // Listen for system theme changes if set to auto
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
   Page Component Factories (Placeholders for HAG-11)
   ============================================================================ */

function pageToday() {
  return {
    title: "Today",
    description: "Urgent review items and recent captures needing action",
    statusText: "Coming soon — HAG-13 will populate this"
  };
}

function pageInbox() {
  return {
    title: "Inbox",
    description: "Unsorted clips awaiting processing, tagging, and filing",
    statusText: "Coming soon — HAG-13 will populate this"
  };
}

function pageLibrary() {
  return {
    title: "Library",
    description: "Archived knowledge base grouped by type, tag, and decay",
    statusText: "Coming soon — HAG-13 will populate this"
  };
}

function pageAsk() {
  return {
    title: "Ask",
    description: "Natural language synthesis and semantic vault search",
    statusText: "Coming soon — HAG-13 will populate this"
  };
}

function pageSkills() {
  return {
    title: "Skills",
    description: "Custom extraction templates, agents, and automation workflows",
    statusText: "Coming soon — HAG-13 will populate this"
  };
}

function pageSettings() {
  return {
    title: "Settings",
    description: "Clipper daemon configuration, vault paths, and device pairing",
    statusText: "Coming soon — HAG-13 will populate this"
  };
}

/* ============================================================================
   Application Shell Root Component
   ============================================================================ */

function clipperShell() {
  return {
    currentPage: "today",
    searchQuery: "",
    scrollPositions: {},
    showShortcutsModal: false,
    showQuickCaptureModal: false,
    quickCaptureUrl: "",
    toasts: ToastManager.items,

    init() {
      ThemeManager.init();
      this.initRouter();
      this.initKeybindings();
    },

    // --- Router ---
    initRouter() {
      const validPages = ["today", "inbox", "library", "ask", "skills", "settings"];

      const handleRoute = () => {
        const hash = window.location.hash.replace(/^#\/?/, "").toLowerCase().trim();
        const prevPage = this.currentPage;

        // Save scroll position of previous page
        if (prevPage) {
          this.scrollPositions[prevPage] = window.scrollY;
        }

        if (validPages.includes(hash)) {
          this.currentPage = hash;
        } else {
          // Default / 404 redirect
          window.location.hash = "#/today";
          this.currentPage = "today";
        }

        // Restore scroll position
        this.$nextTick(() => {
          const savedY = this.scrollPositions[this.currentPage] || 0;
          window.scrollTo(0, savedY);
        });
      };

      window.addEventListener("hashchange", handleRoute);
      handleRoute();
    },

    navigate(page) {
      window.location.hash = `#/${page}`;
    },

    // --- Keyboard Shortcuts ---
    initKeybindings() {
      document.addEventListener("keydown", (e) => {
        const target = e.target;
        const isInputFocused =
          target &&
          (["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName) || target.isContentEditable);

        // Escape: close any modal
        if (e.key === "Escape") {
          if (this.showShortcutsModal || this.showQuickCaptureModal) {
            e.preventDefault();
            this.showShortcutsModal = false;
            this.showQuickCaptureModal = false;
            return;
          }
          if (isInputFocused) {
            target.blur();
            return;
          }
        }

        // Do not trigger single-key shortcuts when typing into input fields
        if (isInputFocused) return;

        // '/' -> focus search input
        if (e.key === "/") {
          e.preventDefault();
          const searchInput = document.getElementById("clipperSearchInput");
          if (searchInput) searchInput.focus();
          return;
        }

        // 'c' -> open quick-capture modal
        if (e.key === "c" || e.key === "C") {
          e.preventDefault();
          this.openQuickCapture();
          return;
        }

        // '?' -> open shortcuts overlay
        if (e.key === "?") {
          e.preventDefault();
          this.showShortcutsModal = true;
          return;
        }

        // 'a' -> on Inbox page, archive selected item (stub: toast 'not yet')
        if (e.key === "a" || e.key === "A") {
          if (this.currentPage === "inbox") {
            e.preventDefault();
            toast("not yet", "info");
          }
          return;
        }

        // 'j' / 'k' -> next/prev in any list (stub for now)
        if (e.key === "j" || e.key === "k") {
          e.preventDefault();
          // Stub for list traversal in HAG-13
          return;
        }
      });
    },

    // --- Actions ---
    toggleTheme() {
      ThemeManager.toggle();
    },

    openQuickCapture() {
      this.quickCaptureUrl = "";
      this.showQuickCaptureModal = true;
      this.$nextTick(() => {
        const input = document.getElementById("quickCaptureInput");
        if (input) input.focus();
      });
    },

    closeModals() {
      this.showShortcutsModal = false;
      this.showQuickCaptureModal = false;
    },

    submitQuickCapture() {
      toast("not yet", "info");
      this.closeModals();
    },

    // Expose helpers to templates
    typeIcon(type) {
      return typeIcon(type);
    },

    decayBar(days) {
      return decayBar(days);
    }
  };
}

// Global exports for Alpine.js
window.clipperShell = clipperShell;
window.pageToday = pageToday;
window.pageInbox = pageInbox;
window.pageLibrary = pageLibrary;
window.pageAsk = pageAsk;
window.pageSkills = pageSkills;
window.pageSettings = pageSettings;
window.api = api;
window.toast = toast;
window.typeIcon = typeIcon;
window.decayBar = decayBar;
