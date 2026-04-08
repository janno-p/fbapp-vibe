function countdown(kickoffMs) {
  function label() {
    const diff = kickoffMs - Date.now();
    if (diff <= 0) return 'In progress';
    const h = Math.floor(diff / 3600000);
    const m = Math.floor((diff % 3600000) / 60000);
    const s = Math.floor((diff % 60000) / 1000);
    return h > 0 ? `Kicks off in ${h}h ${m}m` : `Kicks off in ${m}m ${s}s`;
  }
  return {
    label: label(),
    init() { setInterval(() => { this.label = label(); }, 1000); },
  };
}
