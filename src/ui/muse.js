// Muse floating widget — collapsed pill (240x72) and an expanded popover
// (320x460) with quick settings + recent activity + drag-to-move.
//
// The popover grows TOWARD the bottom-right anchor: when expand() is called
// we capture the current position, then after the resize we set the position
// back so the bottom-right corner stays where it was. When collapse() runs
// we restore the smaller size at the same corner.

const PILL_W = 240, PILL_H = 72;
const POP_W = 320, POP_H = 460;

function museWidget() {
  return {
    museState: "off",
    statusText: "Off — Muse is not watching.",
    expanded: false,
    recentActivity: [],
    cfg: { smart_dedup: true, screenshots: false, debounce_secs: 10, skip_processes: [], record_calls: false, capture_whatsapp: false },
    // Drag state
    drag: { active: false, startX: 0, startY: 0, winX: 0, winY: 0 },
    // Last known on-screen position (logical pixels) — used for position-aware resize.
    lastPos: { x: 0, y: 0 },
    mouse: { x: 0, y: 0 },

    async init() {
      await this.refresh();
      window.addEventListener("mousemove", e => {
        this.mouse = { x: e.clientX, y: e.clientY };
      });
      setInterval(() => this.refresh(), 5000);
    },

    async refresh() {
      try {
        const [state, activity, cfg] = await Promise.all([
          fetch("/api/muse/state").then(r => r.json()).catch(() => null),
          fetch("/api/muse/activity?date=" + this.today() + "&limit=6").then(r => r.json()).catch(() => null),
          fetch("/api/muse/config").then(r => r.json()).catch(() => null),
        ]);
        if (state) {
          this.museState = state.state || "off";
          this.statusText = this.computeStatus();
        }
        if (activity) {
          this.recentActivity = (activity.activity || []).slice(0, 6);
        }
        if (cfg) {
          this.cfg.smart_dedup = !!cfg.smart_dedup;
          this.cfg.screenshots = !!cfg.screenshots;
          this.cfg.debounce_secs = cfg.debounce_secs || 10;
          this.cfg.skip_processes = cfg.skip_processes || [];
          this.cfg.record_calls = !!cfg.record_calls;
          this.cfg.capture_whatsapp = !!cfg.capture_whatsapp;
        }
      } catch (_) {}
    },

    get mood() {
      if (this.museState === "off") return "sleepy";
      if (this.museState === "paused") return "paused";
      return "alert";
    },

    eyeTransform() {
      // The eyes look toward the cursor (clamped to a small box so the
      // expression stays cute, not creepy). We measure the cursor relative
      // to the mascot centre and clamp ±2.5px translation.
      const cx = this.mouse.x, cy = this.mouse.y;
      if (!cx && !cy) return "translate(0 0)";
      const dx = Math.max(-2.5, Math.min(2.5, (cx - 32) / 60));
      const dy = Math.max(-2, Math.min(2, (cy - 32) / 60));
      return `translate(${dx.toFixed(1)} ${dy.toFixed(1)})`;
    },

    async toggle() {
      const next = this.museState === "watching" ? "off" : "watching";
      try {
        const res = await fetch("/api/muse/state", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ state: next })
        });
        const data = await res.json();
        this.museState = data.state || next;
        this.statusText = this.computeStatus();
      } catch (_) {}
    },

    async saveCfg() {
      try {
        await fetch("/api/muse/config", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            smart_dedup: this.cfg.smart_dedup,
            screenshots: this.cfg.screenshots,
            debounce_secs: this.cfg.debounce_secs,
            skip_processes: this.cfg.skip_processes,
            record_calls: this.cfg.record_calls,
            capture_whatsapp: this.cfg.capture_whatsapp,
          })
        });
      } catch (_) {}
    },

    openDashboard() {
      try { window.open("http://127.0.0.1:7137/", "_blank"); } catch (_) {}
    },

    /// Grow the Tauri window to POP size, then snap position so the
    /// bottom-right corner stays anchored to where the pill was.
    expand() {
      this.expanded = true;
      // Position the popover so the bottom-right corner sits where the
      // pill's bottom-right was. We don't know the screen size, but Tauri
      // will keep the existing top-left position on resize; we just adjust
      // the position manually after.
      this.getTauriPos().then(({ x, y }) => {
        this.lastPos = { x, y };
        const dx = POP_W - PILL_W, dy = POP_H - PILL_H;
        this.resizeTauri(POP_W, POP_H);
        // Top-left shifts so bottom-right stays put:
        this.setTauriPos(x - dx, y - dy);
      });
    },

    /// Shrink back. Restore the pill's last top-left + size.
    collapse() {
      this.expanded = false;
      this.resizeTauri(PILL_W, PILL_H);
      if (this.lastPos.x || this.lastPos.y) {
        this.setTauriPos(this.lastPos.x, this.lastPos.y);
      }
    },

    resizeTauri(w, h) {
      try {
        const t = window.__TAURI_INTERNALS__;
        if (t && t.invoke) {
          t.invoke("plugin:window|resize", { width: w, height: h, logical: true });
        }
      } catch (_) {}
    },

    setTauriPos(x, y) {
      try {
        const t = window.__TAURI_INTERNALS__;
        if (t && t.invoke) {
          t.invoke("plugin:window|set_position", { x: Math.round(x), y: Math.round(y) });
        }
      } catch (_) {}
    },

    async getTauriPos() {
      try {
        const t = window.__TAURI_INTERNALS__;
        if (!t || !t.invoke) return { x: 0, y: 0 };
        const outer = await t.invoke("plugin:window|outer_position");
        const sf = await t.invoke("plugin:window|scale_factor").catch(() => 1);
        return { x: (outer.x || 0) / sf, y: (outer.y || 0) / sf };
      } catch (_) {
        return { x: 0, y: 0 };
      }
    },

    /// mousedown on the popover header starts a drag. mousemove updates
    /// the window position; mouseup releases.
    startDrag(e) {
      this.drag.active = true;
      this.drag.startX = e.clientX;
      this.drag.startY = e.clientY;
      this.getTauriPos().then(({ x, y }) => {
        this.drag.winX = x;
        this.drag.winY = y;
      });
      const move = (ev) => {
        if (!this.drag.active) return;
        const nx = this.drag.winX + (ev.clientX - this.drag.startX);
        const ny = this.drag.winY + (ev.clientY - this.drag.startY);
        this.setTauriPos(nx, ny);
      };
      const up = () => {
        this.drag.active = false;
        window.removeEventListener("mousemove", move);
        window.removeEventListener("mouseup", up);
      };
      window.addEventListener("mousemove", move);
      window.addEventListener("mouseup", up);
    },

    computeStatus() {
      if (this.museState === "watching") return "Watching — taking notes.";
      if (this.museState === "paused") return "Paused.";
      return "Off.";
    },

    today() {
      const d = new Date();
      const y = d.getFullYear();
      const m = String(d.getMonth() + 1).padStart(2, "0");
      const day = String(d.getDate()).padStart(2, "0");
      return `${y}-${m}-${day}`;
    },
  };
}
