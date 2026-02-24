# Tetris Duo

Cooperative 2-player Tetris on a single shared board. Both players work together to survive as long as possible.

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-0.16-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- Shared 18×22 board — no fixed lanes, both pieces interact
- Full Tetris Guideline scoring: T-Spins, Back-to-Back, combos
- SRS rotation system with wall kicks
- 3-piece preview + hold for each player
- DAS/ARR tuned to Tetris Guideline defaults
- Persistent high score (`~/.config/tetris-duo/high_score.txt`)
- 8-bit background music + sound effects
- Menu → Playing ↔ Paused → Game Over state machine

## Controls

| Action | Player 1 | Player 2 |
|--------|----------|----------|
| Move Left | A | ← |
| Move Right | D | → |
| Soft Drop | S | ↓ |
| Hard Drop | Space | Enter |
| Rotate CW | E | . (Period) |
| Rotate CCW | Q | , (Comma) |
| Hold | LShift | RShift |

**Menu:** Enter to start
**Pause:** Escape — then Q to quit to menu

## Requirements

- Rust (stable, 2021 edition)
- System packages (Debian/Ubuntu):

```bash
sudo apt install libudev-dev libasound2-dev
```

## Build & Run

```bash
cargo run --release
```

## Run Tests

```bash
cargo test
```

38 unit tests covering board logic, piece rotation, collision detection, player mechanics, and scoring.

## Architecture

Built with [Bevy 0.16](https://bevyengine.org/) ECS and [leafwing-input-manager 0.17](https://github.com/Leafwing-Studios/leafwing-input-manager).

| Module | Role |
|--------|------|
| `main.rs` | App entry point, window config |
| `lib.rs` | Plugin — registers all systems |
| `state.rs` | `GameState` FSM |
| `board.rs` | 18×22 grid, row clearing |
| `piece.rs` | Tetromino kinds, SRS rotation tables |
| `collision.rs` | `piece_fits`, `try_rotate`, T-Spin detection |
| `player.rs` | `ActivePiece`, gravity, lock delay, DAS/ARR, hold |
| `input.rs` | Input maps for both players |
| `scoring.rs` | Score, level, combo, high score persistence |
| `render.rs` | Sprite sync for board, pieces, ghost, preview |
| `audio.rs` | Sound effects and background music |
| `ui.rs` | HUD, menus, overlays |

## License

MIT
