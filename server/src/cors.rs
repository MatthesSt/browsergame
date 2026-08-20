//! Permissive CORS for the API.
//!
//! When the game is served by this binary the browser never asks, because it is all
//! one origin. This exists for the other case: opening index.html straight off disk,
//! or from a dev server on another port, while pointing at this API.
//!
//! If you ever put this behind a real domain with anything user-specific, narrow
//! `Access-Control-Allow-Origin` to that domain instead of `*`.

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "GET, POST, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
        response.set_header(Header::new("Access-Control-Max-Age", "86400"));
    }
}
