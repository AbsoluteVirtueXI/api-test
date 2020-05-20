#![deny(warnings)]
use warp::Filter;

#[tokio::main]
async fn main() {
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([192, 168, 0, 10], 3030)).await;
}
