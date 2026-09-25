use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "player.html")]
struct PlayerTemplate<'a> {
    track_title: &'a str,
    track_file: &'a str,
    track_meta: &'a str,
}

pub async fn player() -> Html<String> {
    let t = PlayerTemplate {
        track_title: "Irregular Resignation",
        track_file: "PROHLIS-Disco-01_01-08-2017-Irregular-Resignation.mp3",
        track_meta: "PROHLIS Disco · 01.08.2017",
    };
    Html(t.render().unwrap())
}
