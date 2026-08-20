# browsergame

A browser game, plus a small Rust/Rocket backend that keeps a leaderboard.

## Playing

```sh
cd server && cargo run          # needs: . "$HOME/.cargo/env"
```

Then open <http://localhost:8090>. The server hosts the game and the leaderboard from
one origin, so nothing needs configuring.

`index.html` still runs on its own by double-clicking it - the leaderboard panel just
reports "offline" until a server is reachable at `localhost:8090`.

## Leaderboard

- **Leaderboard** in the header opens the board. It pauses the game like the skill tree.
- Your best banked progress is what counts. It is reported when a run ends, when you
  open the board, and periodically while playing - only ever when you beat your own
  record.
- Set a display name in the panel; renaming keeps your place on the board.
- Scores are taken on the client's word - see `server/README.md`.

See `server/README.md` for the API, the websocket protocol and configuration.
