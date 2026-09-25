use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    pub tracks: Vec<Track>,
}

#[derive(Deserialize, Clone)]
pub struct Artist {
    pub slug: String,
    pub name: String,
    pub city: String,
    pub portrait: Option<String>,
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
    pub fn load(path: &str) -> Result<Arc<Self>, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let mut file: CatalogFile = toml::from_str(&content).map_err(|e| format!("{path}: {e}"))?;
        file.releases.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(Arc::new(Catalog {
            artists: file.artists,
            releases: file.releases,
        }))
    }

    pub fn artist_by_slug(&self, slug: &str) -> Option<&Artist> {
        self.artists.iter().find(|a| a.slug == slug)
    }

    pub fn release_count_for(&self, artist_slug: &str) -> usize {
        self.releases
            .iter()
            .filter(|r| r.artist == artist_slug)
            .count()
    }
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
