# Meadow Gatherer

A browser game, plus a small Rust/Rocket backend that keeps a leaderboard.

The whole game is `index.html` - no build step, no dependencies. `leaderboard-client.js`
is the only other client file, and it is optional.

## Playing

```sh
cd server && cargo run          # needs: . "$HOME/.cargo/env"
```

Then open <http://localhost:8090>. The server hosts the game and the leaderboard from
one origin, so nothing needs configuring.

`index.html` also runs on its own by double-clicking it. With no server reachable the
leaderboard is simply absent - the tab does not appear at all - and everything else
plays exactly the same.

## The game

You walk a meadow picking up materials, carry them into the tower at the centre to bank
them, and spend what you bank on the things that gather and fight for you. Waves come
for the tower on a timer; the run ends when it falls.

- **Six materials** - wood, stone, crystal, amber, aether and ivory - grow in fields at
  fixed distances from the tower. The further out, the more it is worth.
- **Essence** is the meta currency. It comes from kills and survives the run.
- **The skill tree** is bought with banked materials and carries across runs. Everything
  else the run builds is lost when the tower falls.
- **Minions** come in three roles: gatherers work the fields, traders run goods to the
  village and turrets back from the city, defenders hold ground you give them.
- **Turrets** are bought only by a caravan to the city, then placed by hand. They can be
  upgraded and given a firing order.
- **Across the river** is the tribe's ground, reachable once you build the boat. Its four
  ivory fields have to be taken and held; hold all four at once and their war-chief comes.
- **Every run varies**: one run modifier rolled at the start, a draft of boons every five
  waves, an event every seven, a day/night cycle, and weather that is announced before it
  arrives.

### Controls

| | |
| --- | --- |
| `W` `A` `S` `D` / arrows | move (walking into an enemy attacks it, at a stamina cost) |
| `P` / `Esc` | pause - `Esc` closes any open panel first |
| `F` | cycle game speed 1x / 2x / 3x |
| `T` | toggle tips |
| `M`, `+` / `-` | mute, volume |
| `[` `]` `0` | zoom out, in, reset |
| drag / wheel | pan and zoom the map |
| left-click | place a queued turret, or upgrade the turret under the cursor |
| shift+left-click | cycle a turret's firing priority |
| right-click | cancel what is queued, or sell the turret under the cursor |

The header carries Start Run, Pause, game speed, settings, the Skill Tree and Reset Save.
The panel on the right is the Field Shop, with the Leaderboard as its second tab.

## Leaderboard

- The **Leaderboard** tab sits beside **Field Shop** in the right-hand panel. It appears
  only while a server is reachable, and the game keeps running while you read it.
- Your best banked progress is what counts. It is reported when a run ends, when you open
  the board, and periodically while playing - only ever when you beat your own record.
- Set a display name in the panel; renaming keeps your place on the board.
- Scores are taken on the client's word - see `server/README.md`.

See `server/README.md` for the API, the websocket protocol and configuration.

## Testing

`TESTING.md` is a per-system checklist of everything the game is supposed to do, with the
numbers that are part of the contract written down. Walk it before a release, or dip into
the relevant section after a change.
