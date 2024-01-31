use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

pub async fn router() -> Router {
    let serve_dir = ServeDir::new("crates/unavi-app/dist")
        .not_found_service(ServeFile::new("crates/unavi-app/dist/index.html"));
    Router::new().fallback_service(serve_dir)
}
