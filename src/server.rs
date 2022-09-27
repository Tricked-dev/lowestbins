use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Result, Server};

use std::env;

use crate::AUCTIONS;

static NOTFOUND: &[u8] = b"Not Found";

pub async fn start_server() -> Result<()> {
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_owned());
    let addr = format!("{host}:{port}").parse().unwrap();

    let make_service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(response_examples)) });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn response_examples(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/lowestbins.json") | (&Method::GET, "/lowestbins") => {
            let bytes = (*AUCTIONS.lock().unwrap()).clone();
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::CACHE_CONTROL, "max-age=60")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Headers", "*")
                .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                .body(Body::from(bytes))
                .unwrap())
        }
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
