use crate::catalog::{format_date, tracks_json, Catalog};
use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};
use std::sync::Arc;

#[allow(dead_code)]
pub enum NavTab {
    Releases,
    Artists,
    Listen,
    News,
    About,
    Contact,
}

impl NavTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Releases => "releases",
            Self::Artists => "artists",
            Self::Listen => "listen",
            Self::News => "news",
            Self::About => "about",
            Self::Contact => "contact",
        }
    }
}

// ── Releases page ──────────────────────────────────────────────────────────

pub struct FeaturedView {
    pub slug: String,
    pub title: String,
    pub catalog_num: String,
    pub date: String,
    pub format: String,
    pub track_count: usize,
    pub description: String,
    pub cover: Option<String>,
    pub artist_name: String,
    pub artist_slug: String,
    pub tracks_json: String,
}

pub struct CatalogRow {
    pub position: String,
    pub slug: String,
    pub title: String,
    pub cover: Option<String>,
    pub date: String,
    pub format: String,
    pub artist_name: String,
    pub artist_slug: String,
}

#[derive(Template)]
#[template(path = "releases.html")]
struct ReleasesTemplate {
    active_nav: NavTab,
    featured: Option<FeaturedView>,
    catalog: Vec<CatalogRow>,
}

pub async fn releases(State(catalog): State<Arc<Catalog>>) -> Result<Html<String>, StatusCode> {
    let mut releases = catalog.releases.iter();

    let featured = releases.next().map(|r| {
        let artist_name = resolve_artist(&catalog, &r.artist);
        FeaturedView {
            slug: r.slug.clone(),
            title: r.title.clone(),
            catalog_num: r.catalog_num.clone(),
            date: format_date(&r.date),
            format: r.format.clone(),
            track_count: r.tracks.len(),
            description: r.description.clone().unwrap_or_default(),
            cover: r.cover.as_ref().filter(|s| !s.is_empty()).cloned(),
            artist_name,
            artist_slug: r.artist.clone(),
            tracks_json: tracks_json(&r.tracks),
        }
    });

    let catalog_rows: Vec<CatalogRow> = releases
        .enumerate()
        .map(|(i, r)| {
            let artist_name = resolve_artist(&catalog, &r.artist);
            CatalogRow {
                position: format!("{:02}", i + 2),
                slug: r.slug.clone(),
                title: r.title.clone(),
                cover: r.cover.as_ref().filter(|s| !s.is_empty()).cloned(),
                date: format_date(&r.date),
                format: r.format.clone(),
                artist_name,
                artist_slug: r.artist.clone(),
            }
        })
        .collect();

    ReleasesTemplate {
        active_nav: NavTab::Releases,
        featured,
        catalog: catalog_rows,
    }
    .render()
    .map(Html)
    .map_err(|e| {
        eprintln!("template render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// ── Artists page ───────────────────────────────────────────────────────────

pub struct ArtistView {
    pub slug: String,
    pub name: String,
    pub city: String,
    pub portrait: Option<String>,
    pub position: String,
    pub releases_label: String,
}

#[derive(Template)]
#[template(path = "artists.html")]
struct ArtistsTemplate {
    active_nav: NavTab,
    artists: Vec<ArtistView>,
    grid_cols: usize,
}

pub async fn artists(State(catalog): State<Arc<Catalog>>) -> Result<Html<String>, StatusCode> {
    let artists: Vec<ArtistView> = catalog
        .artists
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let count = catalog.release_count_for(&a.slug);
            let releases_label = if count == 1 {
                "1 RELEASE".to_string()
            } else {
                format!("{count} RELEASES")
            };
            ArtistView {
                slug: a.slug.clone(),
                name: a.name.clone(),
                city: a.city.clone(),
                portrait: a.portrait.as_ref().filter(|s| !s.is_empty()).cloned(),
                position: format!("{:02}", i + 1),
                releases_label,
            }
        })
        .collect();

    let grid_cols = artists.len().clamp(1, 3);

    ArtistsTemplate {
        active_nav: NavTab::Artists,
        artists,
        grid_cols,
    }
    .render()
    .map(Html)
    .map_err(|e| {
        eprintln!("template render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

fn resolve_artist(catalog: &Catalog, slug: &str) -> String {
    match catalog.artist_by_slug(slug) {
        Some(a) => a.name.clone(),
        None => {
            eprintln!("warning: no artist found for slug '{slug}'");
            String::new()
        }
    }
}
