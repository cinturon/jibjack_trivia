# Learning Rust by Building a TUI Trivia Game
### A companion guide to the `trivia-tui` project

---

## How to use this guide

The source code in `trivia-tui/` is a complete, working Rust program.
Read this guide alongside the code — each section below explains a Rust concept as it appears in one of the source files.

**Suggested reading order:**
1. This guide top-to-bottom
2. The files in this order: `Cargo.toml` → `ascii.rs` → `scores.rs` → `trivia.rs` → `app.rs` → `ui.rs` → `main.rs`

---

## Getting started

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This installs `rustc` (the compiler) and `cargo` (the build tool + package manager).

### Build and run the project

```bash
cd trivia-tui
cargo run
```

The first build downloads and compiles all dependencies — this takes a minute or two. Subsequent builds are fast (Rust only recompiles changed files).

**Check for errors without running:**
```bash
cargo check
```

**Run with release optimisations (faster):**
```bash
cargo run --release
```

---

## Chapter 1 — The Rust Mindset

Before diving into syntax, it helps to understand what makes Rust different from languages you may already know.

### Ownership: the big idea

In most languages, you either:
- Manage memory manually (C/C++) — error-prone
- Use a garbage collector (Java, Python, Go) — safe but has runtime overhead

Rust uses a third approach: **ownership**. Every value has exactly one owner. When the owner goes out of scope, the value is freed. The compiler enforces this at compile time — no runtime cost, no garbage collector.

```rust
let s1 = String::from("hello");   // s1 owns the String
let s2 = s1;                       // ownership moves to s2
println!("{}", s1);                // ❌ COMPILE ERROR: s1 no longer owns anything
```

You'll see this throughout the project. When a function needs to *keep* a value, it takes ownership. When it just needs to *read* it, it borrows with `&`.

### Borrowing

```rust
let s = String::from("hello");
let len = calculate_length(&s);    // lend a reference to s
println!("{} has {} chars", s, len); // s is still valid here
```

You can have:
- Any number of **immutable borrows** (`&T`) at once
- OR exactly one **mutable borrow** (`&mut T`)
- Never both at the same time

### No null, no exceptions

Rust has no `null` and no exceptions. Instead:

| Instead of...       | Rust uses...           | Meaning                        |
|---------------------|------------------------|--------------------------------|
| `null`              | `Option<T>`            | Either `Some(value)` or `None` |
| thrown exception    | `Result<T, E>`         | Either `Ok(value)` or `Err(e)` |
| runtime crash       | `panic!`               | Explicit, opt-in crash         |

You'll see `Option` and `Result` constantly in this codebase.

---

## Chapter 2 — Cargo.toml and Crates

Open `Cargo.toml`. It's the manifest for our project — like `package.json` for Node or `requirements.txt` for Python.

```toml
[dependencies]
ratatui = "0.28"
tokio = { version = "1", features = ["full"] }
```

Each entry is a **crate** (Rust's word for a package/library). The version is a [SemVer](https://semver.org) constraint. `"0.28"` means "any 0.28.x version".

**The `features` key** lets you opt into optional parts of a crate. `tokio = { features = ["full"] }` enables the full async runtime. Without it, you'd only get the bare minimum.

Run `cargo add <crate-name>` to add a dependency from the command line.

---

## Chapter 3 — Enums: more than just named numbers

In many languages, enums are just named integers. In Rust, enum variants can carry data — making them incredibly expressive.

### Defining enums (`trivia.rs`)

```rust
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}
```

This is a simple enum, but now look at `QuestionSource`:

```rust
pub enum QuestionSource {
    OpenTDB,
    OpenAI,
    Anthropic,
}
```

And `Screen` in `app.rs` — 14 variants, each representing a different screen. The compiler guarantees that every `match` on `Screen` handles *all* variants; if you add a new screen and forget to handle it somewhere, the code won't compile.

### Pattern matching

`match` is exhaustive. Every variant must be handled:

```rust
let time = match difficulty {
    Difficulty::Easy   => 20,
    Difficulty::Medium => 15,
    Difficulty::Hard   => 10,
};
```

If you add a `Difficulty::Nightmare` variant later, every `match` in the codebase will fail to compile until you handle the new case. This is the Rust compiler acting as a refactoring assistant.

### Option<T> and Result<T, E>

These are just enums with special meaning:

```rust
// Option — value may or may not exist
enum Option<T> {
    Some(T),
    None,
}

// Result — operation may succeed or fail
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

You'll see them everywhere. Common patterns:

```rust
// Unwrap with a fallback
let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

// Early-return on error (the ? operator)
let response = client.get(url).send().await?;
// If the Result is Err, this returns Err immediately from the current function.
// If it's Ok, it unwraps to the value.

// Pattern matching on Option
if let Some(q) = self.current_question() {
    // q is a &Question here
}

// Chaining — map only runs if Some/Ok
let correct_index = options.iter().position(|s| *s == correct).unwrap_or(0);
```

---

## Chapter 4 — Structs and impl blocks

A `struct` groups related data. An `impl` block adds methods to it.

```rust
pub struct Question {
    pub text: String,
    pub category: String,
    pub difficulty: Difficulty,
    pub options: Vec<String>,
    pub correct_index: usize,
}
```

The `pub` keyword means "visible outside this module." Fields without `pub` are private.

```rust
impl Difficulty {
    pub fn time_limit_secs(&self) -> u64 {
        match self {
            Difficulty::Easy   => 20,
            Difficulty::Medium => 15,
            Difficulty::Hard   => 10,
        }
    }
}
```

`&self` is a method that borrows the receiver (read-only). `&mut self` would allow mutation. `self` would consume/take ownership.

---

## Chapter 5 — Traits: shared behaviour

Traits are Rust's equivalent of interfaces. You've seen them via `#[derive(Debug, Clone, PartialEq)]` — that attribute auto-generates trait implementations.

| Trait         | What it adds                         |
|---------------|--------------------------------------|
| `Debug`       | `{:?}` formatting for `println!`     |
| `Clone`       | `.clone()` to make a copy            |
| `PartialEq`   | `==` comparison                      |
| `Serialize`   | Convert struct → JSON (via serde)    |
| `Deserialize` | Convert JSON → struct (via serde)    |
| `Display`     | `{}` formatting (you implement this) |

Here's the manually implemented `Display` trait in `trivia.rs`:

```rust
impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

After this, you can write `format!("{}", difficulty)` or `println!("{}", difficulty)`.

---

## Chapter 6 — Async / Await

Rust's async system lets you write concurrent code that *looks* sequential.

### Why async?

When you make an HTTP request, your program would normally block — doing nothing while waiting for the network. Async lets other work happen during that wait.

### async fn

In `trivia.rs`:

```rust
pub async fn fetch_opentdb_questions(
    category_id: u32,
    difficulty: &Difficulty,
    count: u32,
) -> Result<Vec<Question>, String> {
    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await       // ← suspend here until the network responds
        .map_err(|e| format!("Network error: {}", e))?;  // ← return Err if failed
    // ...
}
```

An `async fn` doesn't run immediately when called — it returns a **Future** (a description of work to do). The work only executes when `.await`-ed.

### The tokio runtime

The `#[tokio::main]` attribute in `main.rs` sets up the async runtime that actually drives Futures to completion.

### Spawning background tasks

When we want to fetch questions *without* blocking the game loop, we spawn a task:

```rust
// Create a one-shot channel — like a letter slot
let (tx, rx) = oneshot::channel();
self.questions_rx = Some(rx);  // keep the receiver to check later

tokio::spawn(async move {
    // This runs on the runtime's thread pool, not the main thread
    let result = fetch_opentdb_questions(category, &difficulty, 10).await;
    let _ = tx.send(result);  // drop the result into the slot
});
```

Every 50ms in `tick()`, we check if a result arrived:

```rust
if let Some(mut rx) = self.questions_rx.take() {
    match rx.try_recv() {
        Ok(Ok(questions)) => { /* start the game */ }
        Ok(Err(e))        => { /* show error */ }
        Err(_)            => { self.questions_rx = Some(rx); } // not ready yet
    }
}
```

This is the **channel pattern** — async tasks communicate by passing messages through channels rather than sharing mutable state.

---

## Chapter 7 — Closures and iterators

Closures are anonymous functions. Iterators are lazy chains of transformations over collections. Together they're incredibly expressive.

### Closures

```rust
// |x| ... is a closure — like a lambda
let doubled: Vec<i32> = vec![1, 2, 3]
    .into_iter()
    .map(|x| x * 2)      // closure takes x, returns x*2
    .collect();           // realise the lazy chain into a Vec
// doubled = [2, 4, 6]
```

In `trivia.rs`, parsing questions:

```rust
let questions: Vec<Question> = raw
    .into_iter()           // consume the Vec, produce an iterator
    .filter_map(|q| {      // like map, but returns Option — None entries are dropped
        let text = q["question"].as_str()?.to_string();
        // The ? inside a closure returning Option means "return None if missing"
        Some(Question { text, ... })
    })
    .collect();             // gather into Vec<Question>
```

### Common iterator methods

| Method          | What it does                                  |
|-----------------|-----------------------------------------------|
| `.map(f)`       | Transform each element                        |
| `.filter(p)`    | Keep only elements where predicate is true    |
| `.filter_map(f)`| Map + filter — discard None results           |
| `.enumerate()`  | Yield `(index, value)` pairs                  |
| `.collect()`    | Gather into a collection (`Vec`, `HashMap`…)  |
| `.position(p)`  | Find index of first element matching predicate|
| `.any(p)`       | True if any element matches                   |
| `.chain(other)` | Concatenate two iterators                     |

---

## Chapter 8 — serde: serialisation made easy

`serde` (short for **ser**ialise/**de**serialise) is the standard Rust library for converting between Rust types and data formats like JSON.

In `trivia.rs`, we describe the OpenTDB response shape:

```rust
#[derive(Deserialize)]   // auto-generate JSON → struct parsing
struct OpenTDBQuestion {
    question: String,
    category: String,
    difficulty: String,
    correct_answer: String,
    incorrect_answers: Vec<String>,
}
```

When we call `response.json::<OpenTDBResponse>().await`, reqwest uses serde to parse the JSON into our struct automatically. Field names in JSON must match struct field names (by default; you can rename with `#[serde(rename = "...")]`).

For saving high scores in `scores.rs`:

```rust
#[derive(Serialize, Deserialize)]
pub struct HighScore { ... }

// Save to JSON
let json = serde_json::to_string_pretty(&scores)?;
fs::write(path, json)?;

// Load from JSON
let scores: Vec<HighScore> = serde_json::from_str(&content)?;
```

---

## Chapter 9 — The ratatui pattern

`ratatui` is a *retained-mode* TUI library: you redraw the entire screen every frame (like a game engine), rather than only updating what changed (ratatui handles the diffing internally).

### The draw loop (`main.rs`)

```rust
loop {
    // 1. Draw the current frame
    terminal.draw(|f| ui::render(f, &app))?;

    // 2. Process input (with a 50ms timeout)
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }

    // 3. Update timers and animations
    app.tick();

    if app.should_quit { break; }
}
```

The 50ms timeout means the loop runs ~20 times per second. This is fast enough for smooth animations and responsive input, while using very little CPU.

### Layouts (`ui.rs`)

`Layout` divides a rectangular area into sub-rectangles:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(4),   // exactly 4 rows tall
        Constraint::Min(10),     // at least 10 rows, takes remaining space
        Constraint::Length(2),   // exactly 2 rows tall
    ])
    .split(area);

// chunks[0], chunks[1], chunks[2] are the resulting Rect values
```

### Widgets

Everything you see on screen is a widget. Widgets are created, configured, and then rendered:

```rust
let paragraph = Paragraph::new("Hello, world!")
    .block(Block::default().borders(Borders::ALL).title("My Box"))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Cyan));

f.render_widget(paragraph, chunks[0]);
```

Common widgets used in this project:

| Widget      | Purpose                                        |
|-------------|------------------------------------------------|
| `Paragraph` | Styled text, optionally word-wrapped           |
| `List`      | A vertical list of `ListItem`s                 |
| `Gauge`     | A horizontal progress / fill bar               |
| `Block`     | A bordered container for other widgets         |
| `Table`     | Rows and columns                               |
| `Clear`     | Clears an area (for popup overlays)            |

### Styled text

`Style` controls colour and text modifiers (bold, italic, etc.):

```rust
let span = Span::styled(
    "Correct!",
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD),
);
```

A `Line` is a row of `Span`s. A `Text` is a collection of `Line`s.

---

## Chapter 10 — State machines

The entire game is a **state machine**: the application is always in exactly one `Screen`, and key presses cause transitions between screens.

```
Splash → MainMenu → QuestionSource → CategorySelect → DifficultySelect → Loading → Playing → AnswerReveal → NameInput → GameOver
                  ↘ AIProviderSelect → APIKeyInput → TopicInput ↗              ↕                                         ↓
                                                                      (loops for each question)               HighScores
```

In `app.rs`, `handle_key` dispatches to a screen-specific handler:

```rust
pub fn handle_key(&mut self, key: KeyEvent) {
    match self.screen.clone() {
        Screen::Playing          => self.key_playing(key),
        Screen::CategorySelect   => self.key_category(key),
        // ...
    }
}
```

State machines are a fundamental design pattern for interactive applications. Rust's exhaustive `match` makes them especially safe — if you add a new screen and forget to handle keys for it, the compiler tells you immediately.

---

## Chapter 11 — Error handling patterns

Rust has no exceptions. Errors are just values — either `Result::Err(e)` or expressed via `Option::None`.

### The `?` operator

Inside any function returning `Result`:

```rust
let data = fs::read_to_string("file.txt")?;
// equivalent to:
let data = match fs::read_to_string("file.txt") {
    Ok(d)  => d,
    Err(e) => return Err(e.into()),
};
```

### `map_err` — converting error types

```rust
let response = client.get(url).send().await
    .map_err(|e| format!("Network error: {}", e))?;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
// Convert reqwest::Error → String, then ? propagates it
```

### `unwrap_or` / `unwrap_or_else`

```rust
let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
// If home_dir() returns None, use "." instead.
// unwrap_or_else takes a closure (evaluated lazily — only if None).
```

---

## Chapter 12 — Modules and visibility

Each `.rs` file is a module. In `main.rs` we declare them:

```rust
mod app;
mod ascii;
mod scores;
mod trivia;
mod ui;
```

`pub` makes items visible outside their module. Without `pub`, they're private to the module they're defined in.

```rust
// In trivia.rs
pub struct Question { ... }  // visible everywhere
pub const CATEGORIES: &[Category] = &[...]; // visible everywhere

fn decode_html(s: &str) -> String { ... }  // private — only used inside trivia.rs
```

In Rust, the module system is your primary tool for encapsulation. Keep implementation details private; only expose what callers need.

---

## Ideas for extending the project

Once you've read through the code and have it running, here are some ways to deepen your understanding by adding features:

**Beginner additions:**
- Add a "streak bonus" — extra points for consecutive correct answers
- Add more ASCII art for different score milestones
- Let the player choose how many questions (5, 10, 20) before the game starts

**Intermediate additions:**
- Add sound effects using the `rodio` crate
- Add a `--category` CLI argument using `clap` so players can start directly in a category
- Persist the player's preferred difficulty in a config file

**Advanced additions:**
- Add a multiplayer mode using `tokio::net::TcpStream` where two players connect over a network
- Add an animated timer bar that changes colour as time runs out (the Gauge widget already supports this — the colour logic is in `draw_playing`)
- Replace the fixed 10-question count with timed rounds (play as many questions as you can in 2 minutes)

---

## Debugging tips

**Print debugging** — add `println!` or `eprintln!` to stderr so output doesn't interfere with the TUI:
```rust
eprintln!("DEBUG: question = {:?}", question);
```

**Compiler error messages** — Rust's compiler errors are famously helpful. Read them carefully; they usually include the fix.

**`cargo check` vs `cargo build`** — use `cargo check` during development to get errors quickly without linking a binary.

**`dbg!` macro** — prints the expression and its value to stderr, then returns the value:
```rust
let x = dbg!(2 + 2); // prints: [src/main.rs:5] 2 + 2 = 4
```

---

## Further reading

- [The Rust Book](https://doc.rust-lang.org/book/) — the official, free, comprehensive guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — learn by reading small working programs
- [Rustlings](https://github.com/rust-lang/rustlings) — small exercises, great for practising syntax
- [ratatui documentation](https://docs.rs/ratatui) — full API docs and examples
- [Tokio tutorial](https://tokio.rs/tokio/tutorial) — deep dive into async Rust

Happy coding!
