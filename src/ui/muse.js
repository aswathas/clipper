// Muse floating widget — collapsed pill (240x72) and an expanded popover
// (320x440) with quick settings + recent activity. All state lives on the
// local Tauri window; it pings the dashboard backend for state, config, and
// recent entries.

function museWidget() {
  return {
    museState: "off",
    statusText: "Off — Muse is not watching.",
    expanded: false,
    recentActivity: [],
    cfg: { smart_dedup: true, screenshots: false, debounce_secs: 10, skip_processes: [] },

    async init() {
      await this.refresh();
      // Poll the backend every 5s so the expanded popover stays current.
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
        }
      } catch (_) {}
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
          })
        });
      } catch (_) {}
    },

    openDashboard() {
      // The Tauri webview can navigate the parent webview; for a 320x440
      // floater that means opening in the user's default browser.
      try { window.open("http://127.0.0.1:7137/", "_blank"); } catch (_) {}
    },

    /// Resize the Tauri window so the popover has room. Called from the click
    /// handler on the collapsed pill. Uses the global Tauri API since
    /// `withGlobalTauri = true` in tauri.conf.json.
    expand() {
      this.expanded = true;
      this.resizeTauri(320, 440);
    },

    collapse() {
      this.expanded = false;
      this.resizeTauri(240, 72);
    },

    resizeTauri(w, h) {
      try {
        const t = window.__TAURI_INTERNALS__;
        if (t && t.invoke) {
          t.invoke("plugin:window|resize", { width: w, height: h, logical: true });
        }
      } catch (_) {}
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
