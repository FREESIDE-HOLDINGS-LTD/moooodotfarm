use crate::app::GetHerdHandler;
use crate::config::Environment;
use crate::errors::{Error, Result};
use crate::{app, config};
use askama::Template;
use axum::response::Html;
use axum::{Router, routing::get};
use axum::{
    extract::Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use http::header;
use include_dir::{Dir, include_dir};
use prometheus::TextEncoder;
use serde::Serialize;
use std::fmt::Display;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

static STATIC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/ports/http/static");

pub struct Server {}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run<D>(&self, config: &config::Config, deps: D) -> Result<()>
    where
        D: Deps + Sync + Send + Clone + 'static,
    {
        let trace = TraceLayer::new_for_http();
        let cors = match config.environment() {
            Environment::Production => CorsLayer::new(),
            Environment::Development => CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        };

        let compression = CompressionLayer::new();

        let app = Router::new()
            .route("/", get(handle_get_index::<D>))
            .route("/rfc", get(handle_get_rfc))
            .route("/metrics", get(handle_get_metrics::<D>))
            .route("/api/herd", get(handle_get_herd::<D>))
            .fallback(handle_static)
            .layer(
                ServiceBuilder::new()
                    .layer(trace.clone())
                    .layer(compression.clone())
                    .layer(cors.clone()),
            )
            .with_state(deps);

        let listener = tokio::net::TcpListener::bind(config.address()).await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn handle_get_index<D>(State(deps): State<D>) -> std::result::Result<Html<String>, AppError>
where
    D: Deps,
{
    let herd = deps.get_herd_handler().get_herd()?;
    let template = IndexTemplate {
        cows: herd.cows().iter().map(|v| v.into()).collect(),
    };
    Ok(Html(template.render()?))
}

async fn handle_get_rfc() -> std::result::Result<Html<String>, AppError> {
    let template = RfcTemplate {};
    Ok(Html(template.render()?))
}

async fn handle_static(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    match STATIC_DIR.get_file(path) {
        Some(file) => match get_mime_type(path) {
            Ok(mime) => ([(header::CONTENT_TYPE, mime)], file.contents()).into_response(),
            Err(_) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file type").into_response(),
        },
        None => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

fn get_mime_type(path: &str) -> std::result::Result<&'static str, ()> {
    if path.ends_with(".png") {
        Ok("image/png")
    } else if path.ends_with(".ico") {
        Ok("image/x-icon")
    } else if path.ends_with(".txt") {
        Ok("text/plain; charset=utf-8")
    } else {
        Err(())
    }
}

async fn handle_get_metrics<D>(State(deps): State<D>) -> std::result::Result<String, AppError>
where
    D: Deps,
{
    let encoder = TextEncoder::new();
    let families = deps.metrics().gather();
    Ok(encoder.encode_to_string(&families)?)
}

async fn handle_get_herd<D>(State(deps): State<D>) -> std::result::Result<Json<Herd>, AppError>
where
    D: Deps,
{
    let herd = deps.get_herd_handler().get_herd()?;
    Ok(Json(Herd::from(&herd)))
}

#[derive(Serialize)]
struct Herd {
    cows: Vec<Cow>,
}

impl From<&app::Herd> for Herd {
    fn from(value: &app::Herd) -> Self {
        Self {
            cows: value.cows().iter().map(|v| v.into()).collect(),
        }
    }
}

#[derive(Serialize)]
struct Cow {
    name: String,
    last_seen: Option<String>,
}

const DT_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

impl From<&app::Cow> for Cow {
    fn from(value: &app::Cow) -> Self {
        Self {
            name: value.name().url().to_string(),
            last_seen: value.last_seen().map(|dt| dt.format(DT_FORMAT)),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    cows: Vec<TemplateCow>,
}

#[derive(Template)]
#[template(path = "rfc.html")]
struct RfcTemplate {}

struct TemplateCow {
    name: String,
    last_seen: String,
    status: CowStatus,
}

impl From<&app::Cow> for TemplateCow {
    fn from(value: &app::Cow) -> Self {
        Self {
            name: value.name().url().to_string(),
            last_seen: value
                .last_seen()
                .map(|v| v.ago())
                .unwrap_or_else(|| "never".to_string()),
            status: value.status().into(),
        }
    }
}

pub enum CowStatus {
    HappilyGrazing,
    RanAway,
    HaveNotCheckedYet,
}

impl From<&app::CowStatus> for CowStatus {
    fn from(value: &app::CowStatus) -> Self {
        match value {
            app::CowStatus::HappilyGrazing => CowStatus::HappilyGrazing,
            app::CowStatus::RanAway => CowStatus::RanAway,
            app::CowStatus::HaveNotCheckedYet => CowStatus::HaveNotCheckedYet,
        }
    }
}

impl Display for CowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CowStatus::HappilyGrazing => write!(f, "happily-grazing"),
            CowStatus::RanAway => write!(f, "ran-away"),
            CowStatus::HaveNotCheckedYet => write!(f, "have-not-checked-yet"),
        }
    }
}

pub trait Deps {
    fn get_herd_handler(&self) -> &impl GetHerdHandler;
    fn metrics(&self) -> &prometheus::Registry;
}

enum AppError {
    UnknownError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UnknownError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };
        (status, Json(TransportError { message })).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<Error>,
{
    fn from(_err: E) -> Self {
        Self::UnknownError
    }
}

impl From<askama::Error> for AppError {
    fn from(_err: askama::Error) -> Self {
        Self::UnknownError
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransportError {
    message: String,
}
