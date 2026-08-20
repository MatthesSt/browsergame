# Leaderboard server

Rocket backend for the game: REST for posting and reading scores, a websocket that
pushes the board to every open tab as it changes, and static hosting for the game
itself so the whole thing runs on one origin.

## Run

Rust lives in `~/.cargo/bin`, which is not on the PATH by default here - rustup was
installed without touching the shell profile. Once per terminal:

```sh
. "$HOME/.cargo/env"
```

or add that same line to `~/.bashrc` to have it always.

```sh
cd server
cargo run              # http://localhost:8090  (game + API)
cargo run --release
```

Then open <http://localhost:8090>. Needs rustup and a C toolchain (`build-essential`),
both installed here; a clean debug build takes about 40 seconds.

The port is 8090 rather than the usual 8080 because this machine already runs Adminer
on 8080 and VS Code on 8081. Override with `ROCKET_PORT` if that changes.

## Configuration

| Variable                   | Default                 | Meaning                                  |
| -------------------------- | ----------------------- | ---------------------------------------- |
| `BROWSERGAME_STATIC_DIR`   | `..`                    | Directory served at `/` (the game)        |
| `BROWSERGAME_DATA`         | `data/leaderboard.json` | Where the board is persisted              |
| `ROCKET_PORT`              | `8090`                  | Any `Rocket.toml` key works as `ROCKET_*` |
| `ROCKET_ADDRESS`           | `0.0.0.0`               |                                           |

## HTTP

```
GET  /api/health
GET  /api/leaderboard?limit=20
POST /api/scores        {"player_id": "...", "name": "...", "score": 1234, "wave": 7}
GET  /api/ws            websocket
```

`POST /api/scores` answers with `{"rank": 3, "best": 1234, "improved": true}`.
`improved` is false when the score did not beat that player's own record.

## Websocket

The client gets a full board on connect, then one message per change:

```json
{ "type": "leaderboard", "total": 42,
  "entries": [{ "rank": 1, "player_id": "…", "name": "Gandalf", "score": 9001, "wave": 12, "updated_at": 1718000000 }] }
```

Client to server:

```json
{ "type": "submit", "player_id": "…", "name": "…", "score": 1234, "wave": 7 }
{ "type": "subscribe" }   // resend the snapshot
{ "type": "ping" }        // -> {"type":"pong"}
```

A submit is answered with `{"type":"ack","rank":3,"best":1234,"improved":true}`, and
the updated board arrives separately through the subscription every client has.

## Model

- One entry per `player_id`, holding that player's **best** score. Re-submitting a
  worse run changes nothing; renaming always takes effect.
- `player_id` is generated and stored by the client (see `../leaderboard-client.js`).
  It is an ownership token, not an account: whoever holds it owns that row.
- Ties are ordered by who reached the score first.
- The board is written to `BROWSERGAME_DATA` after every change, via a temp file and
  a rename so a crash mid-write cannot corrupt it.

## What this does not do

Scores are accepted on the client's word. Anyone who can open devtools can post any
number they like. Fixing that needs the server to own the game rules — validating a
replay, or running the simulation itself. If the board is ever competitive, that is
the piece to build; nothing else here will substitute for it.
