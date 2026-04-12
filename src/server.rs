use flate2::write::GzEncoder;
use flate2::Compression;
use hyper::{
    body::Bytes,
    header,
    http::response,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use crate::{calc_next_update, error::Result, history, round_to_nearest_15, FilterPrice, FilterType, AUCTIONS, CONFIG, SOURCE, SPONSOR};

// Standard JSON response for 404 Not Found errors
static NOTFOUND: &[u8] = b"{\"error\": \"not found\"}";

// Global response cache for heavy, frequently accessed endpoints
static RESPONSE_CACHE: Lazy<RwLock<HashMap<String, Bytes>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Clears the global response cache. Should be called after AUCTIONS map updates.
pub fn clear_response_cache() {
    RESPONSE_CACHE.write().clear();
}

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
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
        .header(header::CACHE_CONTROL, format!("max-age={update}, s-maxage={update}"))
        .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
        .header("funding", SPONSOR)
}

/// Compresses a byte slice using standard Gzip (DEFLATE) algorithm
fn gzip_bytes(data: &[u8]) -> Result<Bytes> {
    let mut encoder = GzEncoder::new(
        Vec::with_capacity(data.len() / 3),
        Compression::default()
    );
    encoder.write_all(data)?;
    Ok(Bytes::from(encoder.finish()?))
}

async fn response(req: Request<Body>) -> Result<Response<Body>> {
    // Handle CORS Preflight (OPTIONS)
    if req.method() == Method::OPTIONS {
        return Ok(response_base().status(204).body(Body::empty())?);
    }
    // Only permit GET and HEAD requests for data
    let is_head = req.method() == Method::HEAD;
    if !is_head && req.method() != Method::GET {
        return Ok(response_base().status(405).body(Body::empty())?);
    }

    let uri_path = req.uri().path();
    let no_trailing = uri_path.strip_suffix('/').unwrap_or(uri_path);
    let query = req.uri().query().unwrap_or("");

    // Safely identify and strip the .gz suffix if present
    let is_gzip = no_trailing.ends_with(".gz") && !no_trailing.ends_with("/.gz");
    let clean_path = if is_gzip {
        no_trailing.strip_suffix(".gz").unwrap_or(no_trailing)
    } else {
        no_trailing
    };

    // Split the route from the parameter to preserve case-sensitivity for item IDs
    let (route_base, path_param) = if clean_path.is_empty() {
        ("", None)
    } else if let Some(idx) = clean_path[1..].find('/') {
        let split_idx = idx + 1;
        (&clean_path[..split_idx], Some(&clean_path[split_idx + 1..]))
    } else {
        (clean_path, None)
    };

    // Normalize only the route segment for secure and consistent matching
    let normalized_route = route_base.to_ascii_lowercase();
    let route_str = if normalized_route.is_empty() {
        ""
    } else {
        normalized_route.as_str()
    };

    let content_type = if is_gzip {
        "application/gzip"
    } else {
        match route_str {
            "/lowestbins.txt" => "text/plain",
            "/metrics" => "text/plain; version=0.0.4",
            _ => "application/json",
        }
    };
    let mut resp = response_base().header(header::CONTENT_TYPE, content_type);

    let (q_type, q_price) = parse_query(query);

    // Construct a canonical cache key incorporating all variables to prevent cache collisions
    let cache_key = format!(
        "{}:{}:{}:{}:{}",
        route_str,
        path_param.unwrap_or(""),
        q_type as u8,
        q_price as u8,
        is_gzip
    );

    // Bypass RESPONSE_CACHE for averages (already cached in history.rs)
    if route_str == "/averages" && CONFIG.enable_history {
        let param = path_param.unwrap_or("");
        // Safely strip optional .json suffix, then extract the day integer
        let base_param = param.strip_suffix(".json").unwrap_or(param);
        let days_str = base_param.strip_suffix("day").unwrap_or("");

        #[allow(clippy::collapsible_if)]
        if let Ok(days) = days_str.parse::<usize>() {
            if (1..=history::DAY_SLOTS).contains(&days) {
                return if let Some(bytes) = history::get_cache(days as u8) {
                    if req.method() == Method::HEAD {
                        return Ok(resp.body(Body::empty())?);
                    }

                    let final_data = if is_gzip {
                        gzip_bytes(&bytes)?
                    } else {
                        bytes.clone()
                    };
                    Ok(resp.body(Body::from(final_data))?)
                } else {
                    Ok(response_base()
                        .header(header::CONTENT_TYPE, "application/json")
                        .status(503)
                        .body(Body::from(r#"{"error": "History cache is currently building. Please try again in a moment."}"#))?)
                }
            }
        }
        return Ok(not_found());
    }

    // Check pre-computed cache for heavy endpoints
    #[allow(clippy::collapsible_if)]
    if route_str == "/lowestbins" || route_str == "/lowestbins.json" || route_str == "/lowestbins.txt" {
        if let Some(cached_data) = RESPONSE_CACHE.read().get(&cache_key) {
            if req.method() == Method::HEAD {
                return Ok(resp.body(Body::empty())?);
            }
            return Ok(resp.body(Body::from(cached_data.clone()))?);
        }
    }

    if is_head {
        return Ok(resp.body(Body::empty())?);
    }

    let raw_bytes: Vec<u8> = match (route_str, path_param) {
        ("/lowestbins" | "/lowestbins.json", None) => {
            let map = AUCTIONS.lock().build_combined_map(q_type, q_price);
            serde_json::to_vec(&map)?
        }
        ("/lowestbins.txt", None) => {
            let auc = AUCTIONS.lock();
            // approximately 30 characters per line
            let mut res = String::with_capacity(auc.historical_auctions.len() * 30);
           auc.for_each_price(q_type, q_price, |key, value| {
               let _ = writeln!(res, "{} {}", key, value);
            });
            res.into_bytes()
        }
        ("/auction" | "/lowestbin", Some(param)) => {
            let id = param.strip_suffix(".json").unwrap_or(param);

            if let Some(price) = AUCTIONS.lock().get_price(id, q_type, q_price) {
                serde_json::to_vec(&price)?
            } else {
                return Ok(not_found());
            }
        }
        ("/metrics", None) => {
            let auc = AUCTIONS.lock();
            let mut res = String::with_capacity(auc.historical_auctions.len() * 90);
            res.push_str("# HELP price Price of each item\n# TYPE price gauge\n");
            auc.for_each_price(FilterType::All, FilterPrice::Historical, |item, price| {
                let display_name = to_display_name(item);
                let _ = writeln!(
                    res,
                    "lowestbin_price{{item=\"{}\", display=\"{}\"}} {}",
                    item, display_name, price
                );
            });
            res.into_bytes()
        }
        ("/" | "", None) => {
            let mut endpoints = vec!["/lowestbins.json".to_owned(), "/lowestbins.txt".to_owned()];
            if CONFIG.enable_history {
                for days in 1..=history::DAY_SLOTS {
                    endpoints.push(format!("/averages/{}day.json", days));
                }
            }
            let bytes = serde_json::to_vec_pretty(&json!({
                "message": "Welcome to the lowestbins API",
                "endpoints": endpoints,
                "parameters": {
                    "type": "Filters items by source. Options: 'all' (default), 'auction', 'bazaar'",
                    "price": "Filters auction items by state. Options: 'historical' (default, last known price), 'available' (currently active on ah)"
                },
                "updates_in": calc_next_update(),
                "funding": SPONSOR,
                "source": SOURCE
            }))?;

            resp = resp.header(header::CACHE_CONTROL, "max-age=2, s-maxage=2");
            bytes
        }
        _ => {
            return Ok(not_found());
        }
    };

    // Apply compression
    let final_data = if is_gzip {
        gzip_bytes(&raw_bytes)?
    } else {
        Bytes::from(raw_bytes)
    };

    // Save only heavy endpoints to cache
    if route_str == "/lowestbins" || route_str == "/lowestbins.json" || route_str == "/lowestbins.txt" {
        RESPONSE_CACHE.write().insert(cache_key, final_data.clone());
    }

    Ok(resp.body(Body::from(final_data))?)
}

/// HTTP status code 404
fn not_found() -> Response<Body> {
    response_base()
        .status(404)
        .header(header::CONTENT_TYPE, "application/json")
        .body(NOTFOUND.into())
        .unwrap()
}

// Helper for strict query parsing
fn parse_query(query: &str) -> (FilterType, FilterPrice) {
    let mut t = FilterType::All;
    let mut p = FilterPrice::Historical;

    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            match key {
                "type" => t = match value {
                    "auction" => FilterType::Auction,
                    "bazaar" => FilterType::Bazaar,
                    _ => FilterType::All,
                },
                "price" => p = match value {
                    "available" => FilterPrice::Available,
                    _ => FilterPrice::Historical,
                },
                _ => {}
            }
        }
    }
    (t, p)
}

include!("../generated/to_display_name.rs");
