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
    if (queueIndex < 0) {
      playRandomTrack();
    } else if (audio.error) {
      playQueueEntry(queueIndex);
    } else if (audio.paused) {
      audio.play().catch(() => {});
    } else {
      audio.pause();
    }
  });
}

// Sets both the sidebar and the mobile docked button.
function setPlayButtons(label, icon) {
  document.querySelectorAll('.player-btn').forEach(btn => {
    btn.setAttribute('aria-label', label);
    btn.innerHTML = '<svg width="12" height="14" viewBox="0 0 12 14" fill="currentColor" aria-hidden="true">' + icon + '</svg>';
  });
}

audio.addEventListener('play', () => {
  setPlayButtons('Pause', '<rect x="0" y="0" width="4" height="14"/><rect x="8" y="0" width="4" height="14"/>');
});

audio.addEventListener('pause', () => {
  setPlayButtons('Play', '<polygon points="0,0 12,7 0,14"/>');
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

function resetProgress() {
  if (fill) fill.style.width = '0%';
  if (dockedProgress) dockedProgress.style.width = '0%';
  if (timeEl) timeEl.textContent = '0:00';
}

audio.addEventListener('ended', () => {
  resetProgress();
  if (queueIndex + 1 < queue.length) playQueueEntry(queueIndex + 1);
});

// A track that fails to load is skipped. At the end of the queue the element
// stays "playing" after an error, so pause it to put the buttons back on Play.
audio.addEventListener('error', () => {
  console.warn('Could not play', audio.currentSrc);
  resetProgress();
  if (queueIndex + 1 < queue.length) {
    playQueueEntry(queueIndex + 1);
  } else {
    audio.pause();
  }
});

if (track) {
  track.addEventListener('click', e => {
    if (!audio.duration) return;
    const r = track.getBoundingClientRect();
    audio.currentTime = ((e.clientX - r.left) / r.width) * audio.duration;
  });
}

// ── Player info update ───────────────────────────────────
function updatePlayerInfo(title, artist, cover) {
  document.querySelectorAll('.player-title').forEach(el => el.textContent = title);
  document.querySelectorAll('.player-artist').forEach(el => el.textContent = artist);
  document.querySelectorAll('.player-cover').forEach(el => {
    if (cover) {
      el.innerHTML = '<img src="' + cover + '" alt="">';
    } else {
      el.innerHTML = '';
    }
  });
}

// ── Playback queue ───────────────────────────────────────
// Kept here rather than read from the page, so playback continues through the
// list after the user navigates away from it.
let queue = [];
let queueIndex = -1;

function playQueueEntry(index) {
  const t = queue[index];
  queueIndex = index;
  audio.src = '/media/' + encodeURIComponent(t.file);
  audio.play().catch(() => {});
  updatePlayerInfo(t.title, t.artist, t.cover);
}

// Play a whole release from its first track.
window.loadRelease = function(btn) {
  const json = btn.dataset.tracks;
  if (!json) return;
  let tracks;
  try { tracks = JSON.parse(json); } catch(_) { return; }
  if (!tracks.length) return;
  const artist = btn.dataset.artist || '';
  const cover  = btn.dataset.cover  || '';
  queue = tracks.map(t => ({ file: t.file, title: t.title, artist, cover }));
  playQueueEntry(0);
};

// Play one track, then continue through the rest of the list it appears in.
window.loadTrack = function(btn) {
  if (!btn.dataset.file) return;
  const buttons = Array.from(btn.closest('main').querySelectorAll('button[data-file]'));
  queue = buttons.map(trackFromButton);
  playQueueEntry(buttons.indexOf(btn));
};

function trackFromButton(b) {
  return {
    file:   b.dataset.file,
    title:  b.dataset.title  || '',
    artist: b.dataset.artist || '',
    cover:  b.dataset.cover  || '',
  };
}

// ── Random start ─────────────────────────────────────────
// The Listen page is the list of every track. It is fetched ahead of time so
// that pressing play can start audio straight away, inside the user's click.
let allTracks = [];

async function fetchAllTracks() {
  try {
    const res = await fetch('/listen');
    if (!res.ok) throw new Error('HTTP ' + res.status);
    const doc = new DOMParser().parseFromString(await res.text(), 'text/html');
    allTracks = Array.from(doc.querySelectorAll('main button[data-file]')).map(trackFromButton);
  } catch (e) {
    console.warn('Could not load the track list', e);
  }
}

fetchAllTracks();

async function playRandomTrack() {
  if (!allTracks.length) await fetchAllTracks();
  // Something else may have started while the list was loading.
  if (queueIndex >= 0 || !allTracks.length) return;
  queue = allTracks;
  playQueueEntry(Math.floor(Math.random() * queue.length));
}

// ── Nav swap (keep audio playing across pages) ──────────
// Takes the highlight from the fetched page, because a detail page such as
// /releases/x highlights a menu item whose URL is not its own.
function setActiveNav(doc) {
  const current = doc.querySelector('nav a[aria-current="page"]');
  const path = current ? current.getAttribute('href') : null;
  document.querySelectorAll('nav a').forEach(a => {
    if (a.getAttribute('href') === path) {
      a.setAttribute('aria-current', 'page');
    } else {
      a.removeAttribute('aria-current');
    }
  });
}

async function navigate(url, { fromHistory = false } = {}) {
  try {
    const res = await fetch(url, { headers: { 'X-Nav-Swap': '1' } });
    if (!res.ok) throw new Error('HTTP ' + res.status);
    const html = await res.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    const newMain = doc.querySelector('main');
    if (!newMain) throw new Error('response has no <main>');
    document.querySelector('main').replaceWith(newMain);
    document.title = doc.title;
    // Back/Forward has already moved the history; only a new visit adds an entry.
    if (!fromHistory && url !== location.href) history.pushState(null, '', url);
    // On mobile the page scrolls on <body>; on desktop <main> is new, so already at the top.
    if (!fromHistory) document.body.scrollTop = 0;
    setActiveNav(doc);
  } catch (_) {
    if (fromHistory) {
      location.replace(url);
    } else {
      location.href = url;
    }
  }
}

// Whether a click on this link should swap <main> rather than leave the page.
// Anything asking for a new tab or a file keeps the browser's own behaviour.
function isPageLink(a, e) {
  if (e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return false;
  if (a.target || a.hasAttribute('download') || a.origin !== location.origin) return false;
  return !a.pathname.startsWith('/media/') && !a.pathname.startsWith('/static/');
}

document.addEventListener('click', e => {
  const a = e.target.closest('a[href]');
  if (!a || !isPageLink(a, e)) return;
  e.preventDefault();
  closeMenu();
  navigate(a.href);
});

window.addEventListener('popstate', () => {
  navigate(location.href, { fromHistory: true });
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

function closeMenu() {
  if (nav) nav.classList.remove('open');
  if (menuBtn) menuBtn.setAttribute('aria-expanded', 'false');
}

if (navClose) navClose.addEventListener('click', closeMenu);
