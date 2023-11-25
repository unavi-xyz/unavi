use axum::response::Response as AxumResponse;
use axum::{
    body::{boxed, Body, BoxBody},
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
};
use leptos::*;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use unavi_web_common::error_template::{AppError, ErrorTemplate};

pub async fn assets_file_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    // Remove "/assets" from the path
    let mut path = uri.path().to_string();
    path = path.replace("/assets", "");
    let uri = Uri::builder().path_and_query(path).build().unwrap();

    // Set root to the assets folder
    let root = options.site_root.clone();
    let options = LeptosOptions {
        site_root: root + "/../../assets",
        ..options
    };

    file_and_error_handler(uri, State(options), req).await
}

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let mut errors = Errors::default();
        errors.insert_with_default_key(AppError::NotFound);

        let handler = leptos_axum::render_app_to_stream(
            options.to_owned(),
            move || view! { <ErrorTemplate outside_errors=errors.clone()/> },
        );

        handler(req).await.into_response()
    }
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();

    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}
