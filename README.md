# Bear Game Solver

A Rust project that models the **Bear Game** as an undirected graph and solves it via exhaustive minimax with memoization (perfect play for both sides).

## Coordinate convention

Coordinates are only used for debugging/layout output. **Positive Y means down**.
Legal movement is determined exclusively by explicit edges.

## Commands

- `cargo run -- validate`
- `cargo run -- moves`
- `cargo run -- solve`
- `cargo run -- render-svg` (optional SVG to `target/bear_game_board.svg`)

## Editing the board

Edit `src/boards/bear_game_board.rs`:

1. Add or modify `BoardNode { id, x, y }` entries.
2. Add `BoardEdge("a", "b")` entries once each (undirected).
3. Set `start.bear`, `start.hunters`, and `start.side_to_move`.

The validator checks for duplicate node IDs, invalid edges, duplicate/self edges, and invalid starting positions.
Duplicate coordinates are warnings (non-fatal).

## Win/loss interpretation

- Hunters win when the bear has no legal move.
- Bear wins when `hunter_turns_used >= turn_limit`.
- Default interpretation: `turn_limit = 40` means 40 hunter moves maximum.
- Hunters choose moves to force a win as fast as possible.
- Bear chooses moves to avoid hunter win if possible; otherwise to delay loss as long as possible.

## Main workflow

1. Define nodes with IDs and coordinates.
2. Define explicit undirected edges.
3. Set start locations and side to move.
4. Run `validate`.
5. Run `solve` to see whether hunters can force a win.

The board is manually encoded; if the node map is inaccurate, update the board definition.
