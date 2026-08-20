//! Leaderboard backend for the browser game.
//!
//! Serves the game itself, a small REST API for posting and reading scores, and a
//! websocket that pushes the board to every open tab as it changes.
//!
//!   GET  /                    the game (static files from the repo root)
//!   GET  /api/health
//!   GET  /api/leaderboard?limit=20
//!   POST /api/scores          {player_id, name, score, wave}
//!   GET  /api/ws              websocket: snapshot on connect, then live updates
//!
//! Both paths are configurable so the binary can run from anywhere:
//!   BROWSERGAME_STATIC_DIR   directory to serve (default: the parent directory)
//!   BROWSERGAME_DATA         leaderboard file (default: ./data/leaderboard.json)

mod cors;
mod leaderboard;
mod routes;

use std::path::PathBuf;

use rocket::fs::FileServer;
use rocket::{routes, Build, Rocket};

use crate::cors::Cors;
use crate::leaderboard::Leaderboard;

fn static_dir() -> PathBuf {
    std::env::var("BROWSERGAME_STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".."))
}

fn data_path() -> PathBuf {
    std::env::var("BROWSERGAME_DATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data/leaderboard.json"))
}

#[rocket::launch]
async fn rocket() -> Rocket<Build> {
    let board = Leaderboard::load(data_path()).await;
    println!(
        "leaderboard: {} entries loaded from {}",
        board.len().await,
        data_path().display()
    );

    let mut app = rocket::build()
        .manage(board)
        .attach(Cors)
        .mount(
            "/api",
            routes![
                routes::health,
                routes::leaderboard,
                routes::submit_score,
                routes::leaderboard_ws,
                routes::preflight,
            ],
        );

    // Serving the game from the same origin as the API is what makes CORS a
    // non-issue in production; skip it when the directory is not there.
    let static_dir = static_dir();
    if static_dir.is_dir() {
        app = app.mount("/", FileServer::from(static_dir));
    } else {
        eprintln!(
            "static files: {} is not a directory, serving the API only",
            static_dir.display()
        );
    }

    app
}
