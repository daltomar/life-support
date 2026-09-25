use askama::Template;
use axum::{http::StatusCode, response::Html};

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

#[derive(Template)]
#[template(path = "releases.html")]
struct ReleasesTemplate {
    active_nav: NavTab,
}

pub async fn releases() -> Result<Html<String>, StatusCode> {
    let t = ReleasesTemplate {
        active_nav: NavTab::Releases,
    };
    t.render()
        .map(Html)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
