use axum::response::Html;

pub async fn index() -> Html<&'static str> {
  Html(std::include_str!("../../index.html"))
}

pub mod chat;
