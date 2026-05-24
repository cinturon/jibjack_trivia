use crate::scores::{add_high_score, load_high_scores, HighScore};
use crate::trivia::{Category, Difficulty, Question, fetch_questions_from_opentdb};
use crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;
use tokio::sync::oneshot;

const MS_PER_TICK: u64 = 50;
const MENU_ITEMS: &[&str] = &["Play", "High Scores", "Quit"];

fn secs_to_ticks(secs: u64) -> u64 {
    secs * 1000 / MS_PER_TICK
}

#[derive(Debug)]
pub enum Screen {
    Splash,
    MainMenu,
    QuestionSource,
    CategorySelect,
    DifficultySelect,
    Loading,
    Playing,
    AnswerReveal,
    NameInput,
    GameOver,
    HighScores,
}

pub struct App {
    pub screen: Screen,
    pub menu_cursor: usize,
    pub menu_items: &'static [&'static str],
    pub difficulty: Difficulty,
    pub difficulties: &'static [Difficulty],
    pub category: Category,
    pub categories: &'static [Category],
    pub questions: Vec<Question>,
    pub current_q: usize,
    pub current_question: Option<Question>,
    pub score: u32,
    pub correct_count: u32,
    pub option_cursor: usize,
    pub answered: bool,
    pub last_correct: bool,
    pub question_time: u64,
    pub reveal_time: u64,
    pub questions_rx: Option<oneshot::Receiver<Result<Vec<Question>, String>>>,
    pub loading_error: Option<String>,
    pub scores: Vec<HighScore>,
    pub name_input: String,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Splash,
            menu_cursor: 0,
            menu_items: MENU_ITEMS,
            difficulty: Difficulty::Medium,
            difficulties: Difficulty::all(),
            category: Category::all()[0],
            categories: Category::all(),
            scores: load_high_scores().unwrap_or_default(),
            name_input: String::new(),
            should_quit: false,
            questions: vec![],
            current_q: 0,
            current_question: None,
            score: 0,
            correct_count: 0,
            option_cursor: 0,
            answered: false,
            last_correct: false,
            question_time: 0,
            reveal_time: 0,
            questions_rx: None,
            loading_error: None,
        }
    }

    pub fn start_loading(&mut self) {
        self.loading_error = None;
        self.screen = Screen::Loading;

        let (tx, rx) = oneshot::channel();
        self.questions_rx = Some(rx);

        let category = self.category;
        let difficulty = self.difficulty;

        tokio::spawn(async move {
            let result = fetch_questions_from_opentdb(category, difficulty, 10).await;
            let _ = tx.send(result);
        });
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<(), Box<dyn Error>> {
        match self.screen {
            Screen::Splash => self.handle_splash_key(key),
            Screen::MainMenu => self.handle_main_menu_key(key),
            Screen::CategorySelect => self.handle_category_select_key(key),
            Screen::DifficultySelect => self.handle_difficulty_select_key(key),
            Screen::Loading => self.handle_loading_key(key),
            Screen::Playing => self.handle_playing_key(key),
            Screen::AnswerReveal => {}
            Screen::NameInput => self.handle_name_input_key(key),
            Screen::GameOver => self.handle_game_over_key(key),
            Screen::HighScores => self.handle_high_scores_key(key),
            Screen::QuestionSource => {}
        }
        Ok(())
    }

    pub fn handle_splash_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.screen = Screen::MainMenu;
            }
            _ => {}
        }
    }

    pub fn handle_main_menu_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.menu_cursor = self.menu_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.menu_cursor = (self.menu_cursor + 1).min(MENU_ITEMS.len() - 1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.menu_cursor {
                0 => {
                    self.option_cursor = 0;
                    self.screen = Screen::CategorySelect;
                }
                1 => self.screen = Screen::HighScores,
                _ => self.should_quit = true,
            },
            _ => {}
        }
    }

    pub fn handle_category_select_key(&mut self, key: KeyEvent) {
        let max = self.categories.len().saturating_sub(1);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.option_cursor = self.option_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.option_cursor = (self.option_cursor + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.category = self.categories[self.option_cursor];
                self.option_cursor = 0;
                self.screen = Screen::DifficultySelect;
            }
            KeyCode::Esc => {
                self.screen = Screen::MainMenu;
                self.menu_cursor = 0;
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn handle_difficulty_select_key(&mut self, key: KeyEvent) {
        let max = self.difficulties.len().saturating_sub(1);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.option_cursor = self.option_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.option_cursor = (self.option_cursor + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.difficulty = self.difficulties[self.option_cursor];
                self.start_loading();
            }
            KeyCode::Esc => {
                self.option_cursor = 0;
                self.screen = Screen::CategorySelect;
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn handle_loading_key(&mut self, key: KeyEvent) {
        if self.loading_error.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.loading_error = None;
                    self.screen = Screen::DifficultySelect;
                }
                _ => {}
            }
        }
    }

    pub fn handle_playing_key(&mut self, key: KeyEvent) {
        if self.answered {
            return;
        }

        let options_len = self
            .current_question
            .as_ref()
            .map(|q| q.answers.len())
            .unwrap_or(1)
            .max(1);

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.option_cursor = self.option_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.option_cursor = (self.option_cursor + 1).min(options_len - 1);
            }
            KeyCode::Enter | KeyCode::Char(' ') => self.submit_answer(),
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::MainMenu;
                self.menu_cursor = 0;
            }
            _ => {}
        }
    }

    pub fn handle_name_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                let initials = if self.name_input.trim().is_empty() {
                    "AAA".to_string()
                } else {
                    self.name_input.trim().to_uppercase()
                };
                let _ = add_high_score(initials, self.score);
                self.scores = load_high_scores().unwrap_or_default();
                self.screen = Screen::GameOver;
            }
            KeyCode::Backspace => {
                self.name_input.pop();
            }
            KeyCode::Char(c) if self.name_input.len() < 3 => {
                self.name_input.push(c.to_ascii_uppercase());
            }
            KeyCode::Esc => self.screen = Screen::GameOver,
            _ => {}
        }
    }

    pub fn handle_game_over_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') | KeyCode::Enter => {
                self.option_cursor = 0;
                self.screen = Screen::CategorySelect;
            }
            KeyCode::Char('h') => self.screen = Screen::HighScores,
            KeyCode::Char('q') | KeyCode::Esc => {
                self.screen = Screen::MainMenu;
                self.menu_cursor = 0;
            }
            _ => {}
        }
    }

    pub fn handle_high_scores_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                self.screen = Screen::MainMenu;
                self.menu_cursor = 0;
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        match self.screen {
            Screen::Loading => self.handle_loading_tick(),
            Screen::Playing if !self.answered => self.handle_playing_tick(),
            Screen::AnswerReveal => self.handle_answer_reveal_tick(),
            _ => {}
        }
    }

    pub fn handle_playing_tick(&mut self) {
        self.question_time += 1;
        let limit = secs_to_ticks(self.difficulty.time_limit_secs());
        if self.question_time >= limit {
            self.answered = true;
            self.last_correct = false;
            self.reveal_time = 0;
            self.screen = Screen::AnswerReveal;
        }
    }

    pub fn handle_answer_reveal_tick(&mut self) {
        self.reveal_time += 1;
        if self.reveal_time >= secs_to_ticks(2) {
            self.advance_question();
        }
    }

    fn submit_answer(&mut self) {
        let Some(question) = self.current_question.as_ref() else {
            return;
        };

        self.answered = true;
        self.last_correct = self.option_cursor == question.correct_answer_index();

        if self.last_correct {
            self.score += self.difficulty.points_value();
            self.correct_count += 1;
        }

        self.reveal_time = 0;
        self.screen = Screen::AnswerReveal;
    }

    fn advance_question(&mut self) {
        self.current_q += 1;
        if self.current_q >= self.questions.len() {
            self.name_input.clear();
            self.screen = Screen::NameInput;
            self.current_question = None;
        } else {
            self.current_question = self.questions.get(self.current_q).cloned();
            self.option_cursor = 0;
            self.answered = false;
            self.question_time = 0;
            self.screen = Screen::Playing;
        }
    }

    pub fn handle_loading_tick(&mut self) {
        if let Some(mut questions_rx) = self.questions_rx.take() {
            match questions_rx.try_recv() {
                // If the questions are fetched successfully, start the game
                Ok(Ok(questions)) => {
                    self.questions = questions;
                    self.current_q = 0;
                    self.current_question = self.questions.first().cloned();
                    self.screen = Screen::Playing;
                    self.score = 0;
                    self.correct_count = 0;
                    self.option_cursor = 0;
                    self.answered = false;
                    self.last_correct = false;
                    self.question_time = 0;
                    self.reveal_time = 0;
                    self.questions_rx = None;
                }
                Ok(Err(e)) => {
                    self.loading_error = Some(e);
                    self.questions_rx = None;
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                    self.questions_rx = Some(questions_rx); // keep the receiver to check again later
                }
                Err(_) => {
                    self.loading_error = Some("Question fetch was cancelled".into());
                    self.questions_rx = None;
                }
            }
        }
    }
}
