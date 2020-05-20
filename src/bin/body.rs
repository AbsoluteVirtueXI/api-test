#![deny(warnings)]
//use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

#[tokio::main]
async fn main() {
    // POST /employees/:rate {"name":"sofiane", "rate":2}
    let promote = warp::post()
        .and(warp::path("employees"))
        .and(warp::path::param::<u32>())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|rate, mut employee: Employee|{
            employee.rate = rate;
            warp::reply::json(&employee)
        });
    warp::serve(promote).run(([192, 168, 0, 10], 3030)).await
}