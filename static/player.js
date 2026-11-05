const audio    = document.getElementById('audio');
const btn      = document.getElementById('btn');
const fill     = document.getElementById('fill');
const scrubber = document.getElementById('scrubber');
const time     = document.getElementById('time');

function fmt(s) {
  const m = Math.floor(s / 60);
  const sec = Math.floor(s % 60).toString().padStart(2, '0');
  return m + ':' + sec;
}

btn.addEventListener('click', () => {
  if (audio.paused) {
    audio.play().catch(() => {
      btn.innerHTML = '&#9654;';
      btn.setAttribute('aria-label', 'Play');
    });
    btn.innerHTML = '&#9646;&#9646;';
    btn.setAttribute('aria-label', 'Pause');
  } else {
    audio.pause();
    btn.innerHTML = '&#9654;';
    btn.setAttribute('aria-label', 'Play');
  }
});

audio.addEventListener('timeupdate', () => {
  const pct = audio.duration ? (audio.currentTime / audio.duration) * 100 : 0;
  fill.style.width = pct + '%';
  time.textContent = fmt(audio.currentTime);
});

audio.addEventListener('ended', () => {
  btn.innerHTML = '&#9654;';
  btn.setAttribute('aria-label', 'Play');
  fill.style.width = '0%';
  time.textContent = '0:00';
});

scrubber.addEventListener('click', e => {
  if (!audio.duration) return;
  const r = scrubber.getBoundingClientRect();
  audio.currentTime = ((e.clientX - r.left) / r.width) * audio.duration;
});
