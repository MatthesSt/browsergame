# syntax=docker/dockerfile:1

# Build context is the repo root, not server/ - the image needs the game files
# (index.html, leaderboard-client.js) as well as the crate:
#   docker build -f server/Dockerfile -t browsergame-server .

FROM rust:1-bookworm AS build
WORKDIR /build

# Dependencies compile first against a stub main, so the ~200 crates behind Rocket
# stay cached and only the game's own sources rebuild on a code change.
COPY server/Cargo.toml server/Cargo.lock ./
RUN mkdir src \
 && echo 'fn main() {}' > src/main.rs \
 && cargo build --release \
 && rm -rf src

COPY server/src ./src
# Cargo goes by mtime, and COPY can hand it timestamps older than the stub build.
RUN touch src/main.rs && cargo build --release


FROM debian:bookworm-slim AS runtime

# curl is only here for the container healthcheck; nothing in the server needs it.
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --no-create-home app

WORKDIR /app
COPY --from=build /build/target/release/browsergame-server /usr/local/bin/browsergame-server
# Read from the working directory at startup: keeps the 8 KiB JSON body limit.
COPY server/Rocket.toml ./Rocket.toml
COPY index.html leaderboard-client.js ./static/

# The game and the API share one origin, which is what keeps CORS out of the picture.
ENV BROWSERGAME_STATIC_DIR=/app/static \
    BROWSERGAME_DATA=/data/leaderboard.json \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=8090

RUN mkdir -p /data && chown app:app /data
VOLUME ["/data"]
USER app
EXPOSE 8090

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8090/api/health || exit 1

CMD ["browsergame-server"]
