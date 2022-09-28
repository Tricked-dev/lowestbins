use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Result, Server};
use serde_json::json;

use std::env;

use crate::AUCTIONS;

static NOTFOUND: &[u8] = b"Not Found";
static PORT: &str = "PORT";
static HOST: &str = "HOST";

pub async fn start_server() -> Result<()> {
    let port = env::var(PORT).unwrap_or_else(|_| "8080".to_string());
    let host = env::var(HOST).unwrap_or_else(|_| "127.0.0.1".to_owned());
    let addr = format!("{host}:{port}").parse().unwrap();

    let make_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(response)) });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn response(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/lowestbins.json")
        | (&Method::GET, "/lowestbins")
        | (&Method::GET, "/auctions/lowestbins") => {
            let bytes = serde_json::to_vec(&*AUCTIONS.lock().unwrap()).unwrap();
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::CACHE_CONTROL, "max-age=60, s-maxage=60")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
                .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
                .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
                .header("funding", "https://github.com/sponsors/Tricked-dev")
                .body(Body::from(bytes))
                .unwrap())
        }
        (_, "/") => Ok(Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
            .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
            .body(Body::from(
                serde_json::to_vec_pretty(&json!({
                    "message": "Welcome to the lowestbins API",
                    "endpoint": "/lowestbins",
                    "funding": "https://github.com/sponsors/Tricked-dev"
                }))
                .unwrap(),
            ))
            .unwrap()),
        _ => Ok(not_found()),
    }
}

/// HTTP status code 404
fn not_found() -> Response<Body> {
    Response::builder()
        .status(404)
        .body(NOTFOUND.into())
        .unwrap()
}
