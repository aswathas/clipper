// Clipper phone PWA — posts a URL to the desktop's /save endpoint.
//
// No build step. Alpine drives the page when the CDN is reachable; if it is
// not (the phone may be on a LAN with no internet), the plain-DOM fallback
// below wires the same form so saving still works.

(function () {
  const TOKEN =
    window.CLIPPER_TOKEN ||
    new URLSearchParams(location.search).get("token") ||
    "";

  const endpoint = (token) => "/save?token=" + encodeURIComponent(token || "");

  async function postClip(url, token) {
    const trimmed = (url || "").trim();
    if (!trimmed) return { ok: false, message: "Paste a link first." };

    try {
      const response = await fetch(endpoint(token), {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Clipper-Token": token || "",
        },
        body: JSON.stringify({ url: trimmed }),
      });

      let payload = {};
      try {
        payload = await response.json();
      } catch (_) {
        /* non-JSON error body */
      }

      if (!response.ok) {
        return {
          ok: false,
          message: payload.error || "Could not save (" + response.status + ").",
        };
      }
      return {
        ok: true,
        message: "Saved. It will be summarised shortly.",
        id: payload.id,
      };
    } catch (error) {
      return { ok: false, message: "Network error: " + error.message };
    }
  }

  function setStatus(text, ok) {
    const el = document.getElementById("status");
    if (!el) return;
    el.textContent = text;
    el.classList.toggle("ok", ok === true);
    el.classList.toggle("err", ok === false);
  }

  // Used only when Alpine never loaded.
  function wirePlainDom() {
    const form = document.getElementById("save-form");
    if (!form) return;
    let busy = false;

    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      if (busy) return;
      busy = true;

      const input = document.getElementById("url");
      const button = document.getElementById("save-button");
      if (button) button.disabled = true;
      setStatus("Saving…", null);

      const result = await postClip(input ? input.value : "", TOKEN);
      setStatus(result.message, result.ok);
      if (result.ok && input) input.value = "";

      if (button) button.disabled = false;
      busy = false;
    });
  }

  document.addEventListener("alpine:init", () => {
    window.Alpine.data("clipperPair", () => ({
      url: "",
      status: "Ready.",
      busy: false,

      async save() {
        this.busy = true;
        this.status = "Saving…";
        const result = await postClip(this.url, TOKEN);
        this.status = result.message;
        if (result.ok) this.url = "";
        this.busy = false;
      },
    }));
  });

  window.addEventListener("load", () => {
    if (!window.Alpine) wirePlainDom();
  });
})();
