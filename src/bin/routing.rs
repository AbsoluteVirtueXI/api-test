#![deny(warnings)]

use warp::Filter;
use serde::{Deserialize};


#[derive(Deserialize)]
struct SumQuery {
    left: u32,
    right: u32,
}

#[tokio::main]
async fn main() {

    let sumquery = warp::path("sumquery")
        .and(warp::query::<SumQuery>())
        .and(warp::path::end())
        .map(|sum_query: SumQuery|{
           format!("{} + {} = {}", sum_query.left, sum_query.right, sum_query.left + sum_query.right)
        });

    let rawquery = warp::path("rawquery")
        .and(warp::query::raw())
        .map(|receive| {
           receive
        });

    // GET /hi
    let hi = warp::path("hi").and(warp::path::end()).map(|| "Hello, world");

    // GET /bye/:string
    let bye = warp::path("bye")
        .and(warp::path::param())
        .and(warp::path::end())
        .map(|name: String| format!("Good bye, {}!", name));

    // GET /hello/from/warp
    let hello_from_warp = warp::path!("hello" / "from" / "warp")
        .map(|| "Hello from warp");

    // GET /sum/:u32/:u32
    let sum = warp::path!("sum" / u32 / u32).map(|a, b| format!{"{} + {} = {}", a, b, a + b});

    // GET /:u16/times/:u16
    let times = warp::path!(u16 / "times" / u16)
        .map(|a, b| format!{"{} times {} = {}", a, b, a*b });

    // GET /math/sum/:u32/:u32
    // GEt /math/:16/times/:16
    let math = warp::path("math").and(sum.or(times));
    // GET /math
    let help = warp::path("math")
        .and(warp::path::end())
        .map(|| "This is the Math api. Try calling /math/sum/:u32/:u32 or /math/:u16/times/:u16");
    let math = help.or(math);
    let sum =
        sum.map(|output| format!("(This route has moved to /math/sum/:u16/:u16) {}", output));
    let times =
        times.map(|output| format!("(This route has moved to /math/:u16/times/:u16) {}", output));

    let routes = warp::get()
        .and(
            hi
                .or(bye)
                .or(hello_from_warp)
                .or(math)
                .or(sum)
                .or(times)
                .or(sumquery)
                .or(rawquery));

    warp::serve(routes).run(([192, 168, 0, 10], 3030)).await;
}