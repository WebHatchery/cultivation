# TODO — Heavenly Mandate

## Standards and correctness

- Expose testable logic through `src/lib.rs`, make `main.rs` use it, and migrate `src/data/loader/tests.rs` and `src/engine/world_sim/tests.rs` into `tests/`, preserving coverage. Target at most five focused cases per major feature (§11).
- Remove `#![allow(warnings)]` from `src/main.rs`; delete unused code and unnecessary `_` parameters, resolve compiler/Clippy warnings, and document any narrow lint allowances (§1.4, §10.2).
- Split long functions in `src/ui/herbs.rs`, `src/data/loader.rs`, and `src/game/update.rs` into cohesive responsibilities under 100 lines. Prioritize `herbs.rs` (787 lines); retain the empty-exception source gate, correct its obsolete “non-test lines” comment, and use named module files when restructuring (§2.2–2.3, §4.1).
- Separate immutable content from campaign state: move runtime buildings and map corruption out of `GameData`, place `WorldSimBalance` in the data domain, and pass structured state views instead of the long sect-base argument lists (§2.1, §4.3, §5).
- Separate rendering from state mutation across `src/state/` and `src/ui/`; route tutorial changes and gameplay decisions through explicit actions, and remove placeholder `draw` methods (§7).
- Add semantic catalogue validation before collecting IDs into maps: reject duplicates, broken recipe/tech/mission/faction references, invalid stage progression, and invalid balance values. Keep toolkit JSON loading and add malformed-data regression cases (§5.3).
- Move remaining cultivation, crafting, herb, beast, building and starting-scenario tuning, tutorial copy, and player-facing strings into typed JSON under `assets/`, loaded through the toolkit; test balance boundaries (§5.3).
- Return save errors from `Game::save` and report success only after storage succeeds; show actionable startup/load failures instead of the `Game::new` panic or silent UI failure (§6).
- Reset spirit beasts, scheduler state, and mutated map state in `handle_start_new_game`; add a regression for starting a second campaign in the same process.
- Validate mission assignments and tribulation outcomes in action handlers before mutation; reject invalid/busy/duplicate disciples and repeated reward claims. Add behavior tests for roster actions, faction milestones, mission assignment/resolution, and save migration (§5.1, §11).
- Isolate gameplay random rolls in small helpers or a campaign-owned seeded RNG; make mission, breakthrough, crafting, and world-evolution outcomes reproducible in tests.
- Add missing module-purpose `//!` documentation, including the `data`, `game`, and screen modules (§9.2).

## Touch controls and presentation

- Add touch pan/zoom and visible placement cancellation to the sect map; replace wheel-only list scrolling with touch gestures or visible controls. Use toolkit release-based interactions for custom buttons and rows (§7.4–7.5).
- Make sect-base sidebars, the forced 420px map width, and the 480px tutorial card adapt to narrow screens. Verify menus, modals, and gameplay at desktop and touch sizes; replace matching captures directly in `docs/verification/` (§7.5, §12).
- Correct the unsupported WASD-pan/Space-pause claims in `README.md` and `game_page.json`; document actual visible controls and gestures. Extend the existing tutorial/highlights to explain cultivation and reward reinvestment, naming the exact next control (§7.5).
- Finish the UI readability pass: fix remaining contrast and panel overlaps, reduce all-caps body copy, and emphasize each screen's pending decision over background logs.
- Replace remaining raw IDs in Sect Annals with display names and authored event text; distinguish major events from routine notices.
- Show when Feng Shui becomes Auspicious and display the actual cultivation or production bonus beside its score.
- Add a persistent sect power/prestige readout and progression milestone ladder.
- Add a browsable, saved chronicle of breakthroughs, deaths, and legends; connect remaining death paths to the existing `moments` overlay.
- Gate bottleneck explanations behind a building or technology and reveal hints consistently in roster details, cultivation notices, and breakthrough feedback.

## Gameplay and content

- Add 2–3 cultivation Laws with distinct mechanics beyond the three seed paths in `assets/data/laws.json`.
- Generate missions from data-driven templates and modifiers; add pre-mission choices for consumables, formations, or increased danger.
- Expand the two spirit-beast definitions and add beast assignment to missions through the existing beast-management UI.
- Expand the two world-map nodes; render current faction ownership and trade-route safety from campaign state, and add mission markers with validated dispatch actions.
- Implement sect-wide resource capacity, storage upgrades/buildings, and a header usage readout.
- Add timestamped saves, bounded offline progression, and a “while you were away” summary.
- Implement prestige/New Game+ with explicit reset and carryover rules.
- Add music and sound effects with visible volume/mute controls.
