//! HTTP and websocket surface.
//!
//! REST is enough to post a score and read the board; the websocket exists so every
//! open tab sees a new high score the moment it lands, without polling.

use futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, options, post, State};
use rocket_ws as ws;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;

use crate::leaderboard::{Leaderboard, RankedEntry, SubmitOutcome, Submission, DEFAULT_LIMIT};

#[derive(Serialize)]
pub struct BoardResponse {
    total: usize,
    entries: Vec<RankedEntry>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

#[get("/health")]
pub fn health() -> Json<Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[get("/leaderboard?<limit>")]
pub async fn leaderboard(
    board: &State<Leaderboard>,
    limit: Option<usize>,
) -> Json<BoardResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    Json(BoardResponse {
        total: board.len().await,
        entries: board.top(limit).await,
    })
}

#[post("/scores", data = "<submission>")]
pub async fn submit_score(
    board: &State<Leaderboard>,
    submission: Json<Submission>,
) -> Result<Json<SubmitOutcome>, (Status, Json<ErrorResponse>)> {
    match board.submit(submission.into_inner()).await {
        Ok(outcome) => Ok(Json(outcome)),
        Err(err) => Err((
            Status::BadRequest,
            Json(ErrorResponse {
                error: err.message().to_string(),
            }),
        )),
    }
}

/// Browsers send a preflight before any cross-origin POST with a JSON body. Serving
/// the game from this same server avoids all of it, but the game also runs straight
/// off disk during development, so answer the preflight rather than fail it.
#[options("/<_..>")]
pub fn preflight() -> Status {
    Status::NoContent
}

/// Live board. On connect the client gets a full snapshot, then one message per
/// change. Clients may also submit over the same socket instead of using REST.
#[get("/ws")]
pub fn leaderboard_ws(ws: ws::WebSocket, board: &State<Leaderboard>) -> ws::Channel<'static> {
    let board = board.inner().clone();

    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut updates = board.subscribe();

            let snapshot = board.snapshot_json(DEFAULT_LIMIT).await;
            if stream.send(ws::Message::Text(snapshot)).await.is_err() {
                return Ok(());
            }

            loop {
                tokio::select! {
                    incoming = stream.next() => match incoming {
                        Some(Ok(ws::Message::Text(text))) => {
                            if let Some(reply) = handle_client_message(&board, &text).await {
                                if stream.send(ws::Message::Text(reply)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Some(Ok(ws::Message::Close(_))) | None => break,
                        Some(Err(_)) => break,
                        // Ping/pong and binary frames need no handling of ours.
                        Some(Ok(_)) => {}
                    },
                    update = updates.recv() => match update {
                        Ok(payload) => {
                            if stream.send(ws::Message::Text(payload)).await.is_err() {
                                break;
                            }
                        }
                        // A slow client missed some updates. The board is a snapshot
                        // anyway, so one fresh copy replaces everything it lost.
                        Err(RecvError::Lagged(_)) => {
                            let snapshot = board.snapshot_json(DEFAULT_LIMIT).await;
                            if stream.send(ws::Message::Text(snapshot)).await.is_err() {
                                break;
                            }
                        }
                        Err(RecvError::Closed) => break,
                    }
                }
            }

            Ok(())
        })
    })
}

/// Returns the reply to send back, if the message calls for one.
async fn handle_client_message(board: &Leaderboard, text: &str) -> Option<String> {
    let message: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => return Some(error_message("message is not valid JSON")),
    };

    match message.get("type").and_then(Value::as_str) {
        Some("ping") => Some(serde_json::json!({ "type": "pong" }).to_string()),
        Some("subscribe") => Some(board.snapshot_json(DEFAULT_LIMIT).await),
        Some("submit") => {
            let submission: Submission = match serde_json::from_value(message) {
                Ok(submission) => submission,
                Err(err) => return Some(error_message(&format!("bad submission: {err}"))),
            };

            match board.submit(submission).await {
                // The updated board reaches this client through its own subscription,
                // so the ack only has to report where the player landed.
                Ok(outcome) => Some(
                    serde_json::json!({
                        "type": "ack",
                        "rank": outcome.rank,
                        "best": outcome.best,
                        "improved": outcome.improved,
                    })
                    .to_string(),
                ),
                Err(err) => Some(error_message(err.message())),
            }
        }
        _ => Some(error_message("unknown message type")),
    }
}

fn error_message(reason: &str) -> String {
    serde_json::json!({ "type": "error", "error": reason }).to_string()
}
