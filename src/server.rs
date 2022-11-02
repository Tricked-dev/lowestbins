use std::collections::BTreeMap;

use hyper::{
    header,
    http::response,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use once_cell::sync::Lazy;
use serde_json::json;

use crate::{error::Result, AUCTIONS, CONFIG, SOURCE, SPONSOR};
// add a json not found response
static NOTFOUND: &[u8] = b"{\"error\": \"not found\"}";

pub async fn start_server() -> Result<()> {
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port)
        .parse()
        .expect("Failed to parse addr");

    let make_service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(response)) });

    let server = Server::bind(&addr).serve(make_service);

    tracing::info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        tracing::error!("server error: {}", e);
    }

    Ok(())
}

fn response_base() -> response::Builder {
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
        .header(header::CACHE_CONTROL, "max-age=60, s-maxage=60")
        .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
        .header("funding", SPONSOR)
}

async fn response(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/lowestbins.json") | (&Method::GET, "/lowestbins") | (&Method::GET, "/auctions/lowestbins") => {
            let bytes = serde_json::to_vec(&*AUCTIONS.lock().unwrap())?;
            Ok(response_base().body(Body::from(bytes))?)
        }
        (&Method::GET, "/metrics") => {
            static DISPLAY_NAMES: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
                let bytes = include_bytes!("../resources/display-names.bin");
                rmp_serde::from_slice(bytes).unwrap()
            });
            let mut res = "# HELP price Price of each item\n# TYPE price gauge".to_owned();
            for (item, price) in &*AUCTIONS.lock().unwrap() {
                let display_name = DISPLAY_NAMES.get(item).unwrap_or(item);
                res.push_str(&format!(
                    "\nlowestbin_price{{item=\"{}\", display=\"{}\"}} {}",
                    item, display_name, price,
                ));
            }

            Ok(response_base().body(Body::from(res)).unwrap())
        }
        (_, "/") => Ok(response_base().body(Body::from(serde_json::to_vec_pretty(&json!({
            "message": "Welcome to the lowestbins API",
            "endpoint": "/lowestbins",
            "funding": SPONSOR,
            "source": SOURCE
        }))?))?),

        _ => Ok(not_found()),
    }
}

/// HTTP status code 404
fn not_found() -> Response<Body> {
    response_base().status(404).body(NOTFOUND.into()).unwrap()
}
