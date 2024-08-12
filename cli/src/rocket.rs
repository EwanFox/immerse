use rocket::fs::{relative, FileServer};
use rocket::{get, routes, Rocket};

pub fn rocket() -> Rocket<rocket::Build> {
    rocket::build()
        .mount("/rev", FileServer::from(relative!("static")))
        .mount("/ws", routes![echo_stream])
}

#[get("/echo?stream")]
fn echo_stream(ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>
        for await message in ws {
            yield message?;
        }
    }
}
