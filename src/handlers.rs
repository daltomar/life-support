use crate::catalog::{date_format_line, release_meta, tracks_json, Catalog};
use crate::cover_colors::CoverPalette;
use askama::Template;
use axum::{extract::{Path, State}, http::StatusCode, response::Html};
use std::sync::Arc;

#[allow(dead_code)]
pub enum NavTab {
    Start,
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
            Self::Start   => "start",
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

pub struct ReleaseRow {
    pub slug: String,
    pub title: String,
    pub catalog_num: String,
    pub meta: String,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub artist_name: String,
    pub artist_slug: String,
    pub has_tracks: bool,
    pub tracks_json: String,
    pub accent: Option<String>,
}

#[derive(Template)]
#[template(path = "releases.html")]
struct ReleasesTemplate {
    active_nav: NavTab,
    releases: Vec<ReleaseRow>,
}

/// Every release, newest first (`Catalog::load` sorts them).
pub async fn releases(State(catalog): State<Arc<Catalog>>) -> Result<Html<String>, StatusCode> {
    let releases = catalog
        .releases
        .iter()
        .map(|r| ReleaseRow {
            slug: r.slug.clone(),
            title: r.title.clone(),
            catalog_num: r.catalog_num.clone(),
            meta: release_meta(&r.date, &r.format, r.tracks.len()),
            description: r
                .description
                .as_deref()
                .map(str::trim)
                .filter(|d| !d.is_empty())
                .map(str::to_string),
            cover: r.cover.as_ref().filter(|s| !s.is_empty()).cloned(),
            artist_name: resolve_artist(&catalog, &r.artist),
            artist_slug: r.artist.clone(),
            has_tracks: !r.tracks.is_empty(),
            tracks_json: tracks_json(&r.tracks),
            accent: r.palette.as_ref().map(|p| p.accent.clone()),
        })
        .collect();

    ReleasesTemplate {
        active_nav: NavTab::Releases,
        releases,
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

// ── Start page ─────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "start.html")]
struct StartTemplate {
    active_nav: NavTab,
}

pub async fn start() -> Result<Html<String>, StatusCode> {
    StartTemplate { active_nav: NavTab::Start }
        .render()
        .map(Html)
        .map_err(|e| {
            eprintln!("template render error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

// ── Listen page ────────────────────────────────────────────────────────────

pub struct ListenTrack {
    pub position: String,
    pub title: String,
    pub file: String,
    pub cover: Option<String>,
    pub artist_name: String,
    pub artist_slug: String,
    pub release_title: String,
    pub release_slug: String,
}

#[derive(Template)]
#[template(path = "listen.html")]
struct ListenTemplate {
    active_nav: NavTab,
    tracks: Vec<ListenTrack>,
}

/// Every track in the catalog: newest release first, album order within each.
pub async fn listen(State(catalog): State<Arc<Catalog>>) -> Result<Html<String>, StatusCode> {
    let mut tracks = Vec::new();
    for r in &catalog.releases {
        let artist_name = resolve_artist(&catalog, &r.artist);
        let cover = r.cover.as_ref().filter(|s| !s.is_empty());
        for t in &r.tracks {
            tracks.push(ListenTrack {
                position: format!("{:02}", tracks.len() + 1),
                title: t.title.clone(),
                file: t.file.clone(),
                cover: cover.cloned(),
                artist_name: artist_name.clone(),
                artist_slug: r.artist.clone(),
                release_title: r.title.clone(),
                release_slug: r.slug.clone(),
            });
        }
    }

    ListenTemplate {
        active_nav: NavTab::Listen,
        tracks,
    }
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
    pub meta: String,
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
            meta: date_format_line(&r.date, &r.format),
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
    meta: String,
    description: String,
    cover: Option<String>,
    artist_name: String,
    artist_slug: String,
    tracks: Vec<ReleaseTrack>,
    tracks_json: String,
    palette: Option<CoverPalette>,
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
        meta: date_format_line(&r.date, &r.format),
        description: r.description.clone().unwrap_or_default(),
        cover: r.cover.as_ref().filter(|s| !s.is_empty()).cloned(),
        artist_name,
        artist_slug: r.artist.clone(),
        tracks,
        tracks_json: tracks_json(&r.tracks),
        palette: r.palette.clone(),
    }
    .render()
    .map(Html)
    .map_err(|e| {
        eprintln!("template render error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Artist, Release, Track};

    fn sample_release(slug: &str, catalog_num: &str, date: &str) -> Release {
        Release {
            slug: slug.to_string(),
            title: format!("Title of {slug}"),
            artist: "someone".to_string(),
            catalog_num: catalog_num.to_string(),
            date: date.to_string(),
            format: "Digital".to_string(),
            description: Some(format!("About {slug}")),
            cover: None,
            accent: None,
            tracks: vec![Track {
                title: "One".to_string(),
                file: "one.mp3".to_string(),
            }],
            palette: None,
        }
    }

    fn catalog(releases: Vec<Release>) -> Catalog {
        Catalog {
            artists: vec![Artist {
                slug: "someone".to_string(),
                name: "Someone".to_string(),
                city: String::new(),
                portrait: None,
                bio: String::new(),
            }],
            releases,
        }
    }

    async fn render(catalog: Catalog) -> String {
        let Html(html) = releases(State(Arc::new(catalog))).await.ok().unwrap();
        html
    }

    #[tokio::test]
    async fn every_release_gets_a_row_in_catalog_order() {
        let html = render(catalog(vec![
            sample_release("new", "LSR002", "2024-08-01"),
            sample_release("old", "LSR001", "2020-08-01"),
        ]))
        .await;

        assert_eq!(html.matches(r#"class="release-row""#).count(), 2);
        let newer = html.find("LSR002").unwrap();
        let older = html.find("LSR001").unwrap();
        assert!(newer < older);
        assert!(html.contains("01 AUG 2024 · Digital · 1 TRACK"));
    }

    #[tokio::test]
    async fn the_latest_release_hero_is_gone() {
        let html = render(catalog(vec![sample_release("new", "LSR002", "2024-08-01")])).await;

        assert!(!html.contains("LATEST RELEASE"));
        assert!(!html.contains("featured-"));
        assert!(!html.contains("btn-play-row"));
    }

    #[tokio::test]
    async fn blank_description_is_left_out() {
        let mut no_description = sample_release("none", "LSR002", "2024-08-01");
        no_description.description = None;
        let mut blank_description = sample_release("blank", "LSR001", "2020-08-01");
        blank_description.description = Some("  ".to_string());

        let html = render(catalog(vec![no_description, blank_description])).await;

        assert!(!html.contains("release-row-desc"));
    }

    #[tokio::test]
    async fn missing_meta_parts_leave_no_stray_separator() {
        let mut bare = sample_release("bare", "LSR001", "");
        bare.format = String::new();
        bare.tracks.clear();

        let html = render(catalog(vec![bare])).await;

        assert!(!html.contains("release-row-meta"));
    }

    #[tokio::test]
    async fn release_without_tracks_gets_a_disabled_play_button() {
        let mut empty = sample_release("empty", "LSR001", "2020-08-01");
        empty.tracks.clear();

        let html = render(catalog(vec![empty])).await;

        assert!(html.contains(r#"<button class="btn-release-action" disabled"#));
        assert!(!html.contains("loadRelease"));
    }

    #[tokio::test]
    async fn unknown_artist_renders_no_artist_line() {
        let mut orphan = sample_release("orphan", "LSR001", "2020-08-01");
        orphan.artist = "nobody".to_string();

        let html = render(catalog(vec![orphan])).await;

        assert!(!html.contains("release-row-artist"));
    }

    fn undated_release() -> Release {
        let mut undated = sample_release("undated", "LSR001", "");
        undated.format = String::new();
        undated
    }

    #[tokio::test]
    async fn release_page_leaves_out_an_empty_date_and_format() {
        let state = State(Arc::new(catalog(vec![undated_release()])));
        let Html(html) = release(state, Path("undated".to_string()))
            .await
            .ok()
            .unwrap();

        assert!(!html.contains("release-detail-meta"));
        assert!(!html.contains(" · "));
    }

    #[tokio::test]
    async fn artist_page_leaves_out_an_empty_date_and_format() {
        let state = State(Arc::new(catalog(vec![undated_release()])));
        let Html(html) = artist(state, Path("someone".to_string()))
            .await
            .ok()
            .unwrap();

        assert!(html.contains("Title of undated"));
        assert!(!html.contains(" · "));
    }

    #[tokio::test]
    async fn release_and_artist_pages_show_date_and_format() {
        let dated = || catalog(vec![sample_release("dated", "LSR001", "2024-08-01")]);
        let line = "01 AUG 2024 · Digital";

        let state = State(Arc::new(dated()));
        let Html(html) = release(state, Path("dated".to_string()))
            .await
            .ok()
            .unwrap();
        assert!(html.contains(line));

        let state = State(Arc::new(dated()));
        let Html(html) = artist(state, Path("someone".to_string()))
            .await
            .ok()
            .unwrap();
        assert!(html.contains(line));
    }
}
