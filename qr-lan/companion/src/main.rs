mod lan;
mod rewrite;

use axum::extract::Query;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use qrcode::render::svg;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use tokio::net::TcpListener;

/// Duplicated in `extension/popup.js` — deliberately, so neither side needs a
/// config file.
const PORT: u16 = 48213;

#[derive(Deserialize)]
struct QrQuery {
    url: String,
}

#[derive(Serialize)]
struct QrResponse {
    url: String,
    svg: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/qr", get(qr));
    let listener = match TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("qr-lan: cannot bind 127.0.0.1:{PORT}: {err}");
            std::process::exit(1);
        }
    };
    println!("qr-lan: listening on http://127.0.0.1:{PORT}");
    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("qr-lan: {err}");
        std::process::exit(1);
    }
}

async fn qr(Query(query): Query<QrQuery>) -> Response {
    // Resolved per request, so moving between networks needs no restart.
    let lan = match lan::lan_ip() {
        Ok(lan) => lan,
        Err(message) => return error(StatusCode::SERVICE_UNAVAILABLE, message),
    };
    let url = match rewrite::rewrite(&query.url, lan) {
        Ok(url) => url,
        Err(err) => return error(StatusCode::BAD_REQUEST, err.message()),
    };
    let Ok(code) = QrCode::new(url.as_bytes()) else {
        return error(StatusCode::BAD_REQUEST, "URL is too long for a QR code");
    };
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(256, 256)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    ok(Json(QrResponse { url, svg }))
}

fn ok(body: impl IntoResponse) -> Response {
    // Loopback-bound with no secrets to leak, so any origin may ask. This
    // spares us pinning an extension ID that differs per machine.
    ([(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")], body).into_response()
}

fn error(status: StatusCode, message: &str) -> Response {
    let body = Json(ErrorResponse { error: message.to_string() });
    (status, [(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")], body).into_response()
}
