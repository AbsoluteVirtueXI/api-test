#![deny(warnings)]
use std::net::SocketAddr;
use warp::Filter;

#[tokio::main]
async fn main() {
    // we assume no DNS was used, so the Host header should be an address
    let host = warp::header::<SocketAddr>("host");

    // Match when we get `accept: */*` exactly
    let accept_stars = warp::header::exact("accept", "*/*");

    let routes = host.and(accept_stars).map(|addr| {
       format!("accepting stars on {}", addr)
    });

    warp::serve(routes).run(([192, 168, 0, 10], 3030)).await;
}