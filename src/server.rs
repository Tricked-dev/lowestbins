use hyper::{
    header,
    http::response,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use serde_json::json;

use crate::{calc_next_update, error::Result, round_to_nearest_15, AUCTIONS, CONFIG, SOURCE, SPONSOR};
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
    let update = round_to_nearest_15(calc_next_update());
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
        .header(header::CACHE_CONTROL, format!("max-age={update}, s-maxage={update}"))
        .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
        .header("funding", SPONSOR)
}

async fn response(req: Request<Body>) -> Result<Response<Body>> {
    let path = req.uri().path().trim_end_matches('/');

    match (req.method(), path) {
        (&Method::GET, "/lowestbins.json") | (&Method::GET, "/lowestbins") | (&Method::GET, "/auctions/lowestbins") => {
            let bytes = serde_json::to_vec(&*AUCTIONS.lock())?;
            Ok(response_base().body(Body::from(bytes))?)
        }
        (&Method::GET, "/lowestbins.txt") => {
            let mut res = String::new();
            for (key, value) in &*AUCTIONS.lock() {
                res.push_str(&format!("{key} {value}\n"));
            }
            Ok(response_base().body(Body::from(res))?)
        }
        (&Method::GET, route) if route.starts_with("/auction/") || route.starts_with("/lowestbin/") => {
            let id = route.trim_start_matches("/auction/").trim_start_matches("/lowestbin/");
            let auctions = AUCTIONS.lock();
            let value = auctions.get(id);

            if let Some(auction) = value {
                let bytes = serde_json::to_vec(&auction)?;
                Ok(response_base().body(Body::from(bytes))?)
            } else {
                Ok(not_found())
            }
        }
        (&Method::GET, "/metrics") => {
            let mut res = "# HELP price Price of each item\n# TYPE price gauge".to_owned();
            for (item, price) in &*AUCTIONS.lock() {
                let display_name = to_display_name(item);
                res.push_str(&format!(
                    "\nlowestbin_price{{item=\"{item}\", display=\"{display_name}\"}} {price}",
                ));
            }

            Ok(response_base().body(Body::from(res)).unwrap())
        }
        (_, "") => Ok(response_base()
            .header(header::CACHE_CONTROL, "max-age=2, s-maxage=2")
            .body(Body::from(serde_json::to_vec_pretty(&json!({
                "message": "Welcome to the lowestbins API",
                "endpoint": "/lowestbins",
                "updates_in": calc_next_update(),
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

include!("../generated/to_display_name.rs");
