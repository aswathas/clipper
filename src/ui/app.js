/**
 * Clipper Dashboard — Alpine.js Component
 * Follows HAG-4 REST API Contract
 */

// Mock fixtures for offline UI testing and pre-HAG-4 verification
const MOCK_DATA = {
  items: [
    {
      id: "clip-20261015-01",
      url: "https://tauri.app/v2/guides/",
      type: "url",
      source: "Ctrl+Shift+C",
      title: "Tauri 2.0 Architecture & Core Concepts",
      summary: "Comprehensive guide on Tauri v2 multi-process architecture, IPC commands, and system tray integration.",
      tags: ["rust", "tauri", "desktop"],
      saved_at: new Date(Date.now() - 2 * 3600 * 1000).toISOString(),
      status: "inbox",
      vault_path: "Inbox/Tauri 2.0 Architecture.md"
    },
    {
      id: "clip-20261014-02",
      url: "https://alpinejs.dev/start-here",
      type: "url",
      source: "extension",
      title: "Alpine.js Reactive Component Model",
      summary: "Lightweight reactive frontend framework patterns with zero build step.",
      tags: ["javascript", "frontend"],
      saved_at: new Date(Date.now() - 26 * 3600 * 1000).toISOString(),
      status: "inbox",
      vault_path: "Inbox/Alpinejs Reactive Model.md"
    },
    {
      id: "clip-20261012-03",
      url: "file:///screenshots/arch-diagram.png",
      type: "screenshot",
      source: "Win+Shift+S",
      title: "HAG Clipper Pipeline Diagram",
      summary: "ASCII pipeline diagram showing queue drainer to MiniMax MCP vision path.",
      tags: ["design", "spec"],
      saved_at: new Date(Date.now() - 4 * 86400 * 1000).toISOString(),
      status: "inbox",
      vault_path: "Inbox/HAG Clipper Pipeline Diagram.md"
    },
    {
      id: "clip-20260920-04",
      url: "https://picocss.com/docs",
      type: "url",
      source: "manual",
      title: "Pico.css Minimal CSS Framework",
      summary: "Classless and semantic CSS framework for pure HTML elements.",
      tags: ["css", "styling"],
      saved_at: new Date(Date.now() - 25 * 86400 * 1000).toISOString(),
      status: "library",
      vault_path: "Inbox/Pico CSS Framework.md"
    },
    {
      id: "clip-20260901-05",
      url: "file:///photos/whiteboard-notes.jpg",
      type: "photo",
      source: "pwa",
      title: "Whiteboard Sprint Planning Photo",
      summary: "OCR capture of multi-agent floor sprint milestones and ticket ledger.",
      tags: ["notes", "meeting"],
      saved_at: new Date(Date.now() - 38 * 86400 * 1000).toISOString(),
      status: "library",
      vault_path: "Inbox/Whiteboard Sprint Planning Photo.md"
    }
  ],
  today: [
    {
      id: "clip-20261015-01",
      url: "https://tauri.app/v2/guides/",
      type: "url",
      source: "Ctrl+Shift+C",
      title: "Tauri 2.0 Architecture & Core Concepts",
      summary: "Comprehensive guide on Tauri v2 multi-process architecture, IPC commands, and system tray integration.",
      tags: ["rust", "tauri", "desktop"],
      saved_at: new Date(Date.now() - 2 * 3600 * 1000).toISOString(),
      status: "inbox",
      vault_path: "Inbox/Tauri 2.0 Architecture.md"
    },
    {
      id: "clip-20261015-06",
      url: "https://docs.rs/rusqlite/latest/rusqlite/",
      type: "text",
      source: "clipboard",
      title: "Rusqlite FTS5 Configuration Snippet",
      summary: "Tokenizing and indexing schemas for full-text search with SQLite.",
      tags: ["rust", "sqlite"],
      saved_at: new Date(Date.now() - 4 * 3600 * 1000).toISOString(),
      status: "inbox",
      vault_path: "Inbox/Rusqlite FTS5 Snippet.md"
    }
  ],
  stats: {
    total: 12,
    by_type: { url: 8, screenshot: 2, photo: 1, text: 1 },
    by_status: { inbox: 7, library: 4, archived: 1 },
    stale_30d: 2
  }
};

function clipper() {
  return {
    items: [],
    today: [],
    stats: {
      total: 0,
      by_type: {},
      by_status: {},
      stale_30d: 0
    },
    query: "",
    selectedId: null,
    selectedDetail: null,
    showAddModal: false,
    showSettingsModal: false,
    newClipUrl: "",
    isLoading: false,
    searchTimeout: null,

    // Determine mock fallback mode
    isMockMode() {
      return Boolean(
        window.__MOCK_API__ ||
        location.search.includes("mock") ||
        !navigator.onLine
      );
    },

    async apiFetch(url, options = {}) {
      if (this.isMockMode()) {
        return this.mockApiHandler(url, options);
      }
      try {
        const res = await fetch(url, options);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return await res.json();
      } catch (err) {
        // Fallback to mock data if backend server is not reachable yet
        console.warn(`[Clipper API] Fetch failed for ${url}, falling back to mock:`, err);
        return this.mockApiHandler(url, options);
      }
    },

    mockApiHandler(url, options = {}) {
      const method = (options.method || "GET").toUpperCase();

      if (url.startsWith("/api/today")) {
        return { items: [...MOCK_DATA.today] };
      }
      if (url.startsWith("/api/stats")) {
        return { ...MOCK_DATA.stats };
      }
      if (url.startsWith("/api/items?")) {
        const params = new URLSearchParams(url.split("?")[1] || "");
        const q = (params.get("q") || "").toLowerCase();
        let filtered = [...MOCK_DATA.items];
        if (q) {
          filtered = filtered.filter(
            (i) =>
              i.title.toLowerCase().includes(q) ||
              i.summary.toLowerCase().includes(q) ||
              (i.tags || []).some((t) => t.toLowerCase().includes(q))
          );
        }
        return { items: filtered, total: filtered.length };
      }
      if (url.match(/^\/api\/items\/[^/]+$/) && method === "GET") {
        const id = url.split("/api/items/")[1];
        const item = MOCK_DATA.items.find((i) => i.id === id) || MOCK_DATA.today.find((i) => i.id === id);
        return item || { id, title: "Item not found", summary: "", tags: [] };
      }
      if (url.match(/\/status$/) && method === "POST") {
        const body = JSON.parse(options.body || "{}");
        const id = url.split("/")[3];
        const item = MOCK_DATA.items.find((i) => i.id === id);
        if (item) item.status = body.status;
        return { ok: true };
      }
      if (url.match(/\/tags$/) && method === "POST") {
        const body = JSON.parse(options.body || "{}");
        const id = url.split("/")[3];
        const item = MOCK_DATA.items.find((i) => i.id === id);
        if (item) item.tags = body.tags;
        return { ok: true, tags: body.tags };
      }
      if (url === "/api/clips" && method === "POST") {
        const body = JSON.parse(options.body || "{}");
        const newClip = {
          id: `clip-${Date.now()}`,
          url: body.url || "",
          type: body.type || "manual",
          source: "manual",
          title: body.url || "Manual Clip",
          summary: "Queued for extraction and enrichment.",
          tags: ["inbox"],
          saved_at: new Date().toISOString(),
          status: "inbox",
          vault_path: `Inbox/${Date.now()}.md`
        };
        MOCK_DATA.items.unshift(newClip);
        return { id: newClip.id, status: "queued" };
      }
      if (url === "/healthz") {
        return "ok";
      }
      return { ok: true };
    },

    async init() {
      this.isLoading = true;
      try {
        await Promise.all([
          this.fetchToday(),
          this.fetchItems(),
          this.fetchStats()
        ]);
      } finally {
        this.isLoading = false;
      }

      // Watch query with 200ms debounce
      this.$watch("query", (val) => {
        clearTimeout(this.searchTimeout);
        this.searchTimeout = setTimeout(() => {
          this.fetchItems(val);
        }, 200);
      });
    },

    async fetchToday() {
      const data = await this.apiFetch("/api/today");
      this.today = data.items || [];
    },

    async fetchItems(q = "") {
      const queryParam = q ? `&q=${encodeURIComponent(q)}` : "";
      const data = await this.apiFetch(`/api/items?limit=50&offset=0${queryParam}`);
      this.items = data.items || [];
    },

    async fetchStats() {
      const data = await this.apiFetch("/api/stats");
      this.stats = data || { total: 0, by_type: {}, by_status: {}, stale_30d: 0 };
    },

    async selectItem(id) {
      if (this.selectedId === id) {
        this.selectedId = null;
        this.selectedDetail = null;
        return;
      }
      this.selectedId = id;
      const detail = await this.apiFetch(`/api/items/${encodeURIComponent(id)}`);
      this.selectedDetail = detail;
    },

    async updateStatus(id, newStatus) {
      await this.apiFetch(`/api/items/${encodeURIComponent(id)}/status`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ status: newStatus })
      });
      if (newStatus === "deleted") {
        this.items = this.items.filter((i) => i.id !== id);
        this.today = this.today.filter((i) => i.id !== id);
        if (this.selectedId === id) {
          this.selectedId = null;
          this.selectedDetail = null;
        }
      } else {
        const item = this.items.find((i) => i.id === id);
        if (item) item.status = newStatus;
        if (this.selectedDetail && this.selectedDetail.id === id) {
          this.selectedDetail.status = newStatus;
        }
      }
      this.fetchStats();
    },

    async addTag(id) {
      const currentTags = (this.selectedDetail && this.selectedDetail.tags) || [];
      const newTag = prompt("Enter tag name to add:");
      if (!newTag || !newTag.trim()) return;
      const cleanTag = newTag.trim().toLowerCase();
      if (currentTags.includes(cleanTag)) return;

      const updatedTags = [...currentTags, cleanTag];
      await this.apiFetch(`/api/items/${encodeURIComponent(id)}/tags`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tags: updatedTags })
      });

      if (this.selectedDetail && this.selectedDetail.id === id) {
        this.selectedDetail.tags = updatedTags;
      }
      const item = this.items.find((i) => i.id === id);
      if (item) item.tags = updatedTags;
    },

    openAddModal() {
      this.newClipUrl = "";
      this.showAddModal = true;
    },

    closeAddModal() {
      this.showAddModal = false;
      this.newClipUrl = "";
    },

    async submitAddClip() {
      if (!this.newClipUrl || !this.newClipUrl.trim()) return;
      await this.apiFetch("/api/clips", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: this.newClipUrl.trim(), type: "manual" })
      });
      this.closeAddModal();
      await this.fetchItems(this.query);
      await this.fetchStats();
    },

    openSettings() {
      this.showSettingsModal = true;
    },

    closeSettings() {
      this.showSettingsModal = false;
    },

    formatDate(isoString) {
      if (!isoString) return "";
      try {
        const date = new Date(isoString);
        return date.toLocaleDateString(undefined, {
          month: "short",
          day: "numeric",
          hour: "2-digit",
          minute: "2-digit"
        });
      } catch (e) {
        return isoString;
      }
    },

    calculateDecay(isoString) {
      if (!isoString) return 0;
      const days = (Date.now() - new Date(isoString).getTime()) / (86400 * 1000);
      const ratio = days / 30;
      return Math.min(Math.max(ratio, 0), 1);
    },

    getDecayClass(decayRatio) {
      if (decayRatio < 0.3) return "decay-green";
      if (decayRatio < 0.7) return "decay-yellow";
      return "decay-red";
    },

    groupLibrary() {
      const groups = {};
      const libraryItems = this.items;

      for (const item of libraryItems) {
        const type = item.type || "other";
        if (!groups[type]) {
          groups[type] = {
            type,
            count: 0,
            items: [],
            lastSavedAt: null,
            decay: 0,
            decayClass: "decay-green"
          };
        }
        groups[type].items.push(item);
        groups[type].count += 1;

        if (!groups[type].lastSavedAt || new Date(item.saved_at) > new Date(groups[type].lastSavedAt)) {
          groups[type].lastSavedAt = item.saved_at;
        }
      }

      return Object.values(groups).map((group) => {
        const decay = this.calculateDecay(group.lastSavedAt);
        group.decay = decay;
        group.decayPercent = Math.round(decay * 100);
        group.decayClass = this.getDecayClass(decay);
        group.lastTouched = this.formatDate(group.lastSavedAt);
        return group;
      });
    }
  };
}

// Global exposure for Alpine.js
window.clipper = clipper;
