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

function togglePlayback() {
  if (queueIndex < 0) {
    playRandomTrack();
  } else if (audio.error) {
    playQueueEntry(queueIndex);
  } else if (cancelBufferWait) {
    cancelBufferWait();
  } else if (audio.paused) {
    playWhenBuffered();
  } else {
    audio.pause();
  }
}

if (playBtn) playBtn.addEventListener('click', togglePlayback);

// True from the moment playback is asked for, including the start buffer wait.
function isPlayingOrStarting() {
  return cancelBufferWait !== null || !audio.paused;
}

// Sets both the sidebar and the mobile docked button. Waiting for the start
// buffer counts as playing: pressing the button then cancels the wait.
function syncPlayButtons() {
  const buffering = cancelBufferWait !== null;
  const playing = isPlayingOrStarting();
  if (mediaSession) mediaSession.playbackState = playing ? 'playing' : 'paused';
  const icon = playing
    ? '<rect x="0" y="0" width="4" height="14"/><rect x="8" y="0" width="4" height="14"/>'
    : '<polygon points="0,0 12,7 0,14"/>';
  document.querySelectorAll('.player-btn').forEach(btn => {
    btn.setAttribute('aria-label', playing ? 'Pause' : 'Play');
    btn.setAttribute('aria-busy', String(buffering));
    btn.innerHTML = '<svg width="12" height="14" viewBox="0 0 12 14" fill="currentColor" aria-hidden="true">' + icon + '</svg>';
  });
}

audio.addEventListener('play', syncPlayButtons);
audio.addEventListener('pause', syncPlayButtons);

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
  const wasPlaying = isPlayingOrStarting();
  if (cancelBufferWait) cancelBufferWait();
  resetProgress();
  if (queueIndex + 1 < queue.length) {
    playQueueEntry(queueIndex + 1, wasPlaying);
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

// ── Volume ───────────────────────────────────────────────
// Sidebar player only: the mobile docked player has no volume controls.
const muteBtn      = document.getElementById('player-mute');
const volumeSlider = document.getElementById('player-volume-slider');

const VOLUME_KEY = 'player-volume';
const MUTED_KEY  = 'player-muted';

const SPEAKER_SHAPE = '<polygon points="0,4.5 3,4.5 7,1 7,13 3,9.5 0,9.5"/>';
const VOLUME_ICON = SPEAKER_SHAPE
  + '<rect x="9" y="5" width="1.5" height="4"/><rect x="12" y="3" width="1.5" height="8"/>';
const MUTED_ICON = SPEAKER_SHAPE
  + '<rect x="8.75" y="6.25" width="5.5" height="1.5" transform="rotate(45 11.5 7)"/>'
  + '<rect x="8.75" y="6.25" width="5.5" height="1.5" transform="rotate(-45 11.5 7)"/>';

// Storage can be blocked, and even touching localStorage can then throw.
function readStored(key) {
  try {
    return localStorage.getItem(key);
  } catch (_) {
    return null;
  }
}

function writeStored(key, value) {
  try {
    localStorage.setItem(key, value);
  } catch (_) {
    // The setting just won't survive a reload.
  }
}

// iOS leaves audio.volume at 1 whatever it is set to.
function canSetVolume() {
  const before = audio.volume;
  audio.volume = 0.5;
  const settable = audio.volume === 0.5;
  audio.volume = before;
  return settable;
}

function restoreVolume() {
  const level = parseFloat(readStored(VOLUME_KEY));
  if (level >= 0 && level <= 1) audio.volume = level;
  audio.muted = readStored(MUTED_KEY) === 'true';
}

// The slider is perceptual: volume = position squared.
function syncVolumeControls() {
  if (volumeSlider) {
    const pos = Math.sqrt(audio.volume);
    volumeSlider.value = pos;
    volumeSlider.style.setProperty('--volume-pos', (pos * 100) + '%');
  }
  if (muteBtn) {
    muteBtn.setAttribute('aria-label', audio.muted ? 'Unmute' : 'Mute');
    muteBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" aria-hidden="true">'
      + (audio.muted ? MUTED_ICON : VOLUME_ICON) + '</svg>';
  }
}

if (volumeSlider) {
  volumeSlider.hidden = !canSetVolume();
  volumeSlider.addEventListener('input', () => {
    const pos = Number(volumeSlider.value);
    audio.volume = pos * pos;
    audio.muted = false;
  });
}

if (muteBtn) muteBtn.addEventListener('click', () => { audio.muted = !audio.muted; });

audio.addEventListener('volumechange', () => {
  syncVolumeControls();
  writeStored(VOLUME_KEY, audio.volume);
  writeStored(MUTED_KEY, audio.muted);
});

restoreVolume();
// volumechange doesn't fire when the restored values match the defaults.
syncVolumeControls();

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

// Setting src replaces any load still in progress, so switching tracks quickly
// leaves only the last choice. With autoplay off the track loads paused.
function playQueueEntry(index, autoplay = true) {
  const t = queue[index];
  queueIndex = index;
  if (cancelBufferWait) cancelBufferWait();
  resetProgress();
  if (durEl) durEl.textContent = '0:00';
  audio.src = '/media/' + encodeURIComponent(t.file);
  updatePlayerInfo(t.title, t.artist, t.cover);
  updateTrackControls();
  updateMediaMetadata(t);
  if (!autoplay) return;
  // Some browsers only allow play() during a click, and the buffer wait below
  // outlasts it. Starting and stopping here keeps the element allowed to play.
  audio.play().catch(() => {});
  audio.pause();
  playWhenBuffered();
}

// ── Previous / next ──────────────────────────────────────
// Previous restarts the track once it has played for this long.
const RESTART_THRESHOLD_SECONDS = 3;

function hasNextTrack() {
  return queueIndex >= 0 && queueIndex + 1 < queue.length;
}

function playNextTrack() {
  if (hasNextTrack()) playQueueEntry(queueIndex + 1, isPlayingOrStarting());
}

function playPreviousTrack() {
  if (queueIndex < 0) return;
  if (queueIndex === 0 || audio.currentTime > RESTART_THRESHOLD_SECONDS) {
    audio.currentTime = 0;
  } else {
    playQueueEntry(queueIndex - 1, isPlayingOrStarting());
  }
}

function setSkipButtonEnabled(btn, enabled) {
  // A disabled button drops keyboard focus, so hand it to the play button.
  if (!enabled && document.activeElement === btn) {
    btn.parentElement.querySelector('.player-btn').focus();
  }
  btn.disabled = !enabled;
}

function updateTrackControls() {
  const hasTrack = queueIndex >= 0;
  document.querySelectorAll('.player-prev').forEach(btn => setSkipButtonEnabled(btn, hasTrack));
  document.querySelectorAll('.player-next').forEach(btn => setSkipButtonEnabled(btn, hasNextTrack()));
  if (mediaSession) {
    setMediaAction('previoustrack', hasTrack ? playPreviousTrack : null);
    setMediaAction('nexttrack', hasNextTrack() ? playNextTrack : null);
  }
}

document.querySelectorAll('.player-prev').forEach(btn => btn.addEventListener('click', playPreviousTrack));
document.querySelectorAll('.player-next').forEach(btn => btn.addEventListener('click', playNextTrack));

// ── Media Session (media keys, lock screen, headphones) ──
// Only available on secure origins: https, or localhost in development.
const mediaSession = 'mediaSession' in navigator ? navigator.mediaSession : null;

// Browsers may reject an action they don't support. These are optional extras,
// so a failure here must not stop playback or the rest of this script.
function setMediaAction(action, handler) {
  try {
    mediaSession.setActionHandler(action, handler);
  } catch (e) {
    console.warn('Media action not supported:', action);
  }
}

function updateMediaMetadata(t) {
  if (!mediaSession) return;
  try {
    mediaSession.metadata = new MediaMetadata({
      title: t.title,
      artist: t.artist,
      artwork: t.cover ? [{ src: t.cover }] : [],
    });
  } catch (e) {
    console.warn('Could not set media metadata', e);
  }
}

if (mediaSession) {
  // Through togglePlayback so a key press also respects the start buffer wait.
  setMediaAction('play', () => {
    if (!isPlayingOrStarting()) togglePlayback();
  });
  setMediaAction('pause', () => {
    if (isPlayingOrStarting()) togglePlayback();
  });
}

// ── Start buffer ─────────────────────────────────────────
// Browsers start playing with very little downloaded, which stutters when the
// connection only just keeps up. Hold the start until a few seconds are ready.
const START_BUFFER_SECONDS = 3;
const START_BUFFER_TIMEOUT_MS = 5000;
const START_BUFFER_POLL_MS = 100;
// A buffered range can stop just short of the last moment of a track.
const BUFFER_TOLERANCE_SECONDS = 0.1;
let cancelBufferWait = null;

function hasStartBuffer() {
  const needed = Math.min(START_BUFFER_SECONDS, audio.duration - audio.currentTime);
  if (isNaN(needed)) return false;
  for (let i = 0; i < audio.buffered.length; i++) {
    if (audio.buffered.start(i) <= audio.currentTime + BUFFER_TOLERANCE_SECONDS
        && audio.buffered.end(i) >= audio.currentTime + needed - BUFFER_TOLERANCE_SECONDS) {
      return true;
    }
  }
  return false;
}

function playWhenBuffered() {
  if (audio.ended) audio.currentTime = 0;
  if (hasStartBuffer()) {
    audio.play().catch(syncPlayButtons);
    return;
  }
  const startedAt = performance.now();
  const timer = setInterval(() => {
    if (hasStartBuffer() || performance.now() - startedAt >= START_BUFFER_TIMEOUT_MS) {
      audio.play().catch(syncPlayButtons);
      cancelBufferWait();
    }
  }, START_BUFFER_POLL_MS);
  cancelBufferWait = () => {
    clearInterval(timer);
    cancelBufferWait = null;
    syncPlayButtons();
  };
  syncPlayButtons();
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
