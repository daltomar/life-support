# Content Guide — Life Support Records

How to add and update content on the site. All catalog data lives in
`catalog.toml`. Media files go in the `media/` directory.

---

## File layout

```
media/
  covers/        album/release cover images
  portraits/     artist portrait images
  *.mp3          audio files (flat, no subdirectories)
catalog.toml     all releases and artists
```

---

## Adding an artist

1. Copy the portrait image to `media/portraits/`.
2. Add a block to `catalog.toml`:

```toml
[[artists]]
slug     = "artist-slug"       # URL-safe: lowercase, hyphens only
name     = "Artist Name"
city     = "City"
portrait = "filename.jpg"      # filename inside media/portraits/; omit if none
bio      = "Short bio text."   # shown on the artist detail page (coming soon)
```

### Artist field reference

| Field      | Required | Notes |
|------------|----------|-------|
| `slug`     | yes      | Used in URLs (`/artists/slug`). Once set, don't change it — it will break existing links. |
| `name`     | yes      | Displayed name. |
| `city`     | yes      | Shown as "CITY · N RELEASES" under the portrait. |
| `portrait` | no       | If omitted or empty, a `[PORTRAIT]` placeholder is shown. |
| `bio`      | no       | Not yet displayed; safe to add now for future use. |

---

## Adding a release

1. Copy the MP3 file(s) to `media/`.
2. Copy the cover image to `media/covers/`.
3. Add a block to `catalog.toml`. The artist must already exist.

```toml
[[releases]]
slug        = "release-slug"       # URL-safe: lowercase, hyphens only
title       = "Release Title"
artist      = "artist-slug"        # must match an existing [[artists]] slug
catalog_num = "LSR002"
date        = "2024-03-15"         # YYYY-MM-DD; releases are ordered newest first
format      = "Digital"            # e.g. Digital, Vinyl, Tape, CD
description = "Optional paragraph shown on the featured and detail pages."
cover       = "filename.jpg"       # filename inside media/covers/; omit if none

  [[releases.tracks]]
  title = "Track One"
  file  = "filename.mp3"           # filename inside media/

  [[releases.tracks]]
  title = "Track Two"
  file  = "another-file.mp3"
```

### Release field reference

| Field         | Required | Notes |
|---------------|----------|-------|
| `slug`        | yes      | Used in URLs (`/releases/slug`). Don't change after publishing. |
| `title`       | yes      | Displayed as a large heading. |
| `artist`      | yes      | Must match a `slug` in `[[artists]]`. |
| `catalog_num` | yes      | e.g. `LSR001`. Shown in the featured release eyebrow. |
| `date`        | yes      | ISO format `YYYY-MM-DD`. Determines display order (newest first). |
| `format`      | yes      | Free text: `Digital`, `Vinyl`, `Tape`, `CD`, etc. |
| `description` | no       | If omitted, the description line is hidden — no placeholder shown. |
| `cover`       | no       | If omitted or empty, a `[COVER]` placeholder is shown. |
| `tracks`      | no       | At least one track is needed for the Play button to work. |

### Track field reference

| Field   | Required | Notes |
|---------|----------|-------|
| `title` | yes      | Track name shown in the player. |
| `file`  | yes      | Filename only — file must be in `media/`. |

---

## Slugs

A slug is the part of a URL that identifies a page: `/artists/prohlis-disco`,
`/releases/disco-01`. Rules:

- Lowercase letters, numbers, and hyphens only — no spaces, accents, or punctuation.
- Set it once. Changing a slug after the page is published breaks any external
  links to it.

Good: `prohlis-disco`, `disco-01`, `lsr001-ep`.
Bad: `PROHLIS Disco`, `disco_01`, `release#2`.

---

## File types and sizes

### Images (covers and portraits)

| Format | Use | Notes |
|--------|-----|-------|
| JPG    | Photos, covers, portraits | Preferred. Export at quality 80–85%. |
| WebP   | Photos | Smaller than JPG at the same quality; works in all modern browsers. |
| PNG    | Logos, graphics with transparency | Produces large files for photos — avoid for covers/portraits. |

Recommended export dimensions:

- **Covers** — square, 1000×1000 px or larger. The site displays them at 320×320 px
  on desktop and full-width on mobile; a 1000 px original keeps them sharp on
  high-DPI screens without wasting bandwidth.
- **Portraits** — 4:5 ratio (e.g. 800×1000 px).

Target file size: **under 300 KB per image**. Above that, page load slows noticeably
on mobile connections.

### Audio

MP3 only. The browser's `<audio>` element supports MP3 universally; WAV and FLAC
do not work reliably in all browsers.

| Bitrate | Use |
|---------|-----|
| 320 kbps | Recommended — standard release quality. |
| 192 kbps | Acceptable for streaming previews. |

No server-side file size limit — the server streams with HTTP byte-range requests,
so even a large file starts playing immediately without buffering the whole file.

Avoid spaces in filenames; use hyphens or underscores instead.

---

## After editing catalog.toml

The catalog is loaded once when the server starts. Restart the server to
pick up changes:

```bash
# development
cargo run

# production (systemd)
sudo systemctl restart life-support
```

---

## What's coming

- **Shell script** — `add-release.sh` to scaffold a new entry interactively.
- **Admin page** — browser-based form for teams with multiple contributors.
