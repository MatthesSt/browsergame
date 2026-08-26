# Working in this repo

Meadow Gatherer: a browser game in one dependency-free `index.html`, plus a small
Rust/Rocket leaderboard backend in `server/`. See `README.md` to run it.

| Path | What it is |
| --- | --- |
| `index.html` | The whole game - markup, CSS and ~12k lines of JS in one IIFE |
| `leaderboard-client.js` | Optional board transport. The game runs fine without it |
| `server/src/` | Rocket: REST, websocket, board persistence, static hosting |
| `TESTING.md` | Per-system checklist of everything the game is supposed to do |

## Every bug found gets written into TESTING.md

**This is the rule that matters.** A bug is not finished when the code is fixed. It is
finished when `TESTING.md` has an item that would have caught it.

The moment a defect is confirmed - whether you found it, a tester reported it, or it
turned up while reading - add a checklist item **in the section for the system it lives
in** (weather bugs go in §15, save bugs in §1, and so on). Not a separate changelog: the
item has to be somewhere a person testing that system will actually read it.

Do this **as part of the fix, in the same turn**, before reporting the work as done.

### What an entry has to contain

Four things. An entry missing any of them will not catch a recurrence:

1. **The invariant, stated positively** - what must be true, in bold. Not "check X isn't
   broken"; the thing a tester can verify.
2. **The concrete number or condition** that makes it checkable.
3. **The root cause**, one clause, prefixed `Regression:` - so a future reader knows what
   shape of mistake to look for rather than only this instance of it.
4. **Why a naive check would miss it** - the sampling trap. This is the part that gets
   skipped and the part that does the work.

Real example, from the weather tint bug:

```markdown
- [ ] **The tint eases in and out over 120 ticks at each end, whatever duration rolled.**
      Alpha must start at 0 and ramp by 1/120 a tick - never snap on in a single frame.
      Regression: the fade-in was anchored to `WEATHER_MAX_TICKS` instead of the rolled
      duration, so ~9 rolls in 10 arrived at full strength instantly and the view appeared
      to jump. Watch several onsets, not one - the old bug was correct for long rolls.
```

That last sentence is the point of the whole entry: checking one onset had a 1-in-10
chance of looking fine.

### Also

- If the bug came from a class of mistake rather than a one-off - iteration order,
  temporal dead zone, missing-vs-empty - add it to **§23 Fragile spots** too.
- Fixing a bug that an existing item *should* have caught means that item was too weak.
  Sharpen it rather than adding a second one beside it.
- Numbers that are part of the contract get written down, so "is this a bug or is it
  tuned that way" has an answer without reading the source.
- A tester's description of *what moved* is a symptom, not a diagnosis. Reproduce and
  measure before fixing - the weather bug was reported as a zoom problem and the camera
  turned out never to change.

A `Stop` hook (`.claude/hooks/record-regression.sh`) points out any turn that changed game
code without touching `TESTING.md`. It cannot tell a fix from a feature, so it only
reminds - it is a backstop for this rule, not the rule itself.

## Style

- Comments explain *why*, especially where a value was tuned or a bug was designed out.
  Match the density already in the file; it is high, and deliberately so.
- `index.html` has no build step. Keep it that way.
- Prefer a named constant with a comment over a magic number.
