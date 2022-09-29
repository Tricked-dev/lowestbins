use axum::{extract::Query, routing::get, Router};
use rust_embed::RustEmbed;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct Pagination {
    page: i8,
}

#[derive(RustEmbed)]
#[folder = "./pages/"]
struct Asset;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route(
        "/skyblock/auctions",
        get(|pagination: Query<Pagination>| async move {
            println!("Page: {}", pagination.page);
            let page = pagination.page;
            Asset::get(&format!("{}.json", page)).unwrap().data
        }),
    );

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
