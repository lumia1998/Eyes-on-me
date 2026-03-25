use std::{path::PathBuf, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
};
use tokio::fs;

#[derive(Clone)]
struct StaticState {
    web_dist_dir: Arc<PathBuf>,
}

pub fn router(web_dist_dir: PathBuf) -> Router {
    let state = StaticState {
        web_dist_dir: Arc::new(web_dist_dir),
    };

    Router::new()
        .route("/", get(index))
        .route("/{*path}", get(asset))
        .with_state(state)
}

async fn index(State(state): State<StaticState>) -> impl IntoResponse {
    serve_file(state.web_dist_dir.join("index.html")).await
}

async fn asset(State(state): State<StaticState>, uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let candidate = state.web_dist_dir.join(path);

    if file_exists(&candidate).await {
        return serve_file(candidate).await;
    }

    serve_file(state.web_dist_dir.join("index.html")).await
}

async fn file_exists(path: &PathBuf) -> bool {
    fs::metadata(path)
        .await
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
}

async fn serve_file(path: PathBuf) -> impl IntoResponse {
    match fs::read(&path).await {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                Body::from(bytes),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            Body::from("not found"),
        )
            .into_response(),
    }
}
