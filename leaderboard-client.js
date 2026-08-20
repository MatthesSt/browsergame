/**
 * Leaderboard client for the browser game.
 *
 * Keeps a websocket to the server so the board stays live, and falls back to REST
 * when the socket is down, so a submitted score is never lost to a dropped
 * connection. Nothing here touches the game itself - wire it up with:
 *
 *   const board = new Leaderboard();
 *   board.onBoard((entries) => renderYourTable(entries));
 *   board.setName("Gandalf");
 *   board.submit(save.bestStored, game.waveNumber);
 *
 * The player identity lives in localStorage: a random id that owns the score, plus
 * a display name the player can change at any time without losing their place.
 */
(function (global) {
    "use strict";

    const IDENTITY_KEY = "browsergame-player";
    const RECONNECT_MIN_MS = 1000;
    const RECONNECT_MAX_MS = 30000;

    function randomPlayerId() {
        if (global.crypto && typeof global.crypto.randomUUID === "function") {
            return global.crypto.randomUUID().replace(/-/g, "");
        }
        // Older browsers: still 32 hex characters, just a weaker source.
        let id = "";
        while (id.length < 32) id += Math.floor(Math.random() * 16).toString(16);
        return id;
    }

    function loadIdentity() {
        try {
            const stored = JSON.parse(global.localStorage.getItem(IDENTITY_KEY) || "{}");
            if (typeof stored.playerId === "string" && stored.playerId.length >= 8) {
                return { playerId: stored.playerId, name: stored.name || "Wanderer" };
            }
        } catch {
            // Corrupt or blocked storage just means a new identity.
        }
        const identity = { playerId: randomPlayerId(), name: "Wanderer" };
        saveIdentity(identity);
        return identity;
    }

    function saveIdentity(identity) {
        try {
            global.localStorage.setItem(IDENTITY_KEY, JSON.stringify(identity));
        } catch {
            // Not being able to remember the player is survivable.
        }
    }

    class Leaderboard {
        /**
         * @param {object} [options]
         * @param {string} [options.baseUrl] API root; defaults to the page's own origin,
         *                                   which is right when the server serves the game.
         */
        constructor(options = {}) {
            // Opened straight off disk there is no origin to talk to, so fall back to
            // where the server runs locally. Served by that same server (the normal
            // case), the page's own origin is used and nothing needs configuring.
            const pageOrigin = global.location.protocol === "file:"
                ? "http://localhost:8090"
                : global.location.origin;
            const origin = options.baseUrl || pageOrigin;
            this.httpBase = `${origin.replace(/\/$/, "")}/api`;
            this.wsUrl = `${this.httpBase.replace(/^http/, "ws")}/ws`;

            this.identity = loadIdentity();
            this.entries = [];
            this.total = 0;
            this.socket = null;
            this.reconnectDelay = RECONNECT_MIN_MS;
            this.reconnectTimer = null;
            this.closed = false;
            this.boardListeners = new Set();
            this.statusListeners = new Set();
            this.lastAck = null;
        }

        get playerId() {
            return this.identity.playerId;
        }

        get name() {
            return this.identity.name;
        }

        setName(name) {
            const trimmed = String(name || "").trim().slice(0, 24);
            if (!trimmed || trimmed === this.identity.name) return;
            this.identity.name = trimmed;
            saveIdentity(this.identity);
        }

        /** Called with (entries, total) every time the board changes. */
        onBoard(listener) {
            this.boardListeners.add(listener);
            if (this.entries.length) listener(this.entries, this.total);
            return () => this.boardListeners.delete(listener);
        }

        /** Called with "connecting" | "live" | "offline". */
        onStatus(listener) {
            this.statusListeners.add(listener);
            return () => this.statusListeners.delete(listener);
        }

        connect() {
            if (this.closed || this.socket) return;
            this.emitStatus("connecting");

            let socket;
            try {
                socket = new WebSocket(this.wsUrl);
            } catch {
                this.scheduleReconnect();
                return;
            }
            this.socket = socket;

            socket.addEventListener("open", () => {
                this.reconnectDelay = RECONNECT_MIN_MS;
                this.emitStatus("live");
            });

            socket.addEventListener("message", (event) => {
                let message;
                try {
                    message = JSON.parse(event.data);
                } catch {
                    return;
                }
                if (message.type === "leaderboard") {
                    this.entries = Array.isArray(message.entries) ? message.entries : [];
                    this.total = Number.isFinite(message.total) ? message.total : this.entries.length;
                    for (const listener of this.boardListeners) listener(this.entries, this.total);
                } else if (message.type === "ack") {
                    this.lastAck = message;
                } else if (message.type === "error") {
                    console.warn("leaderboard:", message.error);
                }
            });

            const drop = () => {
                if (this.socket !== socket) return;
                this.socket = null;
                this.emitStatus("offline");
                this.scheduleReconnect();
            };
            socket.addEventListener("close", drop);
            socket.addEventListener("error", drop);
        }

        close() {
            this.closed = true;
            if (this.reconnectTimer) global.clearTimeout(this.reconnectTimer);
            if (this.socket) this.socket.close();
            this.socket = null;
        }

        /**
         * Sends a score. Only the player's best is kept server-side, so calling this
         * with a worse run is harmless.
         * @returns {Promise<object|null>} the ack when it went over REST, else null.
         */
        async submit(score, wave = 0) {
            const payload = {
                type: "submit",
                player_id: this.identity.playerId,
                name: this.identity.name,
                score: Math.max(0, Math.floor(score || 0)),
                wave: Math.max(0, Math.floor(wave || 0))
            };

            if (this.socket && this.socket.readyState === WebSocket.OPEN) {
                this.socket.send(JSON.stringify(payload));
                return null;
            }

            // No socket: post it, so a score is never lost to a dead connection.
            try {
                const response = await fetch(`${this.httpBase}/scores`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify(payload)
                });
                if (!response.ok) {
                    console.warn("leaderboard: submit rejected", await response.text());
                    return null;
                }
                return await response.json();
            } catch (err) {
                console.warn("leaderboard: submit failed", err);
                return null;
            }
        }

        /** One-off read, for when you do not want a live socket at all. */
        async fetchBoard(limit = 20) {
            const response = await fetch(`${this.httpBase}/leaderboard?limit=${limit}`);
            if (!response.ok) throw new Error(`leaderboard: HTTP ${response.status}`);
            const board = await response.json();
            this.entries = board.entries || [];
            this.total = board.total || this.entries.length;
            for (const listener of this.boardListeners) listener(this.entries, this.total);
            return this.entries;
        }

        scheduleReconnect() {
            if (this.closed || this.reconnectTimer) return;
            const delay = this.reconnectDelay;
            this.reconnectTimer = global.setTimeout(() => {
                this.reconnectTimer = null;
                this.connect();
            }, delay);
            // Back off so a server that is down does not get hammered by every tab.
            this.reconnectDelay = Math.min(delay * 2, RECONNECT_MAX_MS);
        }

        emitStatus(status) {
            for (const listener of this.statusListeners) listener(status);
        }
    }

    global.Leaderboard = Leaderboard;
})(window);
