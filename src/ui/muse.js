// Muse floating widget — minimal: shows state, toggle via /api/muse/state.
const __MUSE_BASE__ = (location.protocol === 'https:' ? 'https:' : 'http:') + '//' + (location.hostname || '127.0.0.1') + ':' + (location.port || '7137');

async function api(path, opts = {}) {
  try {
    const res = await fetch(__MUSE_BASE__ + path, opts);
    if (!res.ok) throw new Error('HTTP ' + res.status);
    return res.json();
  } catch (e) {
    console.warn('[Muse] fetch failed', path, e);
    throw e;
  }
}

function museWidget() {
  return {
    museState: 'off',
    busy: false,
    get statusText() {
      if (this.museState === 'watching') return 'Watching…';
      if (this.museState === 'paused') return 'Paused';
      return 'Off';
    },
    async init() {
      try {
        const res = await api('/api/muse/state');
        this.museState = res.state || 'off';
      } catch (e) {
        this.museState = 'off';
      }
    },
    async toggle() {
      if (this.busy) return;
      this.busy = true;
      const next = this.museState === 'watching' ? 'off' : 'watching';
      try {
        const res = await api('/api/muse/state', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ state: next })
        });
        this.museState = res.state || next;
      } catch (e) {
        // best-effort: flip locally
        this.museState = next;
      } finally {
        this.busy = false;
      }
    }
  };
}
window.museWidget = museWidget;
