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

## Naming conventions

**Slugs** — lowercase letters, numbers, and hyphens only. No spaces or special characters.
Good: `prohlis-disco`, `disco-01`. Bad: `PROHLIS Disco`, `disco_01`.

**Image files** — any format the browser supports (JPG, PNG, WebP).
Recommended sizes: covers **square** (e.g. 1000×1000 px), portraits **4:5 ratio**
(e.g. 800×1000 px). Larger originals are fine; the browser scales them.

**Audio files** — MP3. Filenames can be anything but avoid spaces
(use hyphens or underscores instead).

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
