# Heavenly Mandate

Heavenly Mandate is a cultivation sect strategy game about rebuilding a fallen school, training disciples, and sending them into a dangerous world of factions and consequences.

You guide the sect from survival toward influence by choosing facilities, managing disciples, and deciding which risks are worth taking.

## Gameplay

- Build and improve sect facilities.
- Train disciples and guide their cultivation growth.
- Send followers on missions across the world map.
- Respond to faction pressure and events.
- Balance resources, needs, schedules, and long-term progress.

## Goal

Restore the sect's strength while keeping disciples alive, useful, and advancing toward higher stages of cultivation.

## Controls

- Click or tap the visible controls to navigate, select a hall, and issue actions.
- Use the map's Left / Right / Up / Down controls to pan, + / - to zoom, and Home to recenter. Mouse wheel zoom and middle-button drag are also available.
- Raise Halls opens available plans. Select a plan, then tap a clear map location; Cancel leaves placement mode.
- Recover Doctrine separates available, locked, and recovered teachings. Select a teaching to review its benefits, requirements, and treasury impact.
- Annals opens recent history; Older / Newer pages through events. Settings contains save and title controls.
- The First Decrees guide occupies its own space above gameplay. Hide dismisses it temporarily; Guide brings it back. Escape closes the current focused screen.

## Current Scope

Playable sect-management foundation with base building, disciple growth, world missions, events, and faction simulation systems.

## Documentation

- `gdd.md` â€” the game design document.
- `TODO.md` â€” open work.
- `wuxia-buildings.md` â€” the building-type reference `generate_prompts.py` reads to build art prompts.


## UX verification

The gameplay workspaces use focused lists, selected records, and explicit actions. Roster progress, lineage, and orders have separate views; dispatches move through briefing, team, and review. Facilities separate production, attendants, arrays, and upgrades. Regional maps, markets, factions, archives, reports, and guardian records page through complete content with visible touch controls. Story pages have Previous, Continue, and Skip controls; milestones pause gameplay until acknowledged.

Native UI captures at 800x600 and 1280x720 live in `docs/verification/`. The toolkit capture harness seeds real UI states for repeatable checks, including empty states and disabled actions. Capture scenes include `sectbase`, `hall`, `doctrine`, `annals`, `construction`, `roster`, `roster_orders`, `dispatch`, `dispatch_team`, `dispatch_review`, `report`, `tribulation`, `garden`, `seeds`, `attendants`, `array`, `storage`, `drying`, `workshop`, `guardians`, `mission_board`, `trade`, `factions`, `library`, `worldmap`, `creation`, `intro`, and `moment`.
