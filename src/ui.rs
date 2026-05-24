// All rendering logic. `render()` is called every frame from main.rs (JIB-100).

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, BorderType, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table,
        Wrap,
    },
};

use crate::app::{App, Screen};
use crate::ascii;

const COL_PRIMARY: Color = Color::Cyan;
const COL_ACCENT: Color = Color::Yellow;
const COL_CORRECT: Color = Color::Green;
const COL_WRONG: Color = Color::Red;
const COL_DIM: Color = Color::DarkGray;
const COL_HIGHLIGHT: Color = Color::White;

const MS_PER_TICK: u64 = 50;

fn secs_to_ticks(secs: u64) -> u64 {
    secs * 1000 / MS_PER_TICK
}

/// Draw the current screen from `app` state.
pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();

    match app.screen {
        Screen::Splash => draw_splash(f, area),
        Screen::MainMenu => draw_main_menu(f, app, area),
        Screen::QuestionSource => draw_placeholder(f, "Question Source", area),
        Screen::CategorySelect => draw_category_select(f, app, area),
        Screen::DifficultySelect => draw_difficulty_select(f, app, area),
        Screen::Loading => draw_loading(f, app, area),
        Screen::Playing => draw_playing(f, app, area),
        Screen::AnswerReveal => draw_answer_reveal(f, app, area),
        Screen::NameInput => draw_name_input(f, app, area),
        Screen::GameOver => draw_game_over(f, app, area),
        Screen::HighScores => draw_high_scores(f, app, area),
    }
}

fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COL_PRIMARY))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(COL_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
}

fn centred_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

fn hint_line(hints: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (key, desc) in hints {
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default().fg(COL_ACCENT).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}  "),
            Style::default().fg(COL_DIM),
        ));
    }
    Line::from(spans)
}

fn selected_list_item(label: &str, selected: bool) -> ListItem<'static> {
    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(COL_PRIMARY)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(COL_HIGHLIGHT)
    };
    let prefix = if selected { "▶ " } else { "  " };
    ListItem::new(Line::from(Span::styled(format!("{prefix}{label}"), style)))
}

fn draw_splash(f: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(55),
            Constraint::Percentage(30),
        ])
        .split(area);

    let art = Paragraph::new(ascii::TITLE_ART)
        .alignment(Alignment::Center)
        .style(Style::default().fg(COL_PRIMARY).add_modifier(Modifier::BOLD));
    f.render_widget(art, chunks[1]);

    let prompt = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to start",
            Style::default().fg(COL_ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("q / Esc to quit", Style::default().fg(COL_DIM))),
    ])
    .alignment(Alignment::Center);
    f.render_widget(prompt, chunks[2]);
}

fn draw_main_menu(f: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Jibjack Trivia");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(inner);

    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, label)| selected_list_item(label, i == app.menu_cursor))
        .collect();

    f.render_widget(List::new(items), chunks[0]);
    f.render_widget(
        Paragraph::new(hint_line(&[
            ("↑↓", "Navigate"),
            ("Enter", "Select"),
            ("q", "Quit"),
        ]))
        .alignment(Alignment::Center),
        chunks[1],
    );
}

fn draw_placeholder(f: &mut Frame, title: &str, area: Rect) {
    let block = styled_block(title);
    let p = Paragraph::new("Coming soon")
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn draw_category_select(f: &mut Frame, app: &mut App, area: Rect) {
    let block = styled_block("Choose Category");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| selected_list_item(cat.name, i == app.category_cursor))
        .collect();


    let list = List::new(items);

    app.category_list_state.select(Some(app.category_cursor));
    f.render_stateful_widget(list, chunks[0], &mut app.category_list_state);

    f.render_widget(
        Paragraph::new(hint_line(&[
            ("↑↓", "Navigate"),
            ("Enter", "Select"),
            ("Esc", "Back"),
        ]))
        .alignment(Alignment::Center),
        chunks[1],
    );
}

fn draw_difficulty_select(f: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Choose Difficulty");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(inner);

    let items: Vec<ListItem> = app
        .difficulties
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let label = format!(
                "{} — {}s, {} pts",
                d.as_str(),
                d.time_limit_secs(),
                d.points_value()
            );
            selected_list_item(&label, i == app.option_cursor)
        })
        .collect();

    f.render_widget(List::new(items), chunks[0]);
    f.render_widget(
        Paragraph::new(hint_line(&[
            ("↑↓", "Navigate"),
            ("Enter", "Start"),
            ("Esc", "Back"),
        ]))
        .alignment(Alignment::Center),
        chunks[1],
    );
}

fn draw_loading(f: &mut Frame, app: &App, area: Rect) {
    let popup = centred_rect(52, 9, area);

    if let Some(err) = &app.loading_error {
        let text = Text::from(vec![
            Line::from(Span::styled(
                "Error",
                Style::default().fg(COL_WRONG).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(err.clone()),
            Line::from(""),
            Line::from(Span::styled("Esc — go back", Style::default().fg(COL_DIM))),
        ]);
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text)
                .block(styled_block("Loading"))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            popup,
        );
    } else {
        let frame = ascii::LOADING_FRAMES
            [app.loading_dots % ascii::LOADING_FRAMES.len()];
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("{frame} Fetching questions from Open Trivia DB…"),
                Style::default().fg(COL_PRIMARY).add_modifier(Modifier::BOLD),
            )),
        ]);
        f.render_widget(Clear, popup);
        f.render_widget(
            Paragraph::new(text)
                .block(styled_block("Loading"))
                .alignment(Alignment::Center),
            popup,
        );
    }
}

fn time_fraction_remaining(app: &App) -> f64 {
    let limit_secs = app.difficulty.time_limit_secs() + app.bonus_bank_secs;
    if limit_secs == 0 {
        return 0.0;
    }
    (app.total_secs_remaining() as f64 / limit_secs as f64).clamp(0.0, 1.0)
}

fn time_gauge_label(app: &App) -> String {
    let bank = app.bank_secs_remaining();
    if app.secs_remaining() > 0 {
        if bank > 0 {
            format!("{}s (+{bank}s bank)", app.secs_remaining())
        } else {
            format!("{}s left", app.secs_remaining())
        }
    } else if bank > 0 {
        format!("{bank}s bank")
    } else {
        "0s".to_string()
    }
}

fn draw_playing(f: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Question");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(inner);

    let header = Paragraph::new(format!(
        "Score: {}  |  Q {}/{}",
        app.score,
        app.current_q + 1,
        app.questions.len().max(1)
    ))
    .style(Style::default().fg(COL_DIM));
    f.render_widget(header, chunks[0]);

    let question_text = app
        .current_question
        .as_ref()
        .map(|q| q.question.as_str())
        .unwrap_or("…");
    f.render_widget(
        Paragraph::new(question_text)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(COL_HIGHLIGHT)),
        chunks[1],
    );

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(if time_fraction_remaining(app) < 0.25 {
                    COL_WRONG
                } else {
                    COL_PRIMARY
                })
                .bg(COL_DIM),
        )
        .ratio(time_fraction_remaining(app))
        .label(time_gauge_label(app));
    f.render_widget(gauge, chunks[2]);

    let answers: Vec<ListItem> = app
        .current_question
        .as_ref()
        .map(|q| {
            q.answers
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let label = format!("{}. {}", i + 1, a);
                    selected_list_item(&label, i == app.option_cursor)
                })
                .collect()
        })
        .unwrap_or_default();
    f.render_widget(List::new(answers), chunks[3]);

    f.render_widget(
        Paragraph::new(hint_line(&[
            ("↑↓", "Answer"),
            ("Enter", "Submit"),
            ("Esc", "Menu"),
            ("1-4", "Answer")
        ]))
        .alignment(Alignment::Center),
        chunks[4],
    );
}

fn draw_answer_reveal(f: &mut Frame, app: &App, area: Rect) {
    let popup = centred_rect(50, 10, area);
    let (title, color, art) = if app.last_correct {
        ("Correct!", COL_CORRECT, ascii::CORRECT_ART)
    } else {
        ("Wrong", COL_WRONG, ascii::WRONG_ART)
    };

    let correct = app
        .current_question
        .as_ref()
        .map(|q| q.correct_answer.as_str())
        .unwrap_or("");

    let text = Text::from(vec![
        Line::from(Span::styled(art, Style::default().fg(color))),
        Line::from(""),
        Line::from(Span::styled(
            format!("Answer: {correct}"),
            Style::default().fg(COL_HIGHLIGHT),
        )),
        Line::from(Span::styled(
            format!("Score: {}", app.score),
            Style::default().fg(COL_ACCENT),
        )),
        Line::from(if app.last_correct && app.time_bonus > 0 {
            Span::styled(
                format!("+{} time bonus", app.time_bonus),
                Style::default().fg(COL_CORRECT),
            )
        } else {
            Span::raw("")
        }),
        Line::from(if app.last_correct && app.bank_deposit > 0 {
            Span::styled(
                format!("+{}s to time bank", app.bank_deposit),
                Style::default().fg(COL_ACCENT),
            )
        } else {
            Span::raw("")
        }),
    ]);

    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .title(Span::styled(
                        format!(" {title} "),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    )),
            )
            .alignment(Alignment::Center),
        popup,
    );
}

fn draw_name_input(f: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Enter Initials");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new("Up to 3 letters for the high score board")
            .alignment(Alignment::Center)
            .style(Style::default().fg(COL_DIM)),
        chunks[0],
    );

    let input = Paragraph::new(Span::styled(
        format!("{}_", app.name_input),
        Style::default().fg(COL_ACCENT).add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(COL_PRIMARY)),
    )
    .alignment(Alignment::Center);
    f.render_widget(input, chunks[1]);

    f.render_widget(
        Paragraph::new(hint_line(&[("Enter", "Save"), ("Esc", "Skip")]))
            .alignment(Alignment::Center),
        chunks[2],
    );
}

fn draw_game_over(f: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("Game Over");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Final score: {}", app.score),
            Style::default().fg(COL_ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Correct: {}/{}", app.correct_count, app.questions.len()),
            Style::default().fg(COL_HIGHLIGHT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "r — play again   h — high scores   q — menu",
            Style::default().fg(COL_DIM),
        )),
    ]);

    f.render_widget(
        Paragraph::new(text).alignment(Alignment::Center),
        inner,
    );
}

fn draw_high_scores(f: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("High Scores");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows: Vec<Row> = if app.scores.is_empty() {
        vec![Row::new(vec![Cell::from("No scores yet — play a game!")])]
    } else {
        app.scores
            .iter()
            .enumerate()
            .map(|(i, s)| {
                Row::new(vec![
                    Cell::from(format!("{}", i + 1)),
                    Cell::from(s.initials.clone()),
                    Cell::from(format!("{}", s.score)),
                    Cell::from(s.date.clone()),
                ])
            })
            .collect()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .header(Row::new(vec!["#", "Name", "Score", "Date"]).style(
        Style::default().fg(COL_ACCENT).add_modifier(Modifier::BOLD),
    ));

    f.render_widget(table, inner);
}
