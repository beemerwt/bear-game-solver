# Bear Game Solver

A Rust project that models the **Bear Game** as an undirected graph and solves it as a finite perfect-information game.

## Coordinate convention

Coordinates are only used for debugging/layout output. **Positive Y means down**.
Legal movement is determined exclusively by explicit edges.

## Commands

- `cargo run -- validate`
- `cargo run -- moves`
- `cargo run -- solve`
- `cargo run -- policy -- --limit 25`
- `cargo run -- render-svg` (optional SVG to `target/bear_game_board.svg`)

## Win/loss interpretation

- Hunters win when the bear has no legal move.
- Bear wins when `hunter_turns_used >= turn_limit` and bear is not trapped.
- `hunter_turns_used` increments after a **hunter move** only.
- Default interpretation: `turn_limit = 40` means 40 hunter moves maximum.

## What counts as a guaranteed hunter win?

A hunter win is guaranteed only when the solver finds at least one hunter move at every hunter decision point such that, no matter which legal move the bear chooses afterward, the resulting state is still classified as `HuntersWin`.

If the bear has even one legal response that reaches a `BearWin` state, then the current bear-turn state is **not** a guaranteed hunter win.

The proof is the full solved policy region plus universal bear-branch closure, not a single principal variation line.

## Web Server

Run:

- `cargo run -- serve`

Then open:

- http://127.0.0.1:3000

Notes:
- The browser UI is a debugging/play interface.
- The Rust solver remains the source of truth.
- Coordinates are used only for rendering.
- Edges define legal movement.
- The solver classifies each position as `HuntersWin` or `BearWin` under perfect play.
- “Best move” is the minimax-selected move, not a heuristic guess.
