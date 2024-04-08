use axum::Router;
use tower_http::services::ServeDir;

pub fn router() -> Router {
    let mut path = std::env::current_dir().unwrap();
    path.push("web");

    Router::new().nest_service("/", ServeDir::new(path))
}
