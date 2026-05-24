# jibjack_trivia

A terminal trivia game written in Rust. Pick a category and difficulty, answer 10 multiple-choice questions from [Open Trivia Database](https://opentdb.com/), and compete for a spot on the local high score board.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain via `rustup`)
- Internet access (questions are fetched from OpenTDB at game start)
- A terminal that supports ANSI colors (macOS Terminal, iTerm2, Windows Terminal, etc.)

## Build and run

From the project root:

```bash
# Development build (faster compile, slower runtime)
cargo run

# Optimized build (recommended for play)
cargo run --release
```

## How to play

1. **Splash** — press Enter to continue
2. **Main menu** — choose **Play**, **High Scores**, or **Quit**
3. **Category** — pick from 25 OpenTDB categories (scrollable list)
4. **Difficulty** — Easy, Medium, or Hard
5. **Game** — answer 10 questions before time runs out
6. **Name entry** — enter up to 3 initials for the high score table
7. **Game over** — play again or return to the menu

## Scoring

| Difficulty | Time per question | Base points |
|------------|-------------------|-------------|
| Easy       | 20s               | 100         |
| Medium     | 15s               | 200         |
| Hard       | 10s               | 300         |

- **Time bonus:** +1 point per second left on the question clock when you answer correctly
- **Time bank:** half of your remaining question time is banked (up to 60s) and extends later questions if the per-question timer expires

## Controls

| Screen        | Key              | Action              |
|---------------|------------------|---------------------|
| Any           | `Ctrl+C`         | Quit immediately    |
| Splash        | `Enter` / `Space`| Continue            |
| Splash        | `Esc` / `q`      | Quit                |
| Menus / lists | `↑` / `↓` or `j` / `k` | Move selection |
| Menus / lists | `Enter` / `Space`| Confirm selection   |
| Category / Difficulty | `Esc`    | Back                |
| Category / Difficulty | `q`      | Quit                |
| Playing       | `↑` / `↓` or `j` / `k` | Choose answer |
| Playing       | `1`–`4`           | Choose answer (by number) |
| Playing       | `Enter` / `Space`| Submit answer       |
| Playing       | `Esc` / `q`      | Return to main menu |
| Name input    | Type (max 3)     | Enter initials      |
| Name input    | `Backspace`      | Delete character    |
| Name input    | `Enter`          | Save score          |
| Game over     | `r` / `Enter`    | Play again          |
| Game over     | `h`              | High scores         |
| Game over     | `Esc` / `q`      | Main menu           |
| High scores   | `Esc` / `Enter` / `q` | Main menu      |

## High scores

Scores are saved locally as JSON:

```
~/.jibjack_trivia/high_scores.json
```

The top 10 scores are kept, sorted by score (highest first).

## Question source

Questions are fetched from the [Open Trivia Database API](https://opentdb.com/api_config.php). No API key is required.

## Project layout

```
src/
  main.rs    — event loop, terminal setup/cleanup
  app.rs     — game state and input handling
  ui.rs      — TUI rendering
  trivia.rs  — OpenTDB fetch and question types
  scores.rs  — high score persistence
  ascii.rs   — ASCII art assets
```

## License

Add your license here if applicable.
