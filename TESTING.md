# Feature list / test checklist

Every player-visible behaviour in the game, in the order you would exercise it. Written
to be walked top to bottom before a release, or dipped into per-system after a change.

Each line is one observable thing. Where a number is part of the contract it is written
down, so "is this a bug or is it tuned that way" has an answer without reading the source.

Source of truth: `index.html` (whole game), `leaderboard-client.js` (board transport),
`server/src/` (board persistence). Numbers below are ticks unless stated; **60 ticks = 1
second at 1x**.

**Every bug found gets an item here**, in the section for the system it lives in, as part
of the fix. An entry states the invariant positively, gives the number that makes it
checkable, names the root cause after `Regression:`, and says why a naive single check
would miss it - see `CLAUDE.md` for the format and a worked example. Items marked
`Regression:` are bugs that have actually happened; they are the ones worth re-reading.

---

## 0. Smoke test - the five-minute pass

If time is short, this is the subset that catches the failures that have actually shipped.

- [ ] Load with an **empty** localStorage: fresh save, no console errors, Start Run works.
- [ ] Load with an **existing** save mid-run: the run resumes where it was (see §12).
- [ ] Load with a **corrupt** save (`localStorage['meadow-gatherer-save-v1'] = '{'`): falls
      back to a fresh save without throwing, and does not silently wipe a *valid* save.
- [ ] Gather → bank → open Skill Tree → buy a skill → the cost leaves the bank and
      **Prog drops by the material value of the cost**.
- [ ] Craft a gatherer, watch one full round trip (node → tower → node).
- [ ] Survive to wave 2, take the wave-5 draft, let the tower fall, confirm the run
      summary and that essence/skills survived.
- [ ] Reload after each of the above: nothing lost, nothing duplicated.

---

## 1. Boot, save and reset

### Save file
- [ ] Key is `meadow-gatherer-save-v1`; backup is `meadow-gatherer-save-v1-backup`.
- [ ] Save is written on every purchase, craft, placement, sale and run end.
- [ ] `beforeunload` captures a runtime snapshot **only if a run has started**.
- [ ] **UI preferences survive a run ending**: tips, audio, and which shop groups are
      collapsed all sit outside `freshRunState()`, next to the meta counters. Losing them
      on every defeat would make setting them pointless.

### Migrations (regression-heavy - all three have broken saves before)
- [ ] **Legacy tower keys**: a save with `depotHp`/`depotPulse` in `runtimeState` loads,
      and minions in state `toDepot` walk again (remapped to `toTower`).
      `isRegeneratingAtDepot` → `isRegenerating`.
- [ ] **Legacy coordinates**: a save without `coordOrigin: "tower"` has every stored point
      shifted once and is stamped, so a second load does not shift it again.
- [ ] **Legacy turret placements**: a save with `turrets: 5` and **no** `turretPlacements`
      array force-places 5 turrets on the old ring. A save with `turretPlacements: []` and
      `turrets: 5` must leave all 5 **in the store** - not place them. (Missing ≠ empty.)
- [ ] Turret placements are clamped to the current cap and to the map edges on load.
- [ ] Turret `level` clamps to 0-5, `priority` falls back to `closest` if unrecognised.

### The settings menu (top right)

- [ ] The gear button at the top right opens a panel holding **Restart Run** and
      **Reset Save**, each with a sentence saying what it costs. There is no bare
      `Reset Save` button in the top bar any more.
- [ ] The panel closes on a click outside it, on `Esc`, and after either action is chosen.
      A click *inside* it must not close it.
- [ ] The panel is anchored to its **right** edge. It sits at the end of the bar, so a
      panel hanging off the left corner would run off the side of the window - check at a
      narrow window width, not just a maximised one.
- [ ] **The two descriptions state the actual difference**: Restart Run keeps the skill
      tree, essence and records; Reset Save wipes them and keeps only an undo backup.
      This is the entire reason they were moved behind one menu - as two words in a top bar
      they were indistinguishable.
- [ ] Both still route through their existing confirm popovers - neither acts on one click.

### The primary button

- [ ] **Mid-run the Start button is hidden, not relabelled.** Start a run and confirm the
      top-left button disappears, leaving Pause as the run control; end the run and confirm
      it returns as `New Run`; reset the save and confirm it returns as `Start Run`.
      Regression risk: it used to relabel itself to `Restart Run` while a run was live,
      which put the one irreversible in-run action exactly where the eye goes for "start".
      Its label was assigned from four scattered places (`beginRun`, `endRun`,
      `performReset`, the snapshot restore); it is now decided only by
      `refreshRunControls()`. If a fifth caller ever sets `startBtn.textContent` directly,
      the label and the state can disagree again.
- [ ] **Restart Run in the menu is disabled before a run starts and after one ends**, and
      the row dims with it. The primary button is the way into both of those states.
      Check on a **fresh save** (never started) as well as after a defeat - testing only
      the after-defeat case misses the never-started one, and they reach the disabled state
      by different fields (`game.started` vs `game.runOver`).
- [ ] Opening the menu refreshes its state immediately rather than waiting for the next
      4-tick HUD pass - lose a run with the menu already open, reopen it, and Restart must
      already read as unavailable.

### Reset and undo
- [ ] `Reset Save` opens a confirm popover listing what will be lost; Esc and a
      backdrop click both close it without resetting.
- [ ] Confirming writes the backup, wipes the save, resets the session, and shows the
      undo toast.
- [ ] `Undo` restores the backup and reloads the page; the restored save is complete.
- [ ] `Dismiss` hides the toast; the backup still exists (undo via reset is one-shot only
      because the backup is removed on undo).
- [ ] Reset with storage blocked/full still resets - just without an undo.

---

## 2. Controls

### Keyboard
- [ ] `W`/`A`/`S`/`D` and arrow keys move the player (8-way, diagonals normalised).
- [ ] `P` or `Esc` toggles pause.
- [ ] `Esc` closes, in priority order: bestiary → skill tree → restart-run popover →
      reset popover → settings menu → audio panel. Only if none is open does it pause.
- [ ] `F` cycles game speed 1x → 2x → 3x → 1x.
- [ ] `T` toggles tips.
- [ ] `M` toggles music mute.
- [ ] `+`/`=` and `-`/`_` nudge volume by 10%.
- [ ] `[` zooms out, `]` zooms in, `0` resets map zoom to 1.
- [ ] `1` opens the bestiary; `2`-`7` cast the six spells. `8` and `9` are undefined
      slots and must fall through and do nothing rather than being swallowed.
- [ ] **`0` still resets map zoom.** The toolbar takes `1`-`9` and stops there - there is
      no tenth slot, because `0` was already bound. Press `0` with the toolbar present and
      confirm the zoom snaps to 1x rather than the keypress being eaten.
- [ ] **None of the above fire while typing** in the name field or a defender rename box.
- [ ] Losing window focus (`blur`) clears held keys - the player must not walk forever.
- [ ] Key handling is case-insensitive.

### Mouse
- [ ] Left-drag on the canvas pans the camera; a drag of >3px suppresses the click that
      follows, so panning never places or upgrades anything.
- [ ] Wheel zooms toward the cursor, clamped to 0.25x-1x.
- [ ] With a turret queued: left-click places it.
- [ ] With nothing queued: left-click on a turret **upgrades** it (paying the cost);
      shift+left-click **cycles its firing priority**.
- [ ] Right-click cancels a pending defender placement first, then disarms a selected
      turret type (keeping it in the store), then sells the turret under the cursor.
- [ ] Right-click never opens the browser context menu over the canvas.
- [ ] The placement/sell preview follows the pointer and clears on `pointerleave`.
- [ ] With the game paused, moving the pointer still repaints the preview.

### Touch
- [ ] Tap-and-hold moves the player toward the touch point.
- [ ] With a turret queued, a tap places it instead of moving the player.

---

## 3. Player, bag and banking

- [ ] Base speed 3.5, +0.4 per Player Speed level, +0.5 per `Fleet Foot` boon.
- [ ] **Unlocking `speed` in the skill tree grants level 1 with it**, so the purchase is
      felt on the click: `speedLevel` 0 -> 1, speed 3.5 -> 3.9, the shop reads `Lv 1`, and
      `game.player.speed` updates immediately rather than at the next run.
      Regression: the unlock only revealed the shop button, so a tester who bought
      "Unlock Player Speed" correctly reported buying a speed upgrade and getting nothing.
- [ ] **The granted level survives into every later run.** `speedLevel` is run state and
      the skill is meta, so the grant is a floor re-applied by `freshRunState()`, not a
      one-off bump. Check run **two**, not just the unlock: bumping only at unlock time
      passes every check made in the first minute and reverts the moment a run ends.
      The mechanism is `SKILL_UNLOCK_LEVELS` - see §6.
- [ ] Levels bought on top are run state and *do* reset - after a new run, a player who
      bought up to Lv 4 is back to Lv 1, not Lv 0 and not Lv 4.
- [ ] A player who never unlocked `speed` stays at level 0 and base speed 3.5 forever.
- [ ] Bag capacity 10, +4 per Bag level, +4 per `Deep Bags` boon.
- [ ] **Unlocking `bag` grants level 1**: capacity 10 -> 14 on the click, and the floor is
      re-applied every run (see §6).
- [ ] Walking over a node picks it up; a full bag picks up nothing.
- [ ] **Pickup reach** is `player.r + node.size * 0.6 + PLAYER_PICKUP_GRACE` (~44 units,
      an 87-unit corridor). Walking *near* a node collects it; the grace is half a player
      radius of slack past the point the silhouettes touch, so it must not read as a
      vacuum - a node a full body's width to the side is still missed.
- [ ] The grace applies to the **player only**. Gatherers walk to a node's exact position,
      so their gather rate must be unchanged by it.
- [ ] No tunneling at max speed: the player moves ~11.5/tick at speed level 20 against an
      87-unit corridor, so a node can never be stepped over.
- [ ] Walking into the tower banks the whole bag and clears it.
- [ ] Banking adds `value x resourceMultiplier` to Prog. Values: wood 1, stone 2,
      crystal 4, amber 6, aether 11, ivory 24.
- [ ] **Prog goes down when you spend.** Buying anything subtracts the material value of
      its cost from Prog; selling/refunding adds it back. Prog is clamped at 0.
- [ ] Essence is *never* counted in Prog (it is meta currency).
- [ ] `Best run` on the HUD tracks the highest Prog ever *held*, not total ever gathered.
- [ ] **Attacking is contact-based** - there is no attack key. Walking into an enemy
      knocks it back 18 units and deals `minionStrikeDamage()`, costing 18 stamina.
- [ ] Stamina: max 100, regen 0.2/tick, refilled at run start. Below 18 the contact hit
      does not land at all (and the enemy is not knocked back). The stamina ring renders.
- [ ] Stamina is captured in the runtime snapshot and clamped to 0-100 on restore.
- [ ] Player cannot walk into the river (§9) or through the tower footprint.

---

## 4. Resource fields (spawners)

- [ ] 8 meadow fields: wood and stone on tier 1 (close), 2x crystal and 2x amber on
      tier 2, 2x aether on tier 3 (north/south, the long haul).
- [ ] 4 ivory fields on the tribe's ground across the river.
- [ ] Respawn base ticks: wood 240, stone 320, crystal 460, amber 540, aether 1200,
      ivory 1800.
- [ ] Spawner upgrades speed up wood/stone/crystal/amber only.
- [ ] **Unlocking a spawner skill grants level 1** of that spawner: respawn drops 4%
      (wood 240 -> 230 ticks) the moment it is bought, and the floor returns every run.
- [ ] The **node cap is unchanged at spawner level 1** and first moves at level 2. That is
      the existing rounding, not a broken grant: the cap is
      `round(11 x baseRespawn / currentRespawn)`, and `round(11 x 240/230) = round(11.48)`
      is still 11. The ladder runs 11, 11, 12, 13, 13, 14, 15 for levels 0-6. Do not
      "fix" it by rounding up - that would raise every field's cap at every level.
- [ ] Aether has no unlock, so it gets no grant: `spawnerAetherLevel` stays 0.
- [ ] Ivory is untouched by all of this - flat 1800-tick respawn, no level, no grant.
- [ ] **Ivory never speeds up and never gets a multiplier** - not from spawner levels, not
      from Global Multiplier, not from run modifiers. Check both banking paths.
- [ ] Aether has no shop spawner upgrade.
- [ ] Node cap: 90 across the map, divided per meadow field; `nodeCapScale` modifiers
      change it.
- [ ] Nodes never spawn inside the river or inside the tower.
- [ ] The `Fields` HUD chip counts **meadow** fields still producing - tribe ground never
      counts against it.

---

## 5. Shop and upgrades

For each: the button shows its cost, disables when the bank cannot cover it, hides
entirely until its skill is unlocked, and buying it deducts exactly the quoted cost.

- [ ] **Player**: Upgrade Speed, Upgrade Bag.
      Speed adds amber from level 15; bag adds quadratic crystal from level 10.
- [ ] **Economy**: Wood / Stone / Crystal / Amber Spawner, Global Multiplier
      (+10% per level, costs essence only).
- [ ] **Turrets**: Turret Store (buy via caravan), Turret Capacity (+3 per level from a
      base of 15). The group header shows `owned/cap` - owned counts turrets still in the
      store as well as placed ones, so buying one and not placing it still moves the
      number. It read `placed/cap` here, which is what the counter behind it used to be
      called; it never counted placements.
- [ ] **Minions**: Craft Gatherer, Craft Trader, Craft Defender, Minion Speed, Minion Load
      - five controls, all of which spend something.
- [ ] **Standing Orders**: Gatherer Focus, Trade Routes, Defender Posts. Nothing in this
      group costs anything; they are instructions you set once. There is **no** Far Bank
      panel - the gatherer's focus decides which bank it works (see §7).
      Regression risk: these three used to sit inside Minions, which meant that group
      carried crafting, directing and upgrading at once and held eight of the panel's
      seventeen controls. If a future control is added, put it with the others that do the
      same *kind* of thing, not with the ones that happen to concern the same unit.
- [ ] The Standing Orders group is **hidden entirely** until at least one of its three
      panels is unlocked, so an early run does not show an empty header.
- [ ] **"Minion Load" reads `Lv N carries M`**, where M is `1 + level`. It must not be
      called Capacity: it sits under "Turret Capacity", which *is* a maximum count, and
      the same word for two different things had testers reading it as a limit on how many
      minions they could own. It also raises trader cargo (`24 + level * 6`).
- [ ] **There is no Spells group in the shop.** All six moved to the map toolbar (§18).
      The shop is stock and upgrades; the spells were never either.
- [ ] Every recipe scales with the count already owned - crafting the 4th gatherer costs
      more than the 3rd.

### Reading the panel at a glance

- [ ] **A tile you cannot currently pay for is dimmed.** Bank nothing, open the shop, and
      everything you cannot afford sits at ~42% opacity; bank enough for one and it comes
      back to full on the next HUD pass. It must **dim, not disappear** - the upgrade you
      are saving towards is the one you most need to keep reading.
      Regression: the shop's only greyed state was *locked* (`setButtonDisabled(btn,
      !unlocked)`), and locked controls are hidden anyway - so once most upgrades were
      unlocked, all seventeen visible tiles looked equally live and the only way to tell
      what you could buy was to read six cost chips against the bank strip, seventeen
      times. The skill tree had always done this properly, which is why the shop's version
      was never missed. Test **late**, with most things unlocked and a thin bank; early on
      almost everything is locked and hidden, so the panel looks fine either way.
- [ ] Dimming is a **signal, not the guard**. A dimmed tile is still clickable and clicking
      it does nothing, because every buy handler re-checks `hasBankForCost` for itself.
      Do not make these `disabled` - Chrome sends a disabled button no mouse events (§24).
- [ ] The dimmed state uses the **same cost object** the tile was rendered with, so a tile
      can never show one price and judge affordability by another. Check a stacking cost -
      craft a gatherer until you cannot afford the next, and the tile that dims is the one
      quoting the raised price.
- [ ] **The four expanding panels do not look like purchase tiles.** Turret Store, Gatherer
      Focus, Trade Routes and Defender Posts have a flatter fill, a dashed edge and a
      **chevron** (▸/▾). They carry no cost chips and spend nothing when clicked.
      Regression: all four carried `class="secondary shop-btn"`, making them pixel-identical
      to a tile that spends resources, and their marker was a `+` - which on a shop tile
      reads as "buy one more" rather than "this expands".

- [ ] **A collapsed shop group is still collapsed after a reload.** Collapse Economy and
      Minions, refresh the page, and both must come back collapsed - and Player and Turrets
      must still be open. State lives in `save.shopGroupsOpen`, keyed by the `data-group`
      attribute on each `<details>`.
      Regression: the `open` attribute was hardcoded in the markup and nothing persisted the
      toggle, so collapsing a group lasted exactly as long as the tab did. This item existed
      and read "open/close and keep their state", which is why it passed: within one session
      the `<details>` element keeps its own state with no help, so the only way to see the
      bug is to **reload**. Test across a refresh, not by clicking twice.
- [ ] A group collapsed in an older save (or a brand new one) opens: missing means open, so
      nothing that predates this setting comes back mysteriously shut.
- [ ] The state is **meta** - it survives a run ending and a Restart Run. Collapse a group,
      let the tower fall, and it is still collapsed. Only `Reset Save` reopens them.
- [ ] **`Reset Save` reopens every group immediately, not just in the save file.** Collapse
      all four, reset, and the panel itself must show four open groups without a reload.
      `performReset()` swaps `save` wholesale, so the panel has to be told; a check that
      only reads localStorage afterwards would pass while the visible panel stayed wrong.
- [ ] Collapsing a group writes the save once, not on every animation frame - the `toggle`
      handler ignores events whose value already matches what is stored, which is what makes
      the restore on load a no-op rather than a second write.

---

## 6. Skill tree (27 skills)

- [ ] Opens from the header button; pauses the game; closing resumes it **only if it was
      running when opened**.
- [ ] Esc and a backdrop click both close it.
- [ ] Bank strip inside the popover matches the HUD bank exactly, essence included.
- [ ] Zoom buttons and wheel work, 0.5x-1x; drag pans; a drag does not click a node.
- [ ] A node is buyable only when every `requires` entry is already unlocked and the bank
      covers the cost.
- [ ] Buying deducts the cost, redraws the node as owned, and reveals the matching shop
      buttons immediately.
- [ ] Link lines connect each node to its prerequisites.
- [ ] **Every node shows its cost until it is owned** - locked ones included, so a route
      through the tree can be priced before it is bought into. Only unlocked nodes drop it.
- [ ] **All 27 nodes carry a hint** in the tooltip, above the tier/status/cost lines.
      Where the node buys the *right* to do something rather than the thing itself (all
      three turret types), the hint has to say so.
- [ ] **No two node boxes overlap at 1600x1000 or wider.** Showing costs made every node a
      row taller (52px -> 74px), and the economy strand had exactly 3px of vertical
      clearance at the old 20-degree angle; it runs at 32 now to buy the room vertically
      rather than by spreading the rings, which would have grown the whole tree.
      Regression: check after **any** change to node contents, font, or padding - this
      layout is tuned to within a few pixels and the failure is silent.
- [ ] Known, pre-existing: at **1440x900 and below** the defense strand overlaps
      (Defenders/Slings/Catapults/Slow Field/Turret Capacity). Present before the cost
      change and unrelated to it - node widths clamp at 128px while the map layer keeps
      shrinking. Compare against `git stash` before blaming a new change for it.
- [ ] Prerequisite chains behave (spot-check the awkward ones):
      `minionSpeed` needs **both** `gatherer` and `trader`;
      `gathererFocus` needs `minionCapacity`;
      `warPaint` needs `boat`; `mendSpell` needs `chillSpell` needs `minionHaste`;
      `turretHaste` and `turretCap` both need `slowTurret`.
- [ ] Skills and essence survive a run ending. Everything else does not.
- [ ] A node that carries a granted level says so in its description, so the player knows
      before buying, not only after.

### Unlock-granted levels (`SKILL_UNLOCK_LEVELS`)

One table drives this. A row is `skillId: { field, level, sync? }`, and adding one must be
the **only** change needed - if a new grant needs edits anywhere else, the mechanism has
regressed. Re-check the whole block after adding a row.

Granted today: `speed`, `bag`, and the four spawners (`spawnerWood`, `spawnerStone`,
`spawnerCrystal`, `spawnerAmber`) - all at level 1. Only `speed` needs a `sync`; the rest
are read fresh every time (`bagCapacity()`, `scaledRespawnTicks()`) or self-invalidate
(`activeNodeCapForType` keys its cache on the level).

- [ ] `applyUnlockLevels()` is applied in all four places run state is built: the unlock
      click, `freshRunState()` (so `startRun` and `endRun` keep it), the `loadSave()`
      success path, and the unreadable-save fallback. Missing any one of them means the
      grant survives some transitions and not others.
- [ ] **The floor only ever raises.** A player above it keeps the level they paid for -
      `speedLevel: 7` stays 7, it is not clamped to the granted 1.
- [ ] `sync` is optional, runs **only** on the unlock click, and never touches `game` from
      `applyUnlockLevels` - `loadSave()` calls that long before there is a `game`.
- [ ] **A save that unlocked the skill before its grant existed gets the level on load**,
      and everything else in it survives - `bestRunScore`, `essence`, `bagLevel`, the lot.
- [ ] **The typo guard fires.** A row naming a field that is not on the save, a `skillId`
      that is not in `SKILL_TREE_DEFS`, or a `level` below 1 logs a `console.error` naming
      the row at boot. Without it a misspelled row writes a save key nothing reads and
      grants nothing, silently. Break a row on purpose once and confirm you see it.
- [ ] Regression risk: `loadSave()` applies these floors and runs while the module body is
      still initialising, so `SKILL_UNLOCK_LEVELS` must stay declared **above**
      `const save = loadSave()`. Below it, the const is in its temporal dead zone, the
      throw is swallowed by loadSave's `catch`, and the save is silently wiped. Verify by
      loading a real save and checking `meadow-gatherer-save-v1-unreadable` was **not**
      written. See §24.

---

## 7. Minions

Shared: crafted from the bank, occupy a slot, respect the river, are hit by lightning,
and are killed by ghouls/hunters/the war chief.

- [ ] Minion HP: base 1, x2 with `Minion Armor`, x4 with `Minion Plating`, x1.5 per
      `Iron Posts` boon.
- [ ] Minion carry capacity: 1 + Minion Capacity level.
- [ ] Minion speed scales with Minion Speed level; Haste multiplies by 1.5 for 60s.

### Gatherers
- [ ] States cycle idle → toNode → toTower → idle; a gatherer with a full bag walks home.
- [ ] **Gatherer Focus**: `auto` plus one per resource. A focused gatherer only picks its
      resource; `auto` takes the nearest.
- [ ] **Ivory is not offered until `boat` is unlocked** - six options before, seven after,
      and the list refreshes the moment the skill is bought without needing another redraw.
      Ivory grows only on the tribe's ground, so without the ferry there is none to walk to.
- [ ] An ivory order that arrives anyway is coerced to `auto` - including one restored from
      a save written while the option was still on offer.
      Regression: the focus used to be accepted and then silently ignored (the gatherer
      falls back to the nearest node of anything), so the row read "Ivory" while the minion
      hauled wood. A stalled minion would have been obvious; this was not.
- [ ] **Asking for ivory is the whole instruction.** With `boat` unlocked and at least one
      tribe field taken, a gatherer set to Ivory walks to the jetty, crosses, works the far
      fields and ferries each load home on its own. There is no second posting to set.
      Regression: this used to need a separate Far Bank assignment as well, so two controls
      described the same thing and the focus quietly lost to the posting.
- [ ] **It falls back to the near bank whenever there is no ivory to be had** - fields still
      the tribe's, nodes eaten by a blight, or the river up in the rain. It works the meadow
      like any other gatherer rather than waiting at the jetty for a crossing that pays
      nothing. Check all three causes; they take different routes through the code.
- [ ] A full round trip banks ivory in roughly 11,000 ticks (~3 minutes at 1x) from a
      standing start at the tower. If it never arrives, check the run is actually running
      before blaming the pathing - see §24.
- [ ] Re-pointing a gatherer mid-crossing does not strand it - it finishes the crossing
      and turns around on the far jetty.
- [ ] A focus/far-bank choice is stored per slot and survives reload.
- [ ] Opening a picker and clicking elsewhere does **not** commit a change.

### Traders
- [ ] A fresh trader starts on route `none` and does nothing until given one.
- [ ] **A trader with no route says so.** The map draws `no route set` over it, and the
      collapsed panel summary reads `Trade Routes (N, 1 idle)`. Both cues clear the moment
      a route is assigned. A caravan is expensive and standing still looks identical to
      working, so silence here reads as a purchase that did nothing.
- [ ] The night label still wins over the route label: a routed caravan waiting out the
      dark says `market shut`, not `no route set`.
- [ ] 9 routes: 6 village trades (wood→crystal, stone→crystal, wood→amber, stone→amber,
      crystal→aether, amber→aether) and 3 city turret purchases (stone→Sling,
      crystal→Catapult, amber→Slow Field).
- [ ] Village keeps a 0.7 markup cut - **no chain of trades can out-earn gathering**.
      Verify a round trip does not print money.
- [ ] A turret route requires the matching skill (`turret` / `sniperTurret` / `slowTurret`)
      and delivers **one turret into the store** per trip.
- [ ] Turret trade price rises with the number owned (base 34/26/22, step 14/12/10).
- [ ] Trader speed bonus is +1.5 (much faster than a gatherer) - the village round trip
      must stay under a few minutes.
- [ ] Dwell at a settlement is 90 ticks and the settlement pulses.

### Defenders
- [ ] Patrol radius 48-156 around their post; aggro radius 320; tower alert radius 460.
- [ ] Regen 0.05/tick; recoil damage 0.6 per kill.
- [ ] **Post placement**: clicking a row arms *that one*; ticking several then
      `Station` arms the group against **one** map click, which lays them out in a
      formation around the clicked point.
- [ ] Clicking an already-armed single row disarms it.
- [ ] `Select all` / `Select none` work; `Recall` sends the picked defenders home.
- [ ] Renaming: the box opens focused and selected, `Enter` commits, `Escape` cancels,
      a click inside the box does not fall through to the row.
- [ ] A post clicked on the water stations the defender on the **near** bank.
- [ ] Posts, names and selections survive reload.

---

## 8. Turrets

- [ ] Three types: Sling (range 230), Catapult (range 420), Slow Field
      (radius 190, x0.35 speed).
- [ ] Turrets are bought only by a trader caravan to the city and wait in the store.
- [ ] Selecting a store entry arms the next map click; right-click disarms without losing
      the turret.
- [ ] Placement is refused within 26 units of another turret, inside the tower, in the
      river, or outside the map (12-unit edge padding).
- [ ] **A caravan will not set off for a turret unless a slot is free, counting the ones
      already in carts.** With the cap at 15 and 14 owned, exactly **one** caravan leaves
      for a turret however many are standing idle at the tower.
      Regression: the guard read `placedTurretCount()`, which despite the name returned
      turrets *owned* - `save.turrets` is incremented when a caravan gets home, not when a
      turret goes on the map - and nothing counted turrets already bought and in transit.
      Every idle caravan evaluated the same `14 >= 15` on the same tick, all of them left,
      and three of them took the owned count to 17 against a cap of 15, so the turret group
      header read `17/15`. Why a naive check misses it: with **one** trader the old guard
      was correct, and one trader is how the route gets tested. Reproducing needs two or
      more traders idle at the tower, both set to a turret route, with the bank able to
      cover both orders at once.
- [ ] **Placement is refused once `game.turrets.length` reaches the cap** (base 15 +
      3/level), *even when the store still holds turrets*.
      Regression: `placePendingTurretAt()` had no cap test whatsoever - it checked only
      that the store was non-empty. Over-cap stock could therefore be placed on the map and
      was then silently deleted the next time `syncTurretsFromSave()` ran (any delivery, or
      a reload), because that only ever builds `min(cap, owned)` turrets - which reads as a
      turret being eaten rather than as a limit being enforced. This item existed before
      and did not catch it, because the cap is normally reached *by placing*, and then the
      store empties on the same click that fills the last slot - so the refusal looked
      correct for the wrong reason. To test it you need stock in hand **and** the map
      already full, which since the buy-side fix only happens on a save that went over the
      cap before it (owned can no longer exceed the cap in normal play). Force it by
      editing `save.turrets` above the cap and reloading.
- [ ] Turrets over the cap in an old save are **kept, not refunded** - they sit in the
      store and become placeable if the cap is later raised.
- [ ] **Upgrade**: left-click a placed turret, max level 5, cost 18 + 14 per level.
- [ ] **Priority**: shift+click cycles Nearest → Toughest → Closest to tower, and the
      turret actually retargets accordingly.
- [ ] **Sell**: right-click refunds and plays the sell sting; the count and the
      placement list both decrement.
- [ ] **Cancel a stored turret**: refunds exactly what the caravan paid, so a
      buy/cancel loop is worth nothing either way - **and the price on the button is the
      price you get**.
      Regression: the Turret Store's `dataset.renderKey` was built from the *stored* count,
      but the armed row prints a sell-back price computed from the *owned* count
      (`base + (owned - 1) * step`). Selling a **placed** turret drops owned and leaves
      stored alone, so the key did not change, the render early-returned, and the row kept
      quoting a price `sellStoredTurret()` would no longer pay. To reproduce you need all
      three at once: the store panel **open**, that turret type **armed**, and a *placed*
      turret of the same type sold on the map. Selling from the store instead moves both
      counts, so the key changes and the panel looks correct - which is why the obvious
      test passes.
- [ ] `Rapid Fire` skill: +30% fire rate on slings and catapults. `Keen Slings` boon:
      +20% fire rate.
- [ ] Fog halves turret range; frost widens slow fields by 1.5x.

---

## 9. The river, the crossing and the far bank

- [ ] The river seals the north-west. It meets the left edge and the top edge squarely -
      no walkable sliver around a rounded cap.
- [ ] **Enemies wade; the player and minions do not.** Walking into the water slides you
      along the bank rather than sticking.
- [ ] The water you can see is exactly the water you cannot cross (draw and collision
      share one polyline).
- [ ] Exactly one crossing, at the point the river passes closest to the tower, with a
      dock on each bank. It appears only once `boat` is unlocked.
- [ ] Boarding works within 150 units of a jetty and nowhere else. That radius is measured
      **along the bank** and the player must already be in the water, so it is not a way to
      board from dry land.
- [ ] **The ride runs waterline to waterline.** The jetty (`FERRY_DOCK_OFFSET`, 36 past
      the water's edge) is only what an entity walks *to*; the crossing itself starts and
      ends at `FERRY_BOARD_OFFSET`, 15 past the edge. Watch a gatherer cross both ways: the
      hull must appear as it steps off the deck and vanish as it reaches the far one -
      never slide over ground at either end.
      Regression: the crossing used to run jetty to jetty, so it opened *and* closed with a
      boat on dry land. Check the **landing** as well as the launch - fixing only the near
      end leaves it half wrong, which is what happened the first time.
- [ ] `FERRY_BOARD_OFFSET` has a hard floor: it must clear `RIVER_HALF_WIDTH +
      MINION_RIVER_RADIUS`, or `keepOutOfRiver` shoves minions back short of the point they
      are walking to and the crossing never starts. It currently clears it by 1.
- [ ] The board point must stay **inside the jetty deck** (`dock - 34` to `dock + 42` along
      the crossing axis), so a minion boards from the planks rather than beside them.
- [ ] `FERRY_DOCK_REACH` can be small safely - the step before it snaps an entity exactly
      onto its target once it is within one stride, so arrival never depends on the reach
      being generous. A wide reach only means the hull appears further inland.

### The player and the boat

- [ ] **A ferry is moored at each jetty while the crossing is open**, and both disappear
      when it shuts (rain). Walking to the dock and finding nothing there gave no sign the
      river could be crossed at all.
- [ ] **Boarding happens on the step that would enter the water**, not once the player is
      already in it - so the boat meets you at the end of the jetty instead of fishing you
      out of the river. Verify the player is still on dry land at the moment they board.
- [ ] **The controls are dead for the whole crossing.** Hold any direction, or several: the
      player must travel in a dead straight line to the landing.
      Regression: the ferry step ran *after* the input step, so a held key was applied and
      then partly undone every tick - you could row about mid-river and reach places the
      water exists to keep you out of.
- [ ] Landing leaves the player free again immediately, and holding the same key that
      boarded them does not bounce them straight back (on the far bank it now points
      inland).
- [ ] Crossing speed is 0.55x walking - the ferry is a toll, not a shortcut.
- [ ] A crossing entity is drawn with a hull under it, not walking on water.
- [ ] **Rain closes the crossing entirely** - check what happens to someone already on it
      and to someone stranded on the far bank.

---

## 10. The tribe and the war chief

- [ ] 4 ivory fields plus a camp on the far bank; they start owned by `tribe`.
- [ ] Standing minions within 150 units capture a field over 900 ticks; up to 3 bodies
      count, contested at 2, bleeds back at 1.
- [ ] The capture arc renders while 0 < capture < 900.
- [ ] Wardens garrison 2 per field, cap 16 alive, leash radius 520. They ignore the
      tower entirely and never count towards a wave.
- [ ] Every 2400 ticks the tribe sends a retake party at the field nearest their camp,
      sized `2 + warChiefKills/2 + fieldsHeld/2`.
- [ ] The `Tribe` HUD chip shows fields held out of 4.
- [ ] **Hold all four at once for 1800 ticks and the war chief comes.** Losing one resets
      the provoke timer to zero.
- [ ] War chief: HP 400 + 200/tier, regen 0.35 + 0.12/tier, leash 900. It out-regenerates
      an unpainted party and loses to a painted one - that is the whole fight.
- [ ] Killing it pays 40 + 20/kill essence, adds 500 x kills to Prog, and starts a
      3600-tick mourning period before another can be provoked.
- [ ] Fields stay yours after the kill (no forced recapture).
- [ ] **War Paint**: costs `12 + 6/chief kill + 4/banked duration` ivory, gives x3 damage
      and half damage taken for 2700 ticks (45s); recasting stacks another 45s onto the
      clock rather than restarting it, and the price rises with what is already banked.
- [ ] The run **cannot** end during a chief fight - the tribe never reaches the tower.

---

## 11. Waves, enemies and the tower

- [ ] Tower HP 100; the run ends at 0. Mend is the only way to heal it.
- [ ] First wave after 3600 ticks; breaks are 320 + up to 280 by wave, x`waveBreakScale`.
- [ ] Wave size: 1 at wave 1, then log-ish growth, capped at 34. Concurrent enemies
      capped at `2 + wave*0.35`, max 24.
- [ ] Spawn interval 72 falling to a floor of 24.
- [ ] Enemy HP: linear +8%/wave **and** compounding x1.015/wave, so late waves outrun any
      player power. Damage grows at only 3%/wave.
- [ ] Off-wave enemies (ghouls, blights, wardens) take the **square root** of that curve.
- [ ] An enemy keeps the strength of the wave it spawned on - it must not grow in place.
- [ ] Enemy types unlock: brute at wave 3, wisp at 5, hunter at 8, and the `Incoming`
      chip announces each one.
- [ ] **Boss every 10th wave**: HP `(10 + tier*6) x curve`, damage 16 + 2/tier, shoots
      minions for double. Music turns dark while one is alive and back when it dies or
      the run ends.
- [ ] Hunters target minions (range 72, cooldown 16) and ramp up over time.
- [ ] The `Incoming` HUD countdown matches the actual next wave.

### Blights (field claimers)
- [ ] First at wave 5, then every 3 waves, max 5 active, +1 allowance every 12 waves.
- [ ] A blight rises ~900 units from the field it wants, not from the map edge.
- [ ] It walks past the tower without attacking it.
- [ ] A settled blight eats one node per 300 ticks and the field stops producing.
- [ ] The field ring is **violet while the blight is inbound and red once claimed** -
      the two states must never look the same.
- [ ] Killing the blight frees the field and it recovers rather than popping back.
- [ ] `Warded Fields` boon slows the consumption markedly.
- [ ] Killing one pays 3 essence.

### Ghouls
- [ ] First horde at 1800 ticks, then every 3600; 3-5 per horde, 20 alive max.
- [ ] They spawn ~420 units out, roam between meadows, and never approach the tower.
- [ ] Aggro radius 160, strike range 18, attack cooldown 72 - slow enough that a minion
      can wander back out of reach.
- [ ] Each pays 1 essence. Night halves the interval between hordes.

---

## 12. Run lifecycle

- [ ] `Start Run` begins wave 1 and rolls **one run modifier**, named on the start screen.
- [ ] 7 modifiers: Fair Weather, Lean Soil, Restless Dead, Long Summer, Hard Frost,
      Rich Veins, Thin Ranks. Each one's stated effect actually applies.
- [ ] Meadow field types are reshuffled per run.
- [ ] Starting budget: 40 wood, 30 stone.
- [ ] **Restart Run** popover summarises what is lost, warns skills/essence carry over,
      and Esc/backdrop cancel it.
- [ ] **Run end** (tower at 0): enemies cleared, summary shown with wave, score, essence
      earned this run, turrets, minions, and a record flag if the run beat the previous
      best. Button becomes `New Run`.
- [ ] Run history keeps the last 8 runs with run number, wave, score, peak and modifier.
- [ ] `runsPlayed` and `bestRunWave` increment correctly.
- [ ] The run state is cleared **at end**, not at next start - so reloading on the
      summary screen cannot resurrect a lost bank.
- [ ] **Runtime snapshot**: quitting mid-run and reloading restores player position,
      nodes, enemies, turrets, minions, wave state, weather, day phase, spell cooldowns,
      haste/paint/chill timers and the tribe timers.
- [ ] Snapshot version is 1; a snapshot from an unknown version is ignored, not crashed on.

---

## 13. Drafts and events

- [ ] A draft fires every 5 waves, offering 3 of the 8 boons, and pauses the game.
- [ ] 8 boons: Rich Harvest (+15% banked), Keen Edge (+1 minion/defender damage),
      Iron Posts (+50% minion HP), Deep Bags (+4 capacity), Fleet Foot (faster player),
      Keen Slings (+20% turret fire rate), Warded Fields (slower blight consumption),
      Bounty (30 essence now).
- [ ] Boons stack when drafted more than once.
- [ ] An event fires every 7 waves - offset from the draft, so **a break never asks two
      questions at once**. Verify around wave 35 (LCM of 5 and 7).
- [ ] 4 events, each a trade; `Accept` runs both halves, `Leave it` costs nothing:
      Blight Surge (3 blights now, x3 essence for a minute),
      War Drums (next wave +6, take 50 essence),
      Forced March (x2 yield for 45s, tower loses 12 HP),
      Old Bones (a ghoul horde now, every field refills).
- [ ] Both popovers block input behind them and cannot be dismissed without choosing
      (draft) or answering (event).
- [ ] Boons and event effects are **run state** - gone after the run ends.

---

## 14. Day/night cycle

- [ ] 60s of day, 40s of night, derived from `worldAge` - it cannot drift across a pause,
      a reload or a snapshot restore.
- [ ] Dusk and dawn ease the tint over 300 ticks each; the tint peaks at 0.46.
- [ ] **The gameplay boundary is a hard edge** - "can I trade" never has an ambiguous
      answer even mid-fade.
- [ ] At night: enemy HP x1.35, enemy speed x1.12, ghoul interval x0.5, essence x2.
- [ ] Essence pays at the rate of the moment of death, not the moment of spawn.
- [ ] Day number increments correctly.

---

## 15. Weather

- [ ] First weather ~5400 ticks in; gaps of 7200-14400 ticks; each lasts 2400-3600 ticks.
- [ ] A 600-tick warning with a specific message precedes each one.
- [ ] **The tint eases in and out over 120 ticks at each end, whatever duration rolled.**
      Alpha must start at 0 and ramp by 1/120 a tick - never snap on in a single frame.
      Regression: the fade-in was anchored to `WEATHER_MAX_TICKS` instead of the rolled
      duration, so ~9 rolls in 10 arrived at full strength instantly and the view appeared
      to jump. Watch several onsets, not one - the old bug was correct for long rolls.
- [ ] Nothing about weather touches the camera: `cameraState.zoom`, `viewport`,
      `renderScale` and the canvas size must be identical before and after a change.
- [ ] Restoring a snapshot taken mid-weather shows it **already arrived** - no second
      fade-in on load. Snapshots written before `weatherDuration` existed must do the same.
- [ ] **Fog never rolls after dusk** - but fog already running when night falls rides out.
- [ ] Fog: turret/defender range x0.5, aggro x0.6, enemy shot range x0.55.
- [ ] Rain: respawn x0.6, **ferry closed**, rain particles.
- [ ] Hard Frost: enemy speed x0.65, minion speed x0.78, slow fields x1.5, **fields
      frozen** (nothing regrows), snow particles.
- [ ] Thunderstorm: +35% banked, lightning every 150 ticks in a 95-unit radius doing 14
      to enemies and **3 to your own minions** - it does not check sides.
- [ ] Aetherfall: spell cooldowns x2 rate, enemy speed x1.15, essence x1.5.
- [ ] Lightning marks cap at 8 and expire after 20 ticks.
- [ ] Weather tint layers correctly with the night tint.

---

## 16. Spells

Each is cast from the bank's aether. Verify cost, gating, cooldown and effect.

- [ ] **Haste** (`minionHaste`): 10 aether + 1 per banked minute. x1.5 minion speed for
      60s. Recasting **stacks another minute** rather than restarting.
- [ ] **Mend** (`mendSpell`): 12 aether, 60s cooldown, +25 tower HP. Greyed with
      "Tower intact" at full HP.
- [ ] **Waygate** (`waygateSpell`): 8 aether, 45s cooldown, snaps every minion home and
      **banks their full bags on the way**. Greyed with "No minions" at zero.
- [ ] **Chill** (`chillSpell`): base 14 aether **+1 per 10s already on the clock**, no
      cooldown, x0.45 enemy speed for 10s, stacking. Greyed with "No enemies".
- [ ] **Bloom** (`bloomSpell`): 10 aether, 90s cooldown, refills every field **a blight is
      not sitting on**.
- [ ] **Every spell is cast from the map toolbar, not the shop** - slots 2-7, keys `2`-`7`,
      in the order Haste, War Paint, Bloom, Waygate, Chill, Mend. See §18 for the slot
      behaviour; the costs and effects above are what each one must still do from there.
- [ ] **The Haste slot shows its real cost: 10 aether, rising by 1 per banked minute.**
      Regression: the old shop button called `setShopButtonContent(minionHasteBtn,
      warPaintBtn, "Haste Spell", status, recipe)` - one argument too many, so every
      parameter shifted one place right. The button rendered `[object HTMLButtonElement]`
      as its title, put "Haste Spell" in the level slot, and passed the *status string*
      where the recipe belonged; `costAmount()` read `undefined` off it for every resource
      and the button advertised **`Free`**. Two reasons a naive check missed it: the button
      only renders once `minionHaste` is unlocked, so a fresh save never shows it at all,
      and `setButtonDisabled` used the correct recipe - so the button still greyed out when
      you could not afford it, which reads as a working cost. Check the number, not the
      enabled state.
- [ ] A spell whose cost is a **stacking** one (Haste, War Paint, Chill) shows the price of
      the *next* cast, which rises while the effect runs. Cast one, watch the slot's number
      go up, let it lapse, watch it come back down.
- [ ] Casting from the toolbar **never pauses the run**. Opening the bestiary does
      (§18); a spell is thrown at a wave already walking in. Cast at 1x and confirm enemies
      keep moving through the cast.
- [ ] Cooldown readouts count down in real time and at 2x/3x speed.
- [ ] Aetherfall halves every cooldown's remaining time rate.

---

## 17. HUD

- [ ] Bank strip: wood, stone, crystal, amber, aether, ivory, essence - matches the
      skill-tree strip exactly.
- [ ] Bag strip with `Cap used/max`.
- [ ] Tower HP chip, Wave chip, Incoming countdown + next-wave hint, Tribe chip,
      Fields chip, Prog, Best run.
- [ ] **Every countdown reads mm:ss below one hour and `Nh MM` at or above it.** The
      boundary is exact: 3599s shows `59:59`, 3600s shows `1h00`. `formatWaveCountdown()`
      is shared by the wave break, spell cooldowns and the toolbar badges, so check it in
      all three places after touching it.
      Regression: it was mm:ss unconditionally, and the three stacking spells have no
      ceiling - Haste banks a minute per cast, so a hundred casts reached `100:00`, which
      reads as a hundred *seconds*. Two traps here. The first is that you cannot reach this
      by playing normally: it needs ~100 recasts, so every ordinary session shows a
      perfectly correct clock. The second is the fix that looks obvious and is worse -
      plain `hh:mm` would render 100 minutes as `01:40`, the same two-field shape as 1m40s
      and forty times the value, so the readout would become ambiguous exactly where it
      used to be merely ugly. The unit letter is the load-bearing part; do not "tidy" it
      back into a colon.
- [ ] Numbers refresh every 4 ticks and use short notation for large values.
- [ ] Chips pulse/cue on change (deposit, tower hit, minion loss).
- [ ] The HUD is readable at 1x, 2x and 3x game speed.

---

## 18. Map toolbar and bestiary

The toolbar is a hotbar drawn over the bottom of the map. Slot 1 opens the bestiary,
slots 2-7 cast the six spells, and 8-9 stay empty as room for whatever moves in next.

### The toolbar

- [ ] Nine slots, numbered 1-9, centred over the bottom edge of the map. Slot 1 shows the
      bestiary icon and a `n/9` badge; slots 8 and 9 are dashed and inert.
- [ ] Clicking slot 1 opens the bestiary. Clicking it again while open does nothing (it does
      not toggle shut) - the Close button, Esc and a backdrop click are the ways out.
- [ ] **A press that lands anywhere on the toolbar except a live slot reaches the map.**
      The bar's own padding and the 5px gaps between slots must place a turret or start a
      camera drag exactly as bare map does. Regression risk: the bar is positioned over the
      canvas, so giving the container `pointer-events` would make its whole 46px-tall strip
      a dead zone that silently swallows clicks. Test by aiming at the **gaps between
      slots** and at the bar's outer padding, not at the slots themselves - clicking a slot
      works either way, and clicking the middle of the map works either way, so both of the
      obvious checks pass while the bug is present.
- [ ] The badge count updates without the bar being rebuilt: hover slot 1, kill an enemy of
      a kind never felled before, and the hover highlight must survive the count changing.
      Regression risk: `refreshHud()` runs every 4 ticks, so rebuilding `innerHTML` there
      would drop the hover state roughly 15 times a second - visible only while the cursor
      is actually resting on the slot.
- [ ] The toolbar is legible at 1x, 2x and 3x game speed and at every map zoom - it is
      screen-anchored, so zoom must not move or scale it.

### Spell slots (2-7)

- [ ] Slots 2-7 are Haste, War Paint, Bloom, Waygate, Chill, Mend, in that order, matching
      keys `2`-`7`. Slots 8 and 9 stay empty.
- [ ] Each shows its icon, its cost as an icon-and-number along the bottom, and a countdown
      badge in the top corner. The bestiary's badge stays in the **bottom** corner - it has no
      cost row to make way for.
- [ ] **A locked spell keeps its slot.** Unlock spells one at a time and confirm no slot
      ever changes number: Mend must be `7` before and after Chill is unlocked. Regression
      risk: the shop hid unlocked-only buttons, which is right for a list that reflows and
      wrong for a bar you aim at by number - carrying `setButtonVisible` over would
      resequence every spell you had already learned the position of. A locked slot greys
      out and its tooltip says to unlock it in the skill tree.
- [ ] **A locked slot still shows that tooltip on hover.** It is set with `aria-disabled`,
      not the `disabled` property, because Chrome delivers no mouse events to a disabled
      button and the tooltip - the only thing naming the spell and saying how to get it -
      would silently never appear. Hover a locked slot in Chrome specifically; in some other
      browsers a disabled button *does* show its title, so this passes elsewhere while
      failing where most people play.
- [ ] The badge counts **down** and shows the right clock: a cooldown for Mend, Waygate and
      Bloom, and the remaining effect for the three that stack (Haste, War Paint, Chill).
      No spell is ever in both states - the stacking ones have no cooldown.
- [ ] A badge past an hour reads `1h40`, not `100:00` - see §17. Cast Haste ~100 times and
      confirm the slot stays legible and the number stays inside the 46px slot.
- [ ] A running spell's slot turns green (`.active`) and its badge counts the effect out.
- [ ] A spell that cannot be cast right now - unaffordable, or Mend at full tower HP, or
      Waygate with no minions - dims but **stays pressable**, and pressing it does nothing.
      The cast paths re-check for themselves; the slot is not the guard.
- [ ] Casting updates the slot immediately, not on the next 4-tick HUD pass. Cast Chill and
      watch its price rise on the same frame the aether leaves the bank.
- [ ] Keys `2`-`7` cast without the pointer going near the bar; `8` and `9` do nothing.

### The bestiary

- [ ] Opening the bestiary **pauses a running run**, and closing it resumes - but only if the
      bestiary was what paused it. Pause by hand, open the bestiary, close it: the game must
      stay paused. Same contract as the skill tree, with its own resume flag.
- [ ] A kind never felled shows as `???` with a dark silhouette in the right *shape* and no
      stats. The shape is deliberate: the roster shows how much is left to find.
- [ ] Nine kinds are listed: scout, brute, wisp, hunter, ghoul, warden, blight, boss,
      war chief. **Boss and war chief must both be present.** Regression risk: neither is
      in `enemyTypeConfig` - the boss is built by `spawnBossEnemy()` and the war chief is a
      warden promoted in place by `spawnWarChief()` - so a roster derived from that table
      lists seven and looks complete. Count the rows; do not eyeball the list.

### Kill counts

- [ ] Counts are **meta**: they survive the tower falling. Kill a few things, let the run
      end, and the bestiary still reads the same totals. They sit outside `freshRunState()`
      for this reason, next to `warChiefKills` and `runsPlayed`.
- [ ] `Reset Save` clears them (and the undo restores them), the same as every other
      meta counter.
- [ ] **The wave that kills you is not scored.** `endRun()` clears `game.enemies` directly
      rather than through `defeatEnemyAt()`, so dying with nine enemies on the map must add
      **zero** kills. Note the totals, let the tower fall, compare. A naive check misses
      this because the summary screen covers the map and the bestiary is not open at the time.
- [ ] **The war chief row reads `save.warChiefKills`, not a tally of its own.** Kill a war
      chief and confirm the bestiary row and the War Paint price move together - that price
      is `WAR_PAINT_BASE_COST + warChiefKills * 6`, so a second counter would show up as
      the bestiary and the shop disagreeing about how many you have killed. Regression risk:
      counting the war chief in `recordBestiaryKill()` as well would double-count it into a
      field that also sizes the next war party.
- [ ] Counts persist across a reload without a purchase in between. They ride the same
      path essence does - the `beforeunload` snapshot and the save on run end - rather than
      writing to localStorage on every death.

### The scaling badge

- [ ] Base stats are wave-1 values. The orange `+n%` beside one is what the **current**
      wave adds, and it is read per kind.
- [ ] **Off-wave kinds show a much smaller increase than wave enemies at the same wave.**
      At wave 30 a scout reads about `+431%` health while a ghoul, warden or blight reads
      about `+131%`, because `OFFWAVE_HP_SCALE_ROOT = 0.5` puts anything that never pushes
      the tower on the square root of the curve. Check a ghoul row and a scout row **in the
      same session** - a single row looks perfectly reasonable on its own, and one shared
      multiplier applied to every row would be wrong on five of the nine while still
      looking like a working feature.
- [ ] **The percentages carry that on their own - there is no "off-wave curve" tag.**
      A wave row and an off-wave row differ by a factor of three in the badge, which says
      it more plainly than a label does, and the note under the list gives the reason. The
      tag existed and was removed as a caption on a number that was already speaking; do
      not reinstate it. Only boss and war chief carry a text chip, because their tier is
      genuinely not derivable from the figure shown.
- [ ] **Hovering an HP chip shows the working, line by line.** A ghoul at wave 30 by day
      reads:
      `wave 30: (1 + 30 x 0.08) x 1.015^30 = x5.31` / `off-wave: x5.31^0.5 = x2.31` /
      `= x2.31 -> +131% (3 -> 7 HP)`. The chip carries a dashed underline and a help cursor
      so the tooltip is not a secret.
- [ ] **Only the terms actually doing something are listed.** By day with no modifier
      rolled there is no `night:` line and no modifier line - at midday those would be
      three `x1` rows and no explanation. Check the same kind by day and by night: the
      night line appears and the total changes with it.
- [ ] A modifier that scales enemy health names itself in the working (`Thin Ranks: x1.7`),
      and one that does not touch health adds no line at all.
- [ ] **The working and the badge cannot disagree.** Both come out of the same locals in
      `bestiaryScaling()` - the tooltip is not a second copy of the formula written by
      hand. If they ever differ, the fix is in that function, not in the renderer: a
      formula restated somewhere else is one that will drift.
- [ ] The off-wave step is written as an **exponent** (`^0.5`), not as "sqrt", so it stays
      true if `OFFWAVE_HP_SCALE_ROOT` is ever retuned.
- [ ] Boss and war chief rows carry no working - they show absolute HP and a tier rather
      than a bonus, so there is no percentage to explain.
- [ ] **Chip order is HP, Spd, Dmg - and an optional chip is always the last one.**
      Ghoul, warden and blight deal no tower damage and show **no Dmg chip at all**; their
      rows simply end after Spd. War chief likewise ends after its tier.
      Regression: those rows used to print a placeholder `Dmg -` purely so the Spd chip
      would land in the same place, which put a chip carrying no information in the middle
      of the row. An absent chip has to be last, or its absence moves everything after it.
      Check a zero-damage row **against a damaging one**: a single row looks fine either
      way, and the misalignment only exists as a difference between rows.
- [ ] Dropping the placeholder loses nothing - ghoul, warden and blight each say they do
      not touch the tower in their own description line.
- [ ] **Hovering a Dmg chip says the figure is tower damage only, and shows its working.**
      A scout at wave 30 reads: `tower damage: what it takes off the tower on arrival` /
      `wave 30: 1 + 30 x 0.03 = x1.90` / `= 5 -> 10` / `fixed at spawn: one that arrived on
      an earlier wave still hits for that wave`. The boss instead reads
      `16 + tier 3 x 2 = 22` and says it steps with the tier rather than every wave.
      This matters more than the HP working does: `enemy.damage` is read in exactly one
      place - the tower-hit branch - and what a **minion** takes is a separate path that
      never reads it. A bare "Dmg 5" reads as a general threat rating when it is nothing of
      the kind, which is why the four zero-damage kinds are still lethal to gatherers.
      Regression: the only tooltip saying this was attached to the `Dmg -` placeholder, and
      it was lost along with the placeholder when zero-damage rows stopped showing a chip.
      A tooltip carried by an element that gets deleted for an unrelated reason leaves no
      trace that it was ever there.
- [ ] Boss and war chief show an **absolute** figure, not a percentage. The boss steps
      every ten waves (`(10 + tier*6)` where tier is `floor(wave/10)`), and the war chief is
      off the wave curve entirely at `WARCHIEF_BASE_HP + tier * WARCHIEF_HP_PER_TIER`
      (400 + 200 a kill). Kill a war chief and confirm its row's HP jumps by 200 with no
      wave having passed.
- [ ] **Neither carries a tier chip; both carry their working on hover.** At wave 30 by day
      the boss HP chip reads: `tier 3 (wave 30 / 10)` / `base: 10 + 3 x 6 = 28` /
      `wave 30: (1 + 30 x 0.08) x 1.015^30 = x5.31` / `= 149 HP`. After two war-chief kills
      that row reads: `tier 2 = war chiefs felled` / `400 + 2 x 200 = 800 HP` /
      `flat: neither the wave curve nor night touches this`.
      The chips went for different reasons, and both are worth keeping in mind when adding
      a row: the boss's tier is the first line of its own working, and the war chief's tier
      *is* its kill count, which the row already prints in the KILLS column an inch to the
      left. A chip that restates something already on screen is noise wherever it appears.
- [ ] **The war chief's HP is flat.** No wave curve, no night scale, no run modifier - the
      working says so on its third line. `spawnWarChief()` overwrites the health it
      inherited from the warden it was promoted from, which is what makes it flat; check
      the row is identical at wave 5 and wave 50, and by night as well as by day.
- [ ] **The boss does not take the run modifier** either. There is no `enemyHpScale` term in
      its working, because `spawnBossEnemy()` has none - it applies only the wave curve and
      the night scale. Roll `Thin Ranks` and confirm the boss figure is unchanged while the
      wave enemies' badges all rise. If that ever changes it has to change in both places,
      or the working starts lying.
- [ ] Cross-check both against the real thing: at wave 30 the boss row must read the HP a
      boss actually spawns with (149 by day). The bestiary restates the spawners' arithmetic
      rather than calling it - see §24. `WARCHIEF_HP_PER_TIER` exists so that at least the
      war chief's per-kill step is one number rather than two copies of it.
- [ ] At night the badge rises for every kind (`NIGHT_ENEMY_HP_SCALE = 1.35`) and falls
      again at dawn. Open the bestiary once by day and once by night on the same wave.
- [ ] A run modifier that scales enemy health (`Thin Ranks`, `enemyHpScale: 1.7`) is in the
      badge. Compare the same kind across two runs with different modifiers.
- [ ] Before a run starts the badge is absent rather than `+0%` - wave 0 adds nothing.
- [ ] **If a spawn formula changes, `bestiaryScaling()` changes with it.** It mirrors the
      arithmetic in the spawners rather than calling them, so the two can drift silently.
      Sanity-check a row against a real enemy's health bar at the same wave.

---

## 19. Tutorial and tips

- [ ] Six steps in order: gather → bank → open the skill tree → craft a gatherer →
      unlock defenders/slings → survive wave 1.
- [ ] Each step retires itself the moment its condition is met, **even out of order** -
      banking before the bank hint appears must skip both.
- [ ] Progress keeps being tracked while tips are **off**, so re-enabling shows the step
      you are actually on.
- [ ] Tips are meta state: the game teaches this once, across runs and resets of the run
      (but a full Reset Save starts them again).
- [ ] `T` and the settings-panel toggle agree with each other.

---

## 20. Audio and settings

- [ ] Settings panel opens from `♪`, closes on Esc, on a click outside, and stays open on
      a click inside.
- [ ] Music and Effects mute independently - **muting music must not silence gameplay**.
- [ ] Volume slider and `+`/`-` agree; the readout shows a percentage.
- [ ] Volume and mute states persist across reload.
- [ ] Stings fire: tower hit (scaled by damage), turret sell, lightning, defeat.
- [ ] Music turns dark while a boss is alive and recovers when it dies or the run ends.
- [ ] Audio starts only after a user gesture (no autoplay warning in the console).
- [ ] **A muted page makes no sound at all on the first gesture.** Load with music muted,
      click once, and listen: not one note, not a fragment of one. Same with Effects muted.
      Regression: `ctx.createGain()` starts at gain **1** and the two buses were never
      given a starting value - `applyMusicVolume()` only ever *ramps* toward its target
      with `setTargetAtTime(..., 0.08)`. A muted music bus therefore began at full and
      decayed exponentially: still ~37% one tau (80ms) later, ~14% at 160ms. And
      `musicState.nextStepTime` is set to `ctx.currentTime + 0.08`, putting the very first
      note at 80ms - inside that window. One or two notes escaped and faded out.
      Three reasons a naive check misses it: it is **one or two notes** and gone, so it
      reads as a stray click rather than the track playing; it only happens on the gesture
      that *builds* the audio graph, so muting and unmuting in an already-running session
      never shows it; and whether anything is audible depends on whether step 0 of the
      pattern carries a note, which is why it was reported as happening only sometimes.
- [ ] **Music is muted by the bus gain alone - there is no per-note check.** The SFX
      helpers each early-return on `save.sfxMuted`, but `scheduleMusicStep()` schedules
      unconditionally, so anything that leaves the music bus at the wrong gain is instantly
      audible. Do not "fix" a future leak by having the scheduler skip while muted: the
      step clock would fall behind `ctx.currentTime` and unmuting would flush every missed
      step at once, as a cluster.
- [ ] A mute pressed **during** play still ramps rather than cutting - a hard cut on a
      running voice is an audible click. Only a freshly built graph sets its gains
      outright. Toggle `M` mid-run and listen for a click at the transition.

---

## 21. Camera and rendering

- [ ] Zoom clamps to 0.25x-1x; the camera clamps to the map so no void is ever visible.
- [ ] Resizing the window refits the canvas; **with the game paused or over, the frame
      repaints** rather than going blank behind the overlay.
- [ ] Wheel-zoom keeps the point under the cursor stable.
- [ ] Map labels are baked once and stay crisp at every zoom.
- [ ] Floaters cap at 60 and expire after 42 ticks.
- [ ] Frame rate holds with a full late wave, 20 ghouls, many minions and a storm running.

---

## 22. Leaderboard

### Client
- [ ] With no server reachable, the **tab is hidden entirely** and the game plays
      normally; the status pill reads `offline-mode`.
- [ ] With a server, the tab appears, the pill reads `live`, and the board renders.
- [ ] If the board drops while you are reading it, the tab hides and you are returned to
      the shop - never left on a dead panel.
- [ ] The board does **not** pause the game - unlike the skill tree, the run carries on
      behind it. (The README used to claim otherwise; it now matches the code.)
- [ ] Your own row is highlighted (matched on `player_id`).
- [ ] **Names render as text, never HTML** - submit a name containing `<b>` or a script
      tag from another client and confirm it renders literally.
- [ ] Setting a name persists it, pushes it immediately, and **keeps your place** on the
      board.
- [ ] Scores are submitted on run end, on opening the board, and periodically -
      **only ever when you beat your own record**, and at most once per 5s.
- [ ] Identity (`browsergame-player` in localStorage) survives reload; corrupt or blocked
      storage mints a fresh identity rather than throwing.
- [ ] The socket reconnects with backoff 1s → 30s; a submit made while the socket is down
      falls back to `POST /api/scores` and is not lost.
- [ ] Opened as a `file://` page, the client targets `localhost:8090`.

### Server (`cargo run` in `server/`, port 8090)
- [ ] `GET /api/health` → `{"status":"ok"}`.
- [ ] `GET /api/leaderboard?limit=20`; limit clamps to 1-100, default 20.
- [ ] `POST /api/scores` → `{rank, best, improved}`; `improved: false` when the score did
      not beat that player's own record.
- [ ] Rejects: empty name, name over 24 chars, `player_id` outside 8-64 chars of
      `[A-Za-z0-9_-]`, score over 1e12. Each returns 400 with its message.
- [ ] One row per `player_id`, holding that player's best. Ties order by who got there
      first.
- [ ] Renaming always takes effect; re-submitting a worse score changes nothing.
- [ ] Websocket: full snapshot on connect, one message per change, `subscribe` resends,
      `ping` → `pong`, unknown type → an error message rather than a dropped socket.
- [ ] A lagged subscriber gets one fresh snapshot instead of an error.
- [ ] The board file is written via temp-file + rename, so a crash mid-write cannot
      corrupt it. Kill the server mid-write and confirm the file still parses.
- [ ] Restarting the server reloads the board from `BROWSERGAME_DATA`.
- [ ] Static hosting serves the game at `/` from `BROWSERGAME_STATIC_DIR`; with the
      directory missing, the API still starts and says so.
- [ ] CORS preflight (`OPTIONS`) answers 204 - needed only for the `file://` case.

---

## 23. Known non-goals

These are deliberate. Do not file them as bugs.

- Scores are accepted on the client's word. Anyone with devtools can post any number.
  Fixing it needs the server to own the game rules.
- Run exactly one server replica - the live fan-out is per-process state.
- Aether has no spawner upgrade; ivory has no multiplier of any kind.
- Lightning hits your own minions. That is the price of the storm's banking bonus.
- The ferry is slower than walking.

---

## 24. Fragile spots - check these after any refactor

Each of these has broken the game before, and the failure was silent.

- [ ] **Temporal dead zone on load.** `loadSave()` runs while the module body is still
      initialising and its `catch` swallows everything. A module-scope `const` read from
      inside it throws and silently resets the save. `GAME_SPEEDS`, `TURRET_MAX_LEVEL`,
      `TURRET_CAP_STEP` and the legacy-key rename map are all positioned for this reason.
      **Test: load an existing save and confirm it is not replaced by a fresh one.**
- [ ] **Missing vs empty arrays** in save migration (see §1).
- [ ] **Backwards iteration** wherever the loop body can splice - lightning strikes,
      `defeatEnemyAt`, floater and minion cleanup.
- [ ] **Spawner type fall-through**: `spawnerLevelFor` returns the amber level for
      anything it does not recognise - ivory must be handled before that fall-through.
- [ ] Refunds quoted at a different tier than the purchase must not walk Prog negative.
- [ ] **A render cache keyed on less than it draws.** Several panels skip re-rendering when
      a `dataset.renderKey` matches. The key has to name every value the markup interpolates,
      not just the ones the rows visibly list - the Turret Store printed a price derived from
      a count that was not in its key (§8). The failure is silent and looks like a panel that
      simply did not update, which is indistinguishable from one that had nothing to update.
- [ ] `keys` must be cleared on blur, on run end and on reset.
- [ ] **A "frozen" entity is usually a stopped clock.** `tickGame()` opens with
      `if (!game.running) return`, and `game.running` goes false on a run end *and* on a
      draft or event popover opening - every 5 and 7 waves. Anything driving the simulation
      in a harness has to answer those, or the whole world stands still and reads exactly
      like a pathing bug in whatever you happened to be watching.
- [ ] Both audio buses stay independent.
- [ ] **Anything drawn over the canvas is a click sink until proven otherwise.** The map
      is the game's largest control surface - turret placement, sale, upgrade and camera
      drag all begin with a press on it - so every overlay has to declare what it does with
      pointer events, and the two overlays currently there declare *opposite* things: the
      bag chip is never interactive and opts out entirely, while the map toolbar opts the
      container out and its slots back in. Copying either one into a new overlay without
      reading which case it is gives a transparent dead region. It is invisible, and it
      only bites when the player aims at that part of the map.
- [ ] **`Esc` falls through to pause.** The keydown chain closes each open panel in turn
      and, if nothing claimed the key, toggles pause. A new panel that does not add its own
      branch *above* that fallback does not fail loudly - it stays open and pauses the run
      underneath itself.
- [ ] **The bestiary mirrors spawn arithmetic it does not call.** `bestiaryScaling()`
      restates the five health curves the spawners use. Changing a spawn formula without
      changing it leaves the bestiary confidently wrong - see §18.
- [ ] **Positional arguments shift silently.** `setShopButtonContent(button, label,
      levelText, cost)` and `setSkillNodeContent` both take four or five same-typed
      arguments in a row, and passing one too many does not throw - it slides every later
      argument along and renders `[object HTMLButtonElement]`, a status string where a cost
      belongs, and a `Free` price tag. This shipped on the Haste button (see §16). After
      touching any of these call sites, read the *rendered* button, not the code.
- [ ] **A `disabled` button is invisible to the mouse.** Chrome delivers no events to one,
      so its `title` never appears. Anywhere a tooltip is the only explanation for why a
      control is unavailable, gate it with `aria-disabled` and a guard in the handler
      instead - see the locked toolbar slots in §18.
- [ ] **A limit checked per actor, against only what has already arrived.** Several
      minions run the same decision on the same tick, so a guard that reads finished state
      ("do I own fewer than the cap?") lets every one of them claim the same last slot.
      Anything already paid for and in flight has to count towards the limit at the moment
      the *next* one is authorised - see `turretsInTransit()` and §8. This is invisible
      with one minion, which is how such a route gets tested.
- [ ] **A counter whose name disagrees with what it counts.** `placedTurretCount()`
      returned turrets *owned*, and the turret cap was checked against it precisely because
      the name said what the caller wanted to hear. When a count feeds a limit, read the
      body before trusting the identifier.
- [ ] **A node created with a default, then only ever ramped.** A Web Audio `GainNode`
      starts at 1 and `setTargetAtTime` is an exponential approach, so "set it to muted"
      is a fade *from full*, not a state. Anything built while it should already be silent
      has to be given its value outright at construction - see `startMusic()` and §20. The
      same shape applies to any node whose steady-state setter is a ramp.
