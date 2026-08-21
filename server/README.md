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

## Deploy (Docker + Traefik)

`../docker-compose.yml` builds `Dockerfile` and hands the container to an existing
Traefik instance. The image carries the game files too, so the board and the game
come off one origin and CORS never enters into it.

```sh
cp .env.example .env      # in the repo root: set DOMAIN, and the names your Traefik uses
docker compose up -d --build
```

The compose file expects your Traefik network to exist already; on a machine that
has no Traefik, `docker network create traefik` is enough to satisfy it.

Traefik reaches port 8090 inside the container over the shared network; nothing is
published on the host. The websocket at `/api/ws` needs no extra configuration - it
rides the same router, and the client derives `wss://` from the page origin.

The board is persisted to the `leaderboard-data` volume at `/data/leaderboard.json`.
Run exactly one replica: the live-update fan-out is per-process state, so a second
one would serve a board of its own.

## Share it over a Cloudflare tunnel

To hand the running game to a few friends there is no need to build anything. With
`cloudflared` on the machine, next to `cargo run`:

```sh
cloudflared tunnel --url http://localhost:8090
```

It prints a `https://<random>.trycloudflare.com` address that anyone can open, with
no Cloudflare account, no domain and no open port. Websockets ride through it, so
the board still updates live. The address dies with the process and is different
every time.

The same thing as a container, alongside the server:

```sh
docker compose --profile quick up -d --build
docker compose logs quick-tunnel | grep trycloudflare.com
```

For an address that survives a restart, create a named tunnel under Cloudflare Zero
Trust > Networks > Tunnels on a domain you own, point its public hostname at
`http://browsergame:8090`, put the connector token in `.env` as `TUNNEL_TOKEN`, and:

```sh
docker compose --profile tunnel up -d --build
```

Either tunnel makes the game reachable from the internet by anyone with the link,
and scores are still taken on the client's word - see the last section.

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
