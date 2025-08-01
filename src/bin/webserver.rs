use askama::Template;
use axum::Router;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::http::{HeaderValue, header};
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum_response_cache::CacheLayer;
use chrono::Datelike;
use chrono::Local;
use lunais::disruption_calendar::generate_ical;
use lunais::index_page::IndexTemplate;
use lunais::timezone_pair::TimezonePair;
use std::env;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

const PORT: u16 = 2507;
const TEXT_CALENDAR: HeaderValue = HeaderValue::from_static("text/calendar");

pub async fn index_handler() -> impl IntoResponse {
    let template = IndexTemplate::new();
    if let Ok(body) = template.render() {
        (StatusCode::OK, Html(body)).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

pub async fn ical_handler(Path(tzp): Path<TimezonePair>) -> impl IntoResponse {
    let year = Local::now().naive_utc().date().year();

    let mut d = Vec::new();
    for y in year - 1..year + 3 {
        d.append(&mut tzp.get_disruption_dates(y))
    }

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, TEXT_CALENDAR);

    let i = generate_ical(&d);
    (headers, i.to_string()).into_response()
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
        .route(
            "/calendars/{*tzs}",
            get(ical_handler).layer(CacheLayer::with_lifespan(24 * 60 * 60)),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        // after the tracing, to not pollute log, see
        // https://github.com/tokio-rs/axum/discussions/355
        .route("/healthz", get(|| async { StatusCode::OK }));

    tracing::info!("Server started on port {port}");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
