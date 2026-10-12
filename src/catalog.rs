use crate::cover_colors::{self, CoverPalette};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Serialize, Clone)]
pub struct Track {
    pub title: String,
    pub file: String,
}

#[derive(Deserialize, Clone)]
pub struct Release {
    pub slug: String,
    pub title: String,
    pub artist: String,
    pub catalog_num: String,
    pub date: String,
    pub format: String,
    pub description: Option<String>,
    pub cover: Option<String>,
    #[serde(default)]
    pub accent_color: Option<String>,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(skip)]
    pub palette: Option<CoverPalette>,
}

#[derive(Deserialize, Clone)]
pub struct Artist {
    pub slug: String,
    pub name: String,
    pub city: String,
    pub portrait: Option<String>,
    #[serde(default)]
    pub bio: String,
}

#[derive(Deserialize)]
struct CatalogFile {
    #[serde(default)]
    artists: Vec<Artist>,
    #[serde(default)]
    releases: Vec<Release>,
}

#[derive(Clone, Default)]
pub struct Catalog {
    pub artists: Vec<Artist>,
    pub releases: Vec<Release>,
}

impl Catalog {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let mut file: CatalogFile = toml::from_str(&content).map_err(|e| format!("{path}: {e}"))?;
        file.releases.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(Catalog {
            artists: file.artists,
            releases: file.releases,
        })
    }

    /// Populate each release's `palette` field from its cover image.
    /// CPU-bound; call from `spawn_blocking` in main.
    pub fn with_palettes(mut self, media_dir: &str) -> Self {
        let covers_dir = Path::new(media_dir).join("covers");
        for release in &mut self.releases {
            let cover_file = match release.cover.as_deref().filter(|s| !s.is_empty()) {
                Some(f) => f,
                None => continue,
            };
            let path = covers_dir.join(cover_file);

            // Manual override: run contrast-adjust on the provided hex, skip extraction.
            if let Some(ref override_hex) = release.accent_color.clone() {
                let accent = cover_colors::contrast_adjust(override_hex);
                let accent_muted = derive_muted(&accent);
                eprintln!("palette [{}]: accent={accent} (manual override)", release.slug);
                release.palette = Some(CoverPalette { accent, accent_muted });
                continue;
            }

            match cover_colors::extract_palette(&path) {
                Ok(Some(p)) => {
                    eprintln!(
                        "palette [{}]: accent={} muted={}",
                        release.slug, p.accent, p.accent_muted
                    );
                    release.palette = Some(p);
                }
                Ok(None) => {
                    eprintln!("palette [{}]: no chromatic colour found", release.slug);
                }
                Err(e) => {
                    eprintln!("warning: palette extraction failed for '{cover_file}': {e}");
                }
            }
        }
        self
    }

    pub fn artist_by_slug(&self, slug: &str) -> Option<&Artist> {
        self.artists.iter().find(|a| a.slug == slug)
    }

    pub fn release_by_slug(&self, slug: &str) -> Option<&Release> {
        self.releases.iter().find(|r| r.slug == slug)
    }

    pub fn releases_for(&self, artist_slug: &str) -> Vec<&Release> {
        self.releases
            .iter()
            .filter(|r| r.artist == artist_slug)
            .collect()
    }

    pub fn release_count_for(&self, artist_slug: &str) -> usize {
        self.releases_for(artist_slug).len()
    }
}

/// Derive a muted variant from a contrast-adjusted hex accent.
fn derive_muted(hex: &str) -> String {
    use palette::{FromColor, IntoColor, Lab, Srgb};
    let Some([r, g, b]) = parse_hex(hex) else {
        return "#3a3a3a".to_string();
    };
    let srgb: Srgb<f32> = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let mut lab: Lab = srgb.into_color();
    lab.l = (lab.l - 15.0).max(0.0);
    lab.a *= 0.7;
    lab.b *= 0.7;
    let out: Srgb<f32> = Lab::from_color(lab).into_color();
    let clamp = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", clamp(out.red), clamp(out.green), clamp(out.blue))
}

fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 { return None; }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

pub fn format_date(iso: &str) -> String {
    const MONTHS: [&str; 12] = [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ];
    let mut parts = iso.splitn(3, '-');
    let Some(year) = parts.next() else {
        return iso.to_string();
    };
    let month = parts
        .next()
        .and_then(|m| m.parse::<usize>().ok())
        .and_then(|m| MONTHS.get(m.wrapping_sub(1)))
        .copied();
    let day = parts.next().unwrap_or("");
    match month {
        Some(m) => format!("{day} {m} {year}"),
        None => {
            eprintln!("warning: unrecognised date format '{iso}' in catalog");
            iso.to_string()
        }
    }
}

pub fn tracks_json(tracks: &[Track]) -> String {
    serde_json::to_string(tracks).unwrap_or_else(|_| "[]".to_string())
}
