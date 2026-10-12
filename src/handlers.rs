use crate::catalog::{format_date, tracks_json, Catalog};
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::Html};
use std::sync::Arc;

#[allow(dead_code)]
pub enum NavTab {
    Zero,
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
            Self::Zero    => "zero",
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
    pub has_tracks: bool,
    pub tracks_json: String,
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
                has_tracks: !r.tracks.is_empty(),
                tracks_json: tracks_json(&r.tracks),
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

// ── Zero page ──────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "zero.html")]
struct ZeroTemplate {
    active_nav: NavTab,
}

pub async fn zero() -> Result<Html<String>, StatusCode> {
    ZeroTemplate { active_nav: NavTab::Zero }
        .render()
        .map(Html)
        .map_err(|e| {
            eprintln!("template render error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ── Artist detail page ─────────────────────────────────────

pub struct ArtistRelease {
    pub slug: String,
    pub title: String,
    pub catalog_num: String,
    pub date: String,
    pub format: String,
    pub cover: Option<String>,
    pub has_tracks: bool,
    pub tracks_json: String,
}

#[derive(Template)]
#[template(path = "artist.html")]
struct ArtistTemplate {
    active_nav: NavTab,
    name: String,
    city: String,
    portrait: Option<String>,
    bio: String,
    releases: Vec<ArtistRelease>,
}

pub async fn artist(
    State(catalog): State<Arc<Catalog>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let a = catalog.artist_by_slug(&slug).ok_or(StatusCode::NOT_FOUND)?;
    let releases = catalog
        .releases_for(&slug)
        .into_iter()
        .map(|r| ArtistRelease {
            slug: r.slug.clone(),
            title: r.title.clone(),
            catalog_num: r.catalog_num.clone(),
            date: format_date(&r.date),
            format: r.format.clone(),
            cover: r.cover.as_ref().filter(|s| !s.is_empty()).cloned(),
            has_tracks: !r.tracks.is_empty(),
            tracks_json: tracks_json(&r.tracks),
        })
        .collect();

    ArtistTemplate {
        active_nav: NavTab::Artists,
        name: a.name.clone(),
        city: a.city.clone(),
        portrait: a.portrait.as_ref().filter(|s| !s.is_empty()).cloned(),
        bio: a.bio.clone(),
        releases,
    }
    .render()
    .map(Html)
    .map_err(|e| {
        eprintln!("template render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// ── Release detail page ────────────────────────────────────

pub struct ReleaseTrack {
    pub number: usize,
    pub title: String,
    pub file: String,
}

#[derive(Template)]
#[template(path = "release.html")]
struct ReleaseTemplate {
    active_nav: NavTab,
    title: String,
    catalog_num: String,
    date: String,
    format: String,
    description: String,
    cover: Option<String>,
    artist_name: String,
    artist_slug: String,
    tracks: Vec<ReleaseTrack>,
    tracks_json: String,
}

pub async fn release(
    State(catalog): State<Arc<Catalog>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let r = catalog.release_by_slug(&slug).ok_or(StatusCode::NOT_FOUND)?;
    let artist_name = resolve_artist(&catalog, &r.artist);
    let tracks = r
        .tracks
        .iter()
        .enumerate()
        .map(|(i, t)| ReleaseTrack {
            number: i + 1,
            title: t.title.clone(),
            file: t.file.clone(),
        })
        .collect();

    ReleaseTemplate {
        active_nav: NavTab::Releases,
        title: r.title.clone(),
        catalog_num: r.catalog_num.clone(),
        date: format_date(&r.date),
        format: r.format.clone(),
        description: r.description.clone().unwrap_or_default(),
        cover: r.cover.as_ref().filter(|s| !s.is_empty()).cloned(),
        artist_name,
        artist_slug: r.artist.clone(),
        tracks,
        tracks_json: tracks_json(&r.tracks),
    }
    .render()
    .map(Html)
    .map_err(|e| {
        eprintln!("template render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
