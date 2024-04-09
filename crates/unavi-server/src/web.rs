use axum::{routing::get_service, Router};
use tower_http::services::{ServeDir, ServeFile};

pub fn router() -> Router {
    let mut path = std::env::current_dir().unwrap();
    path.push("web");

    let mut index_path = path.clone();
    index_path.push("index.html");

    Router::new()
        .route("/", get_service(ServeFile::new(index_path)))
        .fallback_service(ServeDir::new(path))
}
