# jibjack_trivia

A terminal trivia game written in Rust. Pick a category and difficulty, answer 10 multiple-choice questions from [Open Trivia Database](https://opentdb.com/), or from an AI and compete for a spot on the local high score board.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain via `rustup`)
- Internet access (questions are fetched when a game starts)
- A terminal that supports ANSI colors (macOS Terminal, iTerm2, Windows Terminal, etc.)
- Optional API keys for AI question sources:
  - `OPENAI_API_KEY` for OpenAI
  - `ANTHROPIC_API_KEY` for Anthropic

## Build and run

From the project root:

```bash
# Development build (faster compile, slower runtime)
cargo run

# Optimized build (recommended for play)
cargo run --release
```

## AI source setup

Open Trivia DB does not require an API key. To use the OpenAI or Anthropic question sources, create environment variables before running the game.

You can create a local `.env` file in the project root:

```env
OPENAI_API_KEY=your-openai-key
ANTHROPIC_API_KEY=your-anthropic-key
```

Or export them from your shell, for example in `~/.zshrc`:

```zsh
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
```

After editing `~/.zshrc`, restart your terminal or run:

```bash
source ~/.zshrc
```

## How to play

1. **Splash** — press Enter to continue
2. **Main menu** — choose **Play**, **High Scores**, or **Quit**
3. **Category** — pick from 25 OpenTDB categories (scrollable list)
4. **Difficulty** — Easy, Medium, or Hard
5. **Question source** — choose Open Trivia DB, OpenAI, or Anthropic
6. **Game** — answer 10 questions before time runs out
7. **Name entry** — enter up to 3 initials for the high score table
8. **Game over** — play again or return to the menu

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

Questions can come from [Open Trivia Database](https://opentdb.com/api_config.php), OpenAI, or Anthropic. OpenTDB works without credentials. OpenAI and Anthropic require the environment variables listed in [AI source setup](#ai-source-setup).

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

This project is licensed under the [MIT License](LICENSE).
