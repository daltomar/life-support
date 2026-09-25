// ── Player ──────────────────────────────────────────────
const audio    = document.getElementById('audio');
const playBtn  = document.getElementById('play-btn');
const fill     = document.getElementById('player-fill');
const track    = document.getElementById('player-track');
const timeEl   = document.getElementById('player-time');
const durEl    = document.getElementById('player-dur');
const dockedProgress = document.getElementById('docked-progress');

function fmt(s) {
  if (!s || isNaN(s)) return '0:00';
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60).toString().padStart(2, '0');
  return m + ':' + sec;
}

if (playBtn) {
  playBtn.addEventListener('click', () => {
    if (audio.paused) {
      audio.play().catch(() => {});
    } else {
      audio.pause();
    }
  });
}

audio.addEventListener('play', () => {
  if (playBtn) {
    playBtn.setAttribute('aria-label', 'Pause');
    playBtn.innerHTML = '<svg width="12" height="14" viewBox="0 0 12 14" fill="currentColor"><rect x="0" y="0" width="4" height="14"/><rect x="8" y="0" width="4" height="14"/></svg>';
  }
});

audio.addEventListener('pause', () => {
  if (playBtn) {
    playBtn.setAttribute('aria-label', 'Play');
    playBtn.innerHTML = '<svg width="12" height="14" viewBox="0 0 12 14" fill="currentColor"><polygon points="0,0 12,7 0,14"/></svg>';
  }
});

audio.addEventListener('timeupdate', () => {
  const pct = audio.duration ? (audio.currentTime / audio.duration) * 100 : 0;
  if (fill) fill.style.width = pct + '%';
  if (dockedProgress) dockedProgress.style.width = pct + '%';
  if (timeEl) timeEl.textContent = fmt(audio.currentTime);
});

audio.addEventListener('loadedmetadata', () => {
  if (durEl) durEl.textContent = fmt(audio.duration);
});

audio.addEventListener('ended', () => {
  if (fill) fill.style.width = '0%';
  if (dockedProgress) dockedProgress.style.width = '0%';
  if (timeEl) timeEl.textContent = '0:00';
});

if (track) {
  track.addEventListener('click', e => {
    if (!audio.duration) return;
    const r = track.getBoundingClientRect();
    audio.currentTime = ((e.clientX - r.left) / r.width) * audio.duration;
  });
}

// ── Nav swap (keep audio playing across pages) ──────────
function setActiveNav(url) {
  const path = new URL(url, location.href).pathname;
  document.querySelectorAll('nav a').forEach(a => {
    const match = new URL(a.href, location.href).pathname === path;
    if (match) {
      a.setAttribute('aria-current', 'page');
    } else {
      a.removeAttribute('aria-current');
    }
  });
}

async function navigate(url) {
  try {
    const res = await fetch(url, { headers: { 'X-Nav-Swap': '1' } });
    const html = await res.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    const newMain = doc.querySelector('main');
    if (newMain) {
      document.querySelector('main').replaceWith(newMain);
    }
    history.pushState(null, '', url);
    setActiveNav(url);
  } catch (_) {
    location.href = url;
  }
}

document.addEventListener('click', e => {
  const a = e.target.closest('nav a');
  if (!a) return;
  e.preventDefault();
  navigate(a.href);
});

window.addEventListener('popstate', () => {
  navigate(location.href);
});

// ── Mobile menu ──────────────────────────────────────────
const menuBtn  = document.getElementById('menu-btn');
const navClose = document.getElementById('nav-close');
const nav      = document.querySelector('nav');

if (menuBtn && nav) {
  menuBtn.addEventListener('click', () => {
    nav.classList.add('open');
    menuBtn.setAttribute('aria-expanded', 'true');
  });
}

if (navClose && nav) {
  navClose.addEventListener('click', () => {
    nav.classList.remove('open');
    if (menuBtn) menuBtn.setAttribute('aria-expanded', 'false');
  });
}
