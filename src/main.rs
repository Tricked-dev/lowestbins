#![deny(warnings)]

use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Method, Request, Response, Result, Server, StatusCode};
use std::env;
use std::future::Future;
use std::process::Command;
use tokio::fs::File;
use tokio::time::{self, Duration};
use tokio_util::codec::{BytesCodec, FramedRead};

static INDEX: &str = "lowestbins.json";
static NOTFOUND: &[u8] = b"Not Found";

fn auctions() -> std::io::Result<()> {
    println!("test");
    let path = env::current_dir()?;
    println!("{}", path.display());
    Command::new("node")
        .arg("lowestbins.js")
        .spawn()
        .expect("failed to execute process");

    Ok(())
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let addr = "127.0.0.1:1337".parse().unwrap();

    let make_service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(response_examples)) });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    auctions().expect("error");
    set_interval(
        || async { auctions().expect("error") },
        Duration::from_millis(300000),
    );

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn response_examples(req: Request<Body>) -> Result<Response<Body>> {
    println!("Request");
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/lowestbins.json") | (&Method::GET, "/lowestbins") => {
            println!("/ GET");
            // Response::builder().header(header::CONTENT_TYPE, "application/json").body(simple_file_send(INDEX));
            simple_file_send(INDEX).await
        }
        _ => Ok(not_found()),
    }
}

/// HTTP status code 404
fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

async fn simple_file_send(filename: &str) -> Result<Response<Body>> {
    // Serve a file by asynchronously reading it by chunks using tokio-util crate.

    if let Ok(file) = File::open(filename).await {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        return Ok(Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap());
    }

    Ok(not_found())
}

fn set_interval<F, Fut>(mut f: F, dur: Duration)
where
    F: Send + 'static + FnMut() -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    // Create stream of intervals.
    let mut interval = time::interval(dur);
    tokio::spawn(async move {
        // Skip the first tick at 0ms.
        interval.tick().await;
        loop {
            // Wait until next tick.
            interval.tick().await;
            // Spawn a task for this tick.
            tokio::spawn(f());
        }
    });
}
