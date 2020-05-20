#![deny(warnings)]

use futures::{FutureExt, StreamExt};
use warp::Filter;

#[tokio::main]
async fn main() {
    let websocket = warp::path("echo")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    let html_content = warp::path("content").map(||"CONTNET HERE");

    let routes = html_content.or(websocket);

    warp::serve(routes).run(([192, 168, 0, 10], 3030)).await;
}