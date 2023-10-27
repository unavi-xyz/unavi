mod fileserve;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use fileserve::file_and_error_handler;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use unavi_web_app::*;

use self::fileserve::assets_file_handler;

pub async fn router() -> Router {
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    let router: Router = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/assets/*path", get(assets_file_handler))
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    router
}
