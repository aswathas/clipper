/**
 * ============================================================================
 * Clipper Dashboard v0.2 — Application Shell & Data Store
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

// Robust date formatting supporting epoch seconds (saved_at * 1000), epoch ms, and ISO strings
function formatDate(val) {
  if (!val && val !== 0) return "";
  let ms;
  if (typeof val === "number") {
    ms = val < 1e11 ? val * 1000 : val;
  } else if (!isNaN(Number(val)) && !String(val).includes("-") && !String(val).includes("T")) {
    const n = Number(val);
    ms = n < 1e11 ? n * 1000 : n;
  } else {
    ms = new Date(val).getTime();
  }
  if (isNaN(ms)) return String(val);
  const d = new Date(ms);
  return d.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit"
  });
}

// Compute days since saved
function getDaysSince(val) {
  if (!val && val !== 0) return 0;
  let ms;
  if (typeof val === "number") {
    ms = val < 1e11 ? val * 1000 : val;
  } else if (!isNaN(Number(val)) && !String(val).includes("-")) {
    const n = Number(val);
    ms = n < 1e11 ? n * 1000 : n;
  } else {
    ms = new Date(val).getTime();
  }
  if (isNaN(ms)) return 0;
  const diffMs = Date.now() - ms;
  return Math.max(0, diffMs / (86400 * 1000));
}

// Mock database for offline development & tests
const MOCK_DATA = {
  items: [
    {
      id: "clip-20261015-01",
      url: "https://tauri.app/v2/guides/",
      type: "repo",
      source: "Ctrl+Shift+C",
      title: "Tauri 2.0 Architecture & Core Concepts",
      summary: "Comprehensive guide on Tauri v2 multi-process architecture, IPC commands, and system tray integration.",
      tags: ["rust", "tauri", "desktop"],
      saved_at: Math.floor((Date.now() - 2 * 3600 * 1000) / 1000),
      status: "inbox",
      vault_path: "Inbox/Tauri 2.0 Architecture.md"
    },
    {
      id: "clip-20261014-02",
      url: "https://alpinejs.dev/start-here",
      type: "article",
      source: "extension",
      title: "Alpine.js Reactive Component Model",
      summary: "Lightweight reactive frontend framework patterns with zero build step.",
      tags: ["javascript", "frontend"],
      saved_at: Math.floor((Date.now() - 26 * 3600 * 1000) / 1000),
      status: "inbox",
      vault_path: "Inbox/Alpinejs Reactive Model.md"
    },
    {
      id: "clip-20261012-03",
      url: "file:///screenshots/arch-diagram.png",
      type: "photo",
      source: "Win+Shift+S",
      title: "HAG Clipper Pipeline Diagram",
      summary: "ASCII pipeline diagram showing queue drainer to MiniMax MCP vision path.",
      tags: ["design", "spec"],
      saved_at: Math.floor((Date.now() - 4 * 86400 * 1000) / 1000),
      status: "inbox",
      vault_path: "Inbox/HAG Clipper Pipeline Diagram.md"
    },
    {
      id: "clip-20260920-04",
      url: "https://picocss.com/docs",
      type: "article",
      source: "manual",
      title: "Pico.css Minimal CSS Framework",
      summary: "Classless and semantic CSS framework for pure HTML elements.",
      tags: ["css", "styling"],
      saved_at: Math.floor((Date.now() - 25 * 86400 * 1000) / 1000),
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
      saved_at: Math.floor((Date.now() - 38 * 86400 * 1000) / 1000),
      status: "library",
      vault_path: "Inbox/Whiteboard Sprint Planning Photo.md"
    },
    {
      id: "clip-20261015-06",
      url: "https://docs.rs/rusqlite/latest/rusqlite/",
      type: "code-error",
      source: "clipboard",
      title: "Rusqlite FTS5 Configuration Snippet",
      summary: "Tokenizing and indexing schemas for full-text search with SQLite.",
      tags: ["rust", "sqlite"],
      saved_at: Math.floor((Date.now() - 4 * 3600 * 1000) / 1000),
      status: "inbox",
      vault_path: "Inbox/Rusqlite FTS5 Snippet.md"
    }
  ],
  skills: [
    {
      id: "skill-summarizer",
      name: "Core Summarizer",
      type: "summarizer",
      description: "Extracts key takeaways, core concepts, and high-level bullet points from any article or text.",
      prompt_template: "# Core Summary Template\n\nYou are an analytical summarizer. Extract 3 core takeaways and a 2-sentence executive summary.\n\n### Key Takeaways\n- Focus on actionable insights\n- Highlight trade-offs\n\n### Raw Payload\n{{content}}"
    },
    {
      id: "skill-repo-analyzer",
      name: "GitHub Repo Inspector",
      type: "analyzer",
      description: "Analyzes GitHub repository architecture, dependencies, license, and purpose.",
      prompt_template: "# Repo Inspection Template\n\nInspect the repository structure and describe:\n1. Main architecture\n2. Key dependencies\n3. Primary use cases\n\nRepo: {{url}}"
    },
    {
      id: "skill-code-debugger",
      name: "Code Error Diagnostics",
      type: "code-error",
      description: "Parses compiler stack traces and error diagnostics, suggesting idiomatic fixes.",
      prompt_template: "# Error Diagnostic Template\n\nAnalyze the following error trace and explain the root cause and recommended solution:\n\n```\n{{content}}\n```"
    }
  ],
  memoryParts: {
    soul: {
      content: "---\nversion: 1\ntone: concise, technical\nlanguage: en\nmax_summary_words: 25\navoid: [filler, marketing-speak, hedging, I-think]\n---\n\nYou are Clipper, the user's second brain.\n- Prefer specific facts over vague statements\n- Show, don't tell\n- Skip introductions and preambles\n- Calibrate depth to the user's known expertise\n- Never invent links, citations, or quotes\n- When asked for a list, give the list. When asked for an explanation, give the explanation. No meta.",
      etag: "etag-soul-1"
    },
    user: {
      content: "---\nexpertise: [rust, javascript, linux]\nstyle: concise\ndislikes: [marketing-jargon]\nlanguage: en\n---\n\nUser context:\n- Primary workflows: technical documentation, code snippets, research articles.",
      etag: "etag-user-1"
    },
    memory: {
      content: "# Active\n- Clipper development v0.2 second brain\n\n# Recent focus\n- Alpine.js frontend UI slices\n- SQLite FTS5 full text search\n\n# Today's plan\n- Finish S5 memory/soul routes\n- SSE integration",
      etag: "etag-memory-1"
    }
  },
  patterns: [
    {
      id: "pat-1",
      name: "Weekend Deep Dives",
      confidence: "94%",
      rule: "Clips exceeding 1500 words captured on weekends receive #deep-dive tag.",
      created_at: "2026-10-15"
    },
    {
      id: "pat-2",
      name: "Rust System Architecture",
      confidence: "88%",
      rule: "Articles discussing memory safety, tokio, and axum automatically cross-reference in vault.",
      created_at: "2026-10-14"
    }
  ],
  topics: [
    { name: "Rust", weight: "34%" },
    { name: "Multi-Agent Hives", weight: "28%" },
    { name: "SQLite FTS5", weight: "18%" },
    { name: "Alpine.js UI", weight: "12%" },
    { name: "Tauri v2", weight: "8%" }
  ],
  providers: {
    active: "claude",
    list: [
      { id: "claude", name: "Claude Code CLI", bin: "claude", args: ["-p"], model: "claude-3-7-sonnet", status: "active", desc: "Sole active extraction provider for v1. Fast streaming synthesis and MCP vision support." },
      { id: "codex", name: "OpenAI Codex", bin: "codex", args: ["exec"], model: "gpt-4o", status: "template", desc: "Config template for OpenAI Codex execution engine." },
      { id: "agy", name: "Antigravity CLI (agy)", bin: "agy", args: ["run"], model: "gemini-2.5-pro", status: "template", desc: "Config template for Google Antigravity multi-agent harness." },
      { id: "opencode", name: "OpenCode", bin: "opencode", args: ["pipe"], model: "default", status: "template", desc: "Config template for OpenCode streaming model runner." }
    ],
    toml: `[provider]\nactive = "claude"\n\n[provider.claude]\nbin = "claude"\nargs = ["-p"]\nmodel = "claude-3-7-sonnet"\n\n# [provider.codex]\n# bin = "codex"\n# args = ["exec"]\n# model = "gpt-4o"\n\n# [provider.agy]\n# bin = "agy"\n# args = ["run"]\n# model = "gemini-2.5-pro"\n\n# [provider.opencode]\n# bin = "opencode"\n# args = ["pipe"]`
  },
  pairing: {
    token: "k7X9p2M5n8Q4v1W6y3Z0b4C7d2E9f1A8",
    lan_ip: "192.168.1.150",
    port: 7137,
    pair_url: "http://192.168.1.150:7137/pair?token=k7X9p2M5n8Q4v1W6y3Z0b4C7d2E9f1A8",
    expires_at: Math.floor(Date.now() / 1000) + 24 * 3600
  },
  devices: [
    { id: "dev-phone-1", name: "iPhone 15 Pro", client: "Safari Mobile 17.4", ip: "192.168.1.142", last_active: Math.floor((Date.now() - 120000) / 1000), status: "active" },
    { id: "dev-phone-2", name: "Pixel 8 Pro", client: "Chrome Mobile 122", ip: "192.168.1.168", last_active: Math.floor((Date.now() - 3600000) / 1000), status: "idle" }
  ]
};

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
    console.warn(`[Clipper API] Network call failed for ${path}, falling back to mock:`, err);
    return mockApiHandler(path, opts);
  }
}

// Mock handler for offline UI testing and early slicing
function mockApiHandler(path, opts = {}) {
  const method = (opts.method || "GET").toUpperCase();

  if (path.startsWith("/api/today")) {
    const todayCutoff = Math.floor((Date.now() - 24 * 3600 * 1000) / 1000);
    const items = MOCK_DATA.items.filter((i) => i.saved_at >= todayCutoff || i.tags.includes("urgent"));
    return { items };
  }
  if (path.startsWith("/api/items?")) {
    const params = new URLSearchParams(path.split("?")[1] || "");
    const q = (params.get("q") || "").toLowerCase();
    const type = params.get("type");
    const status = params.get("status");

    let filtered = [...MOCK_DATA.items];
    if (status) {
      filtered = filtered.filter((i) => i.status === status);
    }
    if (type) {
      filtered = filtered.filter((i) => i.type === type);
    }
    if (q) {
      filtered = filtered.filter(
        (i) =>
          (i.title || "").toLowerCase().includes(q) ||
          (i.summary || "").toLowerCase().includes(q) ||
          (i.tags || []).some((t) => t.toLowerCase().includes(q))
      );
    }
    return { items: filtered, total: filtered.length };
  }
  if (path.match(/^\/api\/items\/[^/]+$/) && method === "GET") {
    const id = path.split("/api/items/")[1];
    const item = MOCK_DATA.items.find((i) => i.id === id);
    return item || { id, title: "Item not found", summary: "", tags: [] };
  }
  if (path.match(/\/status$/) && method === "POST") {
    const body = JSON.parse(opts.body || "{}");
    const id = path.split("/")[3];
    const item = MOCK_DATA.items.find((i) => i.id === id);
    if (item) item.status = body.status;
    return { ok: true };
  }
  if (path.match(/\/tags$/) && method === "POST") {
    const body = JSON.parse(opts.body || "{}");
    const id = path.split("/")[3];
    const item = MOCK_DATA.items.find((i) => i.id === id);
    if (item) item.tags = body.tags;
    return { ok: true, tags: body.tags };
  }
  if (path === "/api/clips" && method === "POST") {
    const body = JSON.parse(opts.body || "{}");
    const newClip = {
      id: `clip-${Date.now()}`,
      url: body.url || "",
      type: body.type || "article",
      source: "manual",
      title: body.url || "Manual Clip",
      summary: "Queued for extraction and enrichment.",
      tags: ["inbox"],
      saved_at: Math.floor(Date.now() / 1000),
      status: "inbox",
      vault_path: `Inbox/${Date.now()}.md`
    };
    MOCK_DATA.items.unshift(newClip);
    return { id: newClip.id, status: "queued" };
  }
  if (path.startsWith("/api/stats")) {
    const by_type = {};
    const by_status = {};
    for (const item of MOCK_DATA.items) {
      by_type[item.type] = (by_type[item.type] || 0) + 1;
      by_status[item.status] = (by_status[item.status] || 0) + 1;
    }
    return {
      total: MOCK_DATA.items.length,
      by_type,
      by_status,
      stale_30d: MOCK_DATA.items.filter((i) => getDaysSince(i.saved_at) >= 30).length
    };
  }
  if (path.startsWith("/api/skills") && method === "GET") {
    return { skills: [...(MOCK_DATA.skills || [])] };
  }
  if (path.startsWith("/api/skills") && method === "POST") {
    const body = JSON.parse(opts.body || "{}");
    const newSkill = {
      id: body.id || `skill-${Date.now()}`,
      name: body.name || "Untitled Skill",
      type: body.type || "extractor",
      description: body.description || "",
      prompt_template: body.prompt_template || ""
    };
    if (!MOCK_DATA.skills) MOCK_DATA.skills = [];
    const existingIdx = MOCK_DATA.skills.findIndex((s) => s.id === newSkill.id);
    if (existingIdx !== -1) {
      MOCK_DATA.skills[existingIdx] = newSkill;
    } else {
      MOCK_DATA.skills.push(newSkill);
    }
    return { ok: true, skill: newSkill };
  }
  if (path.match(/^\/api\/skills\/[^/]+$/) && method === "DELETE") {
    const id = path.split("/api/skills/")[1];
    if (MOCK_DATA.skills) {
      MOCK_DATA.skills = MOCK_DATA.skills.filter((s) => s.id !== id);
    }
    return { ok: true };
  }
  if (path.startsWith("/api/memory/") && method === "GET") {
    const part = path.split("/api/memory/")[1];
    const data = (MOCK_DATA.memoryParts && MOCK_DATA.memoryParts[part]) || { content: "", etag: "1" };
    return data;
  }
  if (path.startsWith("/api/memory/") && method === "POST") {
    const part = path.split("/api/memory/")[1];
    const body = JSON.parse(opts.body || "{}");
    if (!MOCK_DATA.memoryParts) MOCK_DATA.memoryParts = {};
    const newEtag = `etag-${Date.now()}`;
    MOCK_DATA.memoryParts[part] = {
      content: body.content || "",
      etag: newEtag
    };
    return { ok: true, etag: newEtag };
  }
  if (path.startsWith("/api/self/patterns") && method === "GET") {
    return { patterns: [...(MOCK_DATA.patterns || [])] };
  }
  if (path.startsWith("/api/self/topics") && method === "GET") {
    return { topics: [...(MOCK_DATA.topics || [])] };
  }
  if (path === "/api/self/recompute" && method === "POST") {
    const newPat = {
      id: `pat-${Date.now()}`,
      name: "Autonomous Agent Pairing",
      confidence: "91%",
      rule: "Clips with agent coordination patterns cross-linked with PROTOCOL.md.",
      created_at: new Date().toISOString().split("T")[0]
    };
    if (!MOCK_DATA.patterns) MOCK_DATA.patterns = [];
    MOCK_DATA.patterns.unshift(newPat);
    return { ok: true, pattern: newPat };
  }
  if (path === "/api/providers" && method === "GET") {
    return JSON.parse(JSON.stringify(MOCK_DATA.providers));
  }
  if (path === "/api/providers" && method === "POST") {
    if (opts.body) {
      try {
        const payload = JSON.parse(opts.body);
        if (payload.active) MOCK_DATA.providers.active = payload.active;
        if (payload.toml) MOCK_DATA.providers.toml = payload.toml;
      } catch (e) {}
    }
    return { ok: true };
  }
  if (path === "/api/pairing" && method === "GET") {
    return JSON.parse(JSON.stringify(MOCK_DATA.pairing));
  }
  if (path === "/api/pairing/regenerate" && method === "POST") {
    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let token = "";
    for (let i = 0; i < 32; i++) {
      token += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    const expires = Math.floor(Date.now() / 1000) + 24 * 3600;
    MOCK_DATA.pairing.token = token;
    MOCK_DATA.pairing.expires_at = expires;
    MOCK_DATA.pairing.pair_url = `http://${MOCK_DATA.pairing.lan_ip}:${MOCK_DATA.pairing.port}/pair?token=${token}`;
    return { ok: true, token, expires_at: expires, pair_url: MOCK_DATA.pairing.pair_url };
  }
  if (path === "/api/devices" && method === "GET") {
    return { devices: JSON.parse(JSON.stringify(MOCK_DATA.devices)) };
  }
  if (path.startsWith("/api/devices/") && path.endsWith("/revoke") && method === "POST") {
    const id = path.split("/")[3];
    MOCK_DATA.devices = MOCK_DATA.devices.filter(d => d.id !== id);
    return { ok: true };
  }
  if (path === "/api/devices/revoke-all" && method === "POST") {
    MOCK_DATA.devices = [];
    return { ok: true };
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
   Global Clips Store (Optimistic Mutations & Drawer State)
   ============================================================================ */

const ClipStore = {
  activeDetailItem: null,
  showDetailDrawer: false,
  selectedIds: [],
  newTagInput: "",
  sseConnected: false,

  openDetail(item) {
    this.activeDetailItem = JSON.parse(JSON.stringify(item));
    this.showDetailDrawer = true;
    this.newTagInput = "";
  },

  closeDetail() {
    this.showDetailDrawer = false;
    this.activeDetailItem = null;
    this.newTagInput = "";
  },

  toggleSelect(id) {
    const idx = this.selectedIds.indexOf(id);
    if (idx === -1) {
      this.selectedIds.push(id);
    } else {
      this.selectedIds.splice(idx, 1);
    }
  },

  isSelected(id) {
    return this.selectedIds.includes(id);
  },

  selectAll(items) {
    this.selectedIds = items.map((i) => i.id);
  },

  deselectAll() {
    this.selectedIds = [];
  }
};

/* ============================================================================
   Page Component Factories (HAG-13: Today / Inbox / Library)
   ============================================================================ */

// 1. TODAY PAGE COMPONENT
function pageToday() {
  return {
    title: "Today",
    description: "Urgent items and clips captured within the last 24 hours",
    items: [],
    loading: true,
    filterType: "all",

    async init() {
      await this.loadItems();
    },

    async loadItems() {
      this.loading = true;
      try {
        const data = await api("/api/today");
        this.items = data.items || [];
      } catch (e) {
        console.error("Failed loading Today items:", e);
      } finally {
        this.loading = false;
      }
    },

    filteredItems() {
      if (this.filterType === "all") return this.items;
      return this.items.filter((i) => i.type === this.filterType);
    },

    // Optimistic status update
    async setStatus(id, newStatus) {
      const prevItems = JSON.parse(JSON.stringify(this.items));
      const target = this.items.find((i) => i.id === id);
      if (target) target.status = newStatus;
      if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
        ClipStore.activeDetailItem.status = newStatus;
      }
      toast(`Marked as ${newStatus}`, "saved");

      try {
        await api(`/api/items/${encodeURIComponent(id)}/status`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ status: newStatus })
        });
      } catch (err) {
        this.items = prevItems;
        if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
          ClipStore.activeDetailItem.status = prevItems.find((i) => i.id === id)?.status || "inbox";
        }
        toast(`Failed to update status: ${err.message}`, "error");
      }
    },

    // Optimistic tag additions
    async addTag(id, tag) {
      if (!tag || !tag.trim()) return;
      const cleanTag = tag.trim().toLowerCase();
      const target = this.items.find((i) => i.id === id);
      if (!target) return;
      if (!target.tags) target.tags = [];
      if (target.tags.includes(cleanTag)) return;

      const prevTags = [...target.tags];
      target.tags.push(cleanTag);
      if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
        ClipStore.activeDetailItem.tags = [...target.tags];
      }

      try {
        await api(`/api/items/${encodeURIComponent(id)}/tags`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ tags: target.tags })
        });
        toast(`Tag #${cleanTag} added`, "saved");
      } catch (err) {
        target.tags = prevTags;
        if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
          ClipStore.activeDetailItem.tags = prevTags;
        }
        toast(`Failed to save tag: ${err.message}`, "error");
      }
    },

    // Optimistic tag removal
    async removeTag(id, tagToRemove) {
      const target = this.items.find((i) => i.id === id);
      if (!target || !target.tags) return;
      const prevTags = [...target.tags];
      target.tags = target.tags.filter((t) => t !== tagToRemove);
      if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
        ClipStore.activeDetailItem.tags = [...target.tags];
      }

      try {
        await api(`/api/items/${encodeURIComponent(id)}/tags`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ tags: target.tags })
        });
      } catch (err) {
        target.tags = prevTags;
        if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
          ClipStore.activeDetailItem.tags = prevTags;
        }
        toast(`Failed to remove tag: ${err.message}`, "error");
      }
    }
  };
}

// 2. INBOX PAGE COMPONENT
function pageInbox() {
  return {
    title: "Inbox",
    description: "New captures awaiting review, tagging, or filing",
    items: [],
    loading: true,
    filterType: "all",
    searchFilter: "",

    async init() {
      await this.loadItems();
    },

    async loadItems() {
      this.loading = true;
      try {
        const data = await api("/api/items?limit=50&offset=0&status=inbox");
        this.items = data.items || [];
      } catch (e) {
        console.error("Failed loading Inbox items:", e);
      } finally {
        this.loading = false;
      }
    },

    filteredItems() {
      let result = this.items.filter((i) => i.status !== "archived" && i.status !== "deleted");
      if (this.filterType !== "all") {
        result = result.filter((i) => i.type === this.filterType);
      }
      if (this.searchFilter) {
        const q = this.searchFilter.toLowerCase();
        result = result.filter(
          (i) =>
            (i.title || "").toLowerCase().includes(q) ||
            (i.summary || "").toLowerCase().includes(q) ||
            (i.tags || []).some((t) => t.toLowerCase().includes(q))
        );
      }
      return result;
    },

    // Optimistic status update
    async setStatus(id, newStatus) {
      const prevItems = JSON.parse(JSON.stringify(this.items));
      const target = this.items.find((i) => i.id === id);
      if (target) target.status = newStatus;
      if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
        ClipStore.activeDetailItem.status = newStatus;
      }

      // If archiving or deleting, optimistically slide out
      if (newStatus === "archived" || newStatus === "deleted") {
        this.items = this.items.filter((i) => i.id !== id);
      }
      toast(`Marked as ${newStatus}`, "saved");

      try {
        await api(`/api/items/${encodeURIComponent(id)}/status`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ status: newStatus })
        });
      } catch (err) {
        this.items = prevItems;
        if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
          ClipStore.activeDetailItem.status = prevItems.find((i) => i.id === id)?.status || "inbox";
        }
        toast(`Failed to update status: ${err.message}`, "error");
      }
    },

    // Optimistic tag additions
    async addTag(id, tag) {
      if (!tag || !tag.trim()) return;
      const cleanTag = tag.trim().toLowerCase();
      const target = this.items.find((i) => i.id === id);
      if (!target) return;
      if (!target.tags) target.tags = [];
      if (target.tags.includes(cleanTag)) return;

      const prevTags = [...target.tags];
      target.tags.push(cleanTag);
      if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
        ClipStore.activeDetailItem.tags = [...target.tags];
      }

      try {
        await api(`/api/items/${encodeURIComponent(id)}/tags`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ tags: target.tags })
        });
        toast(`Tag #${cleanTag} added`, "saved");
      } catch (err) {
        target.tags = prevTags;
        if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
          ClipStore.activeDetailItem.tags = prevTags;
        }
        toast(`Failed to save tag: ${err.message}`, "error");
      }
    },

    // Optimistic tag removal
    async removeTag(id, tagToRemove) {
      const target = this.items.find((i) => i.id === id);
      if (!target || !target.tags) return;
      const prevTags = [...target.tags];
      target.tags = target.tags.filter((t) => t !== tagToRemove);
      if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
        ClipStore.activeDetailItem.tags = [...target.tags];
      }

      try {
        await api(`/api/items/${encodeURIComponent(id)}/tags`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ tags: target.tags })
        });
      } catch (err) {
        target.tags = prevTags;
        if (ClipStore.activeDetailItem && ClipStore.activeDetailItem.id === id) {
          ClipStore.activeDetailItem.tags = prevTags;
        }
        toast(`Failed to remove tag: ${err.message}`, "error");
      }
    },

    // Bulk actions
    async bulkArchive() {
      const ids = [...ClipStore.selectedIds];
      if (!ids.length) return;
      const count = ids.length;
      this.items = this.items.filter((i) => !ids.includes(i.id));
      ClipStore.deselectAll();
      toast(`Archiving ${count} clips...`, "saved");

      await Promise.all(
        ids.map((id) =>
          api(`/api/items/${encodeURIComponent(id)}/status`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ status: "archived" })
          }).catch((e) => console.warn("Bulk archive item failed:", e))
        )
      );
    },

    async bulkDelete() {
      const ids = [...ClipStore.selectedIds];
      if (!ids.length) return;
      const count = ids.length;
      this.items = this.items.filter((i) => !ids.includes(i.id));
      ClipStore.deselectAll();
      toast(`Deleted ${count} clips`, "saved");

      await Promise.all(
        ids.map((id) =>
          api(`/api/items/${encodeURIComponent(id)}/status`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ status: "deleted" })
          }).catch((e) => console.warn("Bulk delete item failed:", e))
        )
      );
    },

    async bulkTag() {
      const ids = [...ClipStore.selectedIds];
      if (!ids.length) return;
      const tag = prompt("Enter tag to apply to selected clips:");
      if (!tag || !tag.trim()) return;
      const cleanTag = tag.trim().toLowerCase();

      for (const id of ids) {
        const item = this.items.find((i) => i.id === id);
        if (item) {
          if (!item.tags) item.tags = [];
          if (!item.tags.includes(cleanTag)) item.tags.push(cleanTag);
        }
      }
      ClipStore.deselectAll();
      toast(`Tagged ${ids.length} clips with #${cleanTag}`, "saved");

      await Promise.all(
        ids.map((id) => {
          const item = this.items.find((i) => i.id === id);
          return api(`/api/items/${encodeURIComponent(id)}/tags`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ tags: item?.tags || [cleanTag] })
          }).catch((e) => console.warn("Bulk tag item failed:", e));
        })
      );
    }
  };
}

// 3. LIBRARY PAGE COMPONENT
function pageLibrary() {
  return {
    title: "Library",
    description: "Knowledge vault grouped by media type with memory decay telemetry",
    items: [],
    loading: true,
    groupBy: "type", // 'type' | 'decay'
    selectedTypeFilter: "all",

    async init() {
      await this.loadItems();
    },

    async loadItems() {
      this.loading = true;
      try {
        const data = await api("/api/items?limit=100&offset=0");
        this.items = data.items || [];
      } catch (e) {
        console.error("Failed loading Library items:", e);
      } finally {
        this.loading = false;
      }
    },

    groups() {
      const map = {};
      for (const item of this.items) {
        const key = item.type || "article";
        if (this.selectedTypeFilter !== "all" && key !== this.selectedTypeFilter) continue;

        if (!map[key]) {
          map[key] = {
            type: key,
            count: 0,
            items: [],
            lastSavedAt: null
          };
        }
        map[key].items.push(item);
        map[key].count += 1;

        if (!map[key].lastSavedAt || item.saved_at > map[key].lastSavedAt) {
          map[key].lastSavedAt = item.saved_at;
        }
      }

      return Object.values(map).map((g) => {
        const days = getDaysSince(g.lastSavedAt);
        const bar = decayBar(days);
        return {
          ...g,
          decayInfo: bar,
          lastTouchedStr: formatDate(g.lastSavedAt)
        };
      });
    }
  };
}

// Placeholder factories for HAG-14+
function pageAsk() {
  return {
    title: "Ask",
    description: "Natural language synthesis and semantic vault search",
    statusText: "Coming soon — HAG-18 will populate this"
  };
}

function pageSkills() {
  return {
    title: "Skills",
    description: "Custom extraction templates, agents, and automation workflows",
    skills: [],
    loading: true,
    isEditing: false,
    editForm: {
      id: "",
      name: "",
      type: "summarizer",
      description: "",
      prompt_template: ""
    },

    async init() {
      await this.loadSkills();
    },

    async loadSkills() {
      this.loading = true;
      try {
        const data = await api("/api/skills");
        this.skills = data.skills || [];
      } catch (e) {
        console.error("Failed loading Skills:", e);
      } finally {
        this.loading = false;
      }
    },

    newSkill() {
      this.editForm = {
        id: "skill-" + Date.now(),
        name: "",
        type: "summarizer",
        description: "",
        prompt_template: "# New Extraction Skill\n\nInstructions for Claude:\n- Extract key facts\n- Synthesize action points\n\n### Content\n{{content}}"
      };
      this.isEditing = true;
    },

    editSkill(skill) {
      this.editForm = JSON.parse(JSON.stringify(skill));
      this.isEditing = true;
    },

    cancelEdit() {
      this.isEditing = false;
    },

    renderPreview() {
      const raw = this.editForm.prompt_template || "*No prompt template entered.*";
      try {
        if (typeof marked !== "undefined" && typeof DOMPurify !== "undefined") {
          return DOMPurify.sanitize(marked.parse(raw));
        }
        if (typeof DOMPurify !== "undefined") {
          return DOMPurify.sanitize(raw);
        }
      } catch (e) {
        console.warn("Markdown render error:", e);
      }
      return raw;
    },

    async saveSkill() {
      if (!this.editForm.name || !this.editForm.name.trim()) {
        toast("Please enter a skill name", "error");
        return;
      }
      const skillToSave = JSON.parse(JSON.stringify(this.editForm));
      const idx = this.skills.findIndex((s) => s.id === skillToSave.id);
      if (idx !== -1) {
        this.skills[idx] = skillToSave;
      } else {
        this.skills.push(skillToSave);
      }
      this.isEditing = false;
      toast(`Skill "${skillToSave.name}" saved`, "saved");

      try {
        await api("/api/skills", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(skillToSave)
        });
      } catch (err) {
        toast(`Failed to save skill: ${err.message}`, "error");
      }
    },

    async deleteSkill(id) {
      const target = this.skills.find((s) => s.id === id);
      const name = target?.name || id;
      this.skills = this.skills.filter((s) => s.id !== id);
      if (this.isEditing && this.editForm.id === id) {
        this.isEditing = false;
      }
      toast(`Deleted skill "${name}"`, "saved");

      try {
        await api(`/api/skills/${encodeURIComponent(id)}`, {
          method: "DELETE"
        });
      } catch (err) {
        toast(`Failed deleting skill: ${err.message}`, "error");
      }
    }
  };
}

function pageSettings() {
  return {
    title: "Settings",
    description: "Clipper second brain memory, soul persona, and system configuration",
    activeTab: "soul", // 'soul' | 'memory' | 'user' | 'self' | 'providers' | 'pairing' | 'devices' | 'daemon'
    loading: false,
    editMode: false,
    parts: {
      soul: { content: "", etag: "1", dirty: false },
      memory: { content: "", etag: "1", dirty: false },
      user: { content: "", etag: "1", dirty: false }
    },
    stats: { total: 0, by_type: {}, by_status: {}, stale_30d: 0 },
    patterns: [],
    topics: [],
    providers: {
      active: "claude",
      list: [],
      toml: ""
    },
    pairing: {
      token: "",
      lan_ip: "127.0.0.1",
      port: 7137,
      pair_url: "",
      expires_at: 0,
      ttl_formatted: "24h 00m",
      qr_url: "/pair.png"
    },
    devices: [],
    showRegenerateModal: false,
    regenerating: false,
    ttlInterval: null,
    store: ClipStore,
    formatDate(val) {
      return formatDate(val);
    },

    async init() {
      await this.loadAll();
    },

    async loadAll() {
      this.loading = true;
      try {
        const [soulData, memData, userData, statsData, patData, topData, provData, pairData, devData] = await Promise.all([
          api("/api/memory/soul").catch(() => ({ content: "", etag: "1" })),
          api("/api/memory/memory").catch(() => ({ content: "", etag: "1" })),
          api("/api/memory/user").catch(() => ({ content: "", etag: "1" })),
          api("/api/stats").catch(() => ({})),
          api("/api/self/patterns").catch(() => ({ patterns: [] })),
          api("/api/self/topics").catch(() => ({ topics: [] })),
          api("/api/providers").catch(() => ({ active: "claude", list: [], toml: "" })),
          api("/api/pairing").catch(() => ({ token: "", lan_ip: "127.0.0.1", port: 7137, pair_url: "", expires_at: 0 })),
          api("/api/devices").catch(() => ({ devices: [] }))
        ]);
        this.parts.soul = { content: soulData.content || "", etag: soulData.etag || "1", dirty: false };
        this.parts.memory = { content: memData.content || "", etag: memData.etag || "1", dirty: false };
        this.parts.user = { content: userData.content || "", etag: userData.etag || "1", dirty: false };
        this.stats = statsData || {};
        this.patterns = patData.patterns || [];
        this.topics = topData.topics || [];
        this.providers = provData || { active: "claude", list: [], toml: "" };
        this.pairing = {
          token: pairData.token || "",
          lan_ip: pairData.lan_ip || "127.0.0.1",
          port: pairData.port || 7137,
          pair_url: pairData.pair_url || "",
          expires_at: pairData.expires_at || 0,
          ttl_formatted: "24h 00m",
          qr_url: "/pair.png"
        };
        this.devices = devData.devices || [];
        this.updateTTL();
        if (!this.ttlInterval) {
          this.ttlInterval = setInterval(() => this.updateTTL(), 10000);
        }
      } finally {
        this.loading = false;
      }
    },

    addPattern(pat) {
      if (!this.patterns) this.patterns = [];
      this.patterns.unshift(pat);
    },

    async recomputeSelf() {
      toast("Recomputing self-improvement patterns...", "info");
      try {
        const res = await api("/api/self/recompute", { method: "POST" });
        if (res.pattern) {
          this.addPattern(res.pattern);
          toast(`Discovered pattern: ${res.pattern.name}`, "saved");
        } else {
          toast("Self-improvement analysis up to date", "saved");
        }
      } catch (e) {
        toast(`Recompute error: ${e.message}`, "error");
      }
    },

    setTab(tab) {
      this.activeTab = tab;
      this.editMode = false;
      this.checkCodeMirror();
    },

    currentContent() {
      return this.parts[this.activeTab]?.content || "";
    },

    renderMarkdown() {
      const raw = this.currentContent();
      try {
        if (typeof marked !== "undefined" && typeof DOMPurify !== "undefined") {
          return DOMPurify.sanitize(marked.parse(raw || "*Empty file.*"));
        }
        if (typeof DOMPurify !== "undefined") {
          return DOMPurify.sanitize(raw);
        }
      } catch (e) {
        console.warn("Markdown render error:", e);
      }
      return raw;
    },

    // Lazy load CodeMirror from CDN if above 10KB (> 10240 characters)
    checkCodeMirror() {
      const content = this.currentContent();
      if (content.length > 10240 && !window.CodeMirror) {
        const link = document.createElement("link");
        link.rel = "stylesheet";
        link.href = "https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.16/codemirror.min.css";
        document.head.appendChild(link);

        const script = document.createElement("script");
        script.src = "https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.16/codemirror.min.js";
        script.onload = () => {
          const modeScript = document.createElement("script");
          modeScript.src = "https://cdnjs.cloudflare.com/ajax/libs/codemirror/5.65.16/mode/markdown/markdown.min.js";
          document.head.appendChild(modeScript);
        };
        document.head.appendChild(script);
      }
    },

    async saveCurrentPart() {
      const part = this.activeTab;
      const target = this.parts[part];
      if (!target) return;

      toast(`Saving ${part}.md...`, "info");
      try {
        const res = await api(`/api/memory/${encodeURIComponent(part)}`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "If-Match": target.etag || ""
          },
          body: JSON.stringify({ content: target.content })
        });
        if (res.etag) {
          target.etag = res.etag;
        }
        this.editMode = false;
        toast(`Successfully saved ${part}.md`, "saved");
      } catch (err) {
        if (err.status === 412) {
          toast(`Conflict: ${part}.md was modified elsewhere (412 Precondition Failed)`, "error");
        } else {
          toast(`Failed saving ${part}.md: ${err.message}`, "error");
        }
      }
    },

    updateTTL() {
      if (!this.pairing.expires_at) {
        this.pairing.ttl_formatted = "24h 00m";
        return;
      }
      const now = Math.floor(Date.now() / 1000);
      const diffSec = Math.max(0, this.pairing.expires_at - now);
      const hours = Math.floor(diffSec / 3600);
      const minutes = Math.floor((diffSec % 3600) / 60);
      this.pairing.ttl_formatted = `${hours}h ${minutes < 10 ? "0" : ""}${minutes}m remaining`;
    },

    async selectProvider(id) {
      this.providers.active = id;
      toast(`Active provider set to ${id}`, "saved");
      try {
        await api("/api/providers", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ active: id, toml: this.providers.toml })
        });
      } catch (e) {
        toast(`Failed updating provider: ${e.message}`, "error");
      }
    },

    async saveProvidersToml() {
      toast("Saving config.toml...", "info");
      try {
        await api("/api/providers", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ active: this.providers.active, toml: this.providers.toml })
        });
        toast("Saved ~/.clipper/config.toml", "saved");
      } catch (e) {
        toast(`Failed saving config: ${e.message}`, "error");
      }
    },

    openRegenerateModal() {
      this.showRegenerateModal = true;
    },

    closeRegenerateModal() {
      this.showRegenerateModal = false;
    },

    async confirmRegeneratePairing() {
      this.regenerating = true;
      try {
        const res = await api("/api/pairing/regenerate", { method: "POST" });
        if (res.token) {
          this.pairing.token = res.token;
          this.pairing.expires_at = res.expires_at || (Math.floor(Date.now() / 1000) + 24 * 3600);
          this.pairing.pair_url = `http://${this.pairing.lan_ip}:${this.pairing.port}/pair?token=${res.token}`;
        }
        if (typeof navigator !== "undefined" && navigator.serviceWorker && navigator.serviceWorker.controller) {
          navigator.serviceWorker.controller.postMessage({ type: "INVALIDATE_QR" });
        }
        this.pairing.qr_url = `/pair.png?t=${Date.now()}`;
        this.updateTTL();
        toast("Pairing token regenerated. Active phone sessions revoked.", "saved");
      } catch (err) {
        toast(`Failed to regenerate token: ${err.message}`, "error");
      } finally {
        this.regenerating = false;
        this.showRegenerateModal = false;
      }
    },

    async copyText(text, label = "Copied") {
      try {
        if (navigator.clipboard) {
          await navigator.clipboard.writeText(text);
          toast(`${label} copied to clipboard`, "saved");
        } else {
          toast("Clipboard unavailable", "error");
        }
      } catch (e) {
        toast("Failed to copy", "error");
      }
    },

    async revokeDevice(id) {
      const dev = this.devices.find(d => d.id === id);
      const name = dev?.name || id;
      this.devices = this.devices.filter(d => d.id !== id);
      toast(`Revoked device "${name}"`, "saved");
      try {
        await api(`/api/devices/${encodeURIComponent(id)}/revoke`, { method: "POST" });
      } catch (e) {
        toast(`Failed revoking device: ${e.message}`, "error");
      }
    },

    async revokeAllDevices() {
      this.devices = [];
      toast("Revoked all paired phone devices", "saved");
      try {
        await api("/api/devices/revoke-all", { method: "POST" });
      } catch (e) {
        toast(`Failed revoking devices: ${e.message}`, "error");
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
    scrollPositions: {},
    showShortcutsModal: false,
    showQuickCaptureModal: false,
    quickCaptureUrl: "",
    toasts: ToastManager.items,
    store: ClipStore,
    sseConnected: false,
    eventSource: null,

    init() {
      ThemeManager.init();
      this.initRouter();
      this.initKeybindings();
      this.initSSE();
      if (typeof navigator !== "undefined" && "serviceWorker" in navigator) {
        navigator.serviceWorker.register("/sw.js").catch((e) => {
          console.debug("Service worker registration:", e);
        });
      }
    },

    initSSE() {
      if (typeof EventSource === "undefined") return;
      if (window.__MOCK_API__ || !navigator.onLine) {
        this.sseConnected = true;
        ClipStore.sseConnected = true;
        return;
      }

      try {
        const es = new EventSource("/api/events");
        this.eventSource = es;

        es.onopen = () => {
          this.sseConnected = true;
          ClipStore.sseConnected = true;
        };

        es.onerror = () => {
          this.sseConnected = false;
          ClipStore.sseConnected = false;
        };

        const handlePatternAdded = (data) => {
          const pat = typeof data === "string" ? JSON.parse(data) : data;
          toast(`Pattern discovered: ${pat.name || pat.rule || "New user pattern"}`, "saved");
          const settingsComp = document.querySelector('[x-data*="pageSettings"]');
          if (settingsComp && settingsComp._x_dataStack) {
            settingsComp._x_dataStack[0].addPattern(pat);
          }
        };

        const handleTokenRotated = (data) => {
          const payload = typeof data === "string" ? JSON.parse(data) : data;
          if (typeof navigator !== "undefined" && navigator.serviceWorker && navigator.serviceWorker.controller) {
            navigator.serviceWorker.controller.postMessage({ type: "INVALIDATE_QR" });
          }
          toast("Session token rotated by daemon (24h)", "info");
          const settingsComp = document.querySelector('[x-data*="pageSettings"]');
          if (settingsComp && settingsComp._x_dataStack) {
            const comp = settingsComp._x_dataStack[0];
            comp.pairing.token = payload.token;
            comp.pairing.expires_at = payload.expires_at || (Math.floor(Date.now() / 1000) + 24 * 3600);
            comp.pairing.pair_url = `http://${comp.pairing.lan_ip}:${comp.pairing.port}/pair?token=${payload.token}`;
            comp.pairing.qr_url = `/pair.png?t=${Date.now()}`;
            comp.updateTTL();
          }
        };

        es.addEventListener("pattern_added", (e) => {
          try {
            handlePatternAdded(e.data);
          } catch (err) {
            console.warn("SSE parse error:", err);
          }
        });

        es.addEventListener("token_rotated", (e) => {
          try {
            handleTokenRotated(e.data);
          } catch (err) {
            console.warn("Token rotation SSE error:", err);
          }
        });

        es.onmessage = (e) => {
          try {
            const payload = JSON.parse(e.data);
            if (payload.type === "pattern_added") {
              handlePatternAdded(payload.pattern || payload);
            } else if (payload.type === "token_rotated") {
              handleTokenRotated(payload);
            }
          } catch (err) {}
        };
      } catch (err) {
        console.warn("EventSource setup failed:", err);
      }
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

        // Escape: close any open modal or detail drawer
        if (e.key === "Escape") {
          if (this.showShortcutsModal || this.showQuickCaptureModal) {
            e.preventDefault();
            this.showShortcutsModal = false;
            this.showQuickCaptureModal = false;
            return;
          }
          if (ClipStore.showDetailDrawer) {
            e.preventDefault();
            ClipStore.closeDetail();
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

        // 'a' -> on Inbox page, archive selected item(s)
        if (e.key === "a" || e.key === "A") {
          if (this.currentPage === "inbox") {
            e.preventDefault();
            if (ClipStore.selectedIds.length > 0) {
              // Trigger bulk archive
              const inboxComp = document.querySelector('[x-data*="pageInbox"]');
              if (inboxComp && inboxComp._x_dataStack) {
                inboxComp._x_dataStack[0].bulkArchive();
              }
            } else if (ClipStore.activeDetailItem) {
              const id = ClipStore.activeDetailItem.id;
              ClipStore.closeDetail();
              const inboxComp = document.querySelector('[x-data*="pageInbox"]');
              if (inboxComp && inboxComp._x_dataStack) {
                inboxComp._x_dataStack[0].setStatus(id, "archived");
              }
            } else {
              toast("No item selected to archive", "info");
            }
          }
          return;
        }

        // 'j' / 'k' -> next/prev in any list (stub)
        if (e.key === "j" || e.key === "k") {
          e.preventDefault();
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

    // Optimistic add-clip
    async submitQuickCapture() {
      if (!this.quickCaptureUrl || !this.quickCaptureUrl.trim()) return;
      const url = this.quickCaptureUrl.trim();
      this.closeModals();

      toast("Clip queued for processing", "saved");

      try {
        await api("/api/clips", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ url, type: "manual" })
        });
        // Dispatch refresh to Inbox if mounted
        const inboxComp = document.querySelector('[x-data*="pageInbox"]');
        if (inboxComp && inboxComp._x_dataStack) {
          inboxComp._x_dataStack[0].loadItems();
        }
      } catch (err) {
        toast(`Failed to save clip: ${err.message}`, "error");
      }
    },

    // Expose helpers to templates
    typeIcon(type) {
      return typeIcon(type);
    },

    decayBar(days) {
      return decayBar(days);
    },

    formatDate(val) {
      return formatDate(val);
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
window.formatDate = formatDate;
window.ClipStore = ClipStore;
