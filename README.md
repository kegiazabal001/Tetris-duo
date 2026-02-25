# Tetris Duo

Cooperative 2-player Tetris on a single shared board. Both players work together to clear lines in three game modes.

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-0.16-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- Shared 18×22 board — no fixed lanes, both pieces interact
- **Three game modes:** Endless, Sprint (20 lines), Ultra (2 minutes)
- Full Tetris Guideline scoring: T-Spins, Back-to-Back, combos
- SRS rotation system with wall kicks
- 3-piece preview + hold for each player
- DAS/ARR tuned to Tetris Guideline defaults
- Ghost piece, line clear flash, progressive color effects in Ultra
- Persistent high scores per mode (`~/.config/tetris-duo/settings.json`)
- Settings screen with per-player key rebinding
- 8-bit background music + sound effects

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

All keys are rebindable from the **Settings** screen.

**Menu:** Enter to start, S/↓ to open Settings
**Pause:** Escape — then Q to quit to menu

## Download & Play

Pre-built binaries are available on the [Releases page](../../releases/latest).

| Platform | File |
|----------|------|
| Linux x86_64 | `tetris-duo-linux-x86_64.tar.gz` |
| Windows x86_64 | `tetris-duo-windows-x86_64.zip` |

### Linux

```bash
tar -xzf tetris-duo-linux-x86_64.tar.gz
cd tetris-duo
./tetris-duo
```

> **Note:** The game requires ALSA and udev libraries. On Debian/Ubuntu:
> ```bash
> sudo apt install libudev1 libasound2
> ```
> These are usually already installed on desktop systems.

### Windows

Extract `tetris-duo-windows-x86_64.zip` and run `tetris-duo.exe`.

---

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

40 unit tests covering board logic, piece rotation, collision detection, player mechanics, and scoring.

## Architecture

Built with [Bevy 0.16](https://bevyengine.org/) ECS and [leafwing-input-manager 0.17](https://github.com/Leafwing-Studios/leafwing-input-manager).

| Module | Role |
|--------|------|
| `main.rs` | App entry point, window config |
| `lib.rs` | Plugin — registers all systems |
| `state.rs` | `GameState` FSM |
| `constants.rs` | Centralised gameplay constants (DAS, ARR, cell size, etc.) |
| `board.rs` | 18×22 grid, row clearing |
| `piece.rs` | Tetromino kinds, SRS rotation tables |
| `collision.rs` | `piece_fits`, `try_rotate`, T-Spin detection |
| `player.rs` | `ActivePiece`, gravity, lock delay, DAS/ARR, hold |
| `input.rs` | Input maps for both players |
| `scoring.rs` | Score, level, combo, high score persistence |
| `modes.rs` | Sprint and Ultra mode logic |
| `config.rs` | `AppConfig` — settings and high scores (JSON) |
| `render.rs` | Sprite sync for board, pieces, ghost, preview |
| `audio.rs` | Sound effects and background music |
| `ui.rs` | HUD, menus, overlays, settings screen |

## License

MIT
