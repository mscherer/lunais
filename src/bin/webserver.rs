use askama::Template;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use chrono_tz::Tz;
use lunais::index_page::IndexTemplate;
use std::env;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

// TODO set a different port
const PORT: u16 = 2507;

/*
 * fn parse_tz(paths: Vec<&str>) -> Option<Vec<Tz>> {
    let res = Vec::new();
    if  paths.len() < 2 {
        return None;
    }
    if  paths.len() > 4 {
        return None;
    }

    Some(res)
} */

pub async fn index_handler() -> impl IntoResponse {
    let template = IndexTemplate::new();
    if let Ok(body) = template.render() {
        (StatusCode::OK, Html(body)).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

pub async fn ical_handler(Path(path): Path<String>) -> impl IntoResponse {
    let elements: Vec<&str> = path.split('/').collect();
    println!("{:?}", elements);
    match elements.len() {
        2 => {
            let tz: Result<Tz, _> = elements[0].parse();
            println!("{:?}", tz);
            "ok 2"
        }
        3 => "ok 3",
        4 => "ok 4",
        _ => "not ok",
    }
}

pub async fn health_checker_handler() -> impl IntoResponse {
    "All is fine"
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let port: u16 = match env::var("PORT") {
        Ok(val) => val.parse().unwrap_or_else(|_| {
            tracing::warn!("Incorrect PORT value: {val}, using default: {PORT}");
            PORT
        }),
        Err(_) => PORT,
    };
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/calendars/{*tzs}", get(ical_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // https://github.com/tokio-rs/axum/discussions/355
        .route("/healthz", get(health_checker_handler));

    tracing::info!("Server started on port {port}");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
