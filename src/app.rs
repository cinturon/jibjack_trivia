use crate::scores::{add_high_score, load_high_scores, HighScore};
use crate::trivia::{Category, Difficulty, Question, fetch_questions_from_opentdb};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::error::Error;
use tokio::sync::oneshot;
use crate::ascii;

const MS_PER_TICK: u64 = 50;
const MENU_ITEMS: &[&str] = &["Play", "High Scores", "Quit"];
/// Max seconds that can be stored in the bonus time bank between questions.
const BONUS_BANK_CAP: u64 = 60;

fn secs_to_ticks(secs: u64) -> u64 {
    secs * 1000 / MS_PER_TICK
}

fn ticks_to_secs(ticks: u64) -> u64 {
    ticks * MS_PER_TICK / 1000
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
    pub difficulty_cursor: usize,
    pub category: Category,
    pub categories: &'static [Category],
    pub category_list_state: ListState,
    pub category_cursor: usize,
    pub questions: Vec<Question>,
    pub current_q: usize,
    pub current_question: Option<Question>,
    pub option_cursor: usize,
    pub score: u32,
    pub correct_count: u32,
    pub answered: bool,
    pub last_correct: bool,
    pub question_time: u64,
    /// Bonus points earned on the last correct answer (for answer-reveal UI).
    pub time_bonus: u32,
    /// Seconds banked from fast answers; extends the clock after the per-question limit.
    pub bonus_bank_secs: u64,
    /// Seconds added to the bank on the last correct answer (for answer-reveal UI).
    pub bank_deposit: u64,
    pub reveal_time: u64,
    pub questions_rx: Option<oneshot::Receiver<Result<Vec<Question>, String>>>,
    pub loading_error: Option<String>,
    pub scores: Vec<HighScore>,
    pub name_input: String,
    pub loading_dots: usize,
    pub loading_dots_tick: usize,
    pub should_quit: bool,
    pub saved_scores_error: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Splash,
            menu_cursor: 0,
            menu_items: MENU_ITEMS,
            difficulty: Difficulty::Medium,
            difficulties: Difficulty::all(),
            difficulty_cursor: 0,
            category: Category::all()[0],
            categories: Category::all(),
            category_list_state: ListState::default(),
            category_cursor: 0,
            scores: load_high_scores().unwrap_or_default(),
            name_input: String::new(),
            should_quit: false,
            questions: vec![],
            current_q: 0,
            current_question: None,
            option_cursor: 0,
            score: 0,
            correct_count: 0,
            answered: false,
            last_correct: false,
            question_time: 0,
            time_bonus: 0,
            bonus_bank_secs: 0,
            bank_deposit: 0,
            reveal_time: 0,
            questions_rx: None,
            loading_error: None,
            loading_dots: 0,
            loading_dots_tick: 0,
            saved_scores_error: None,
        }
    }

    pub fn start_loading(&mut self) {
        self.loading_error = None;
        self.loading_dots = 0;
        self.loading_dots_tick = 0;
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
                    self.category_cursor = 0;
                    self.category_list_state.select(Some(0));
                    self.screen = Screen::CategorySelect;
                }
                1 => {
                    self.scores = load_high_scores().unwrap_or_default();
                    self.screen = Screen::HighScores;
                }
                _ => self.should_quit = true,
            },
            _ => {}
        }
    }

    pub fn handle_category_select_key(&mut self, key: KeyEvent) {
        let max = self.categories.len().saturating_sub(1);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.category_cursor = self.category_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.category_cursor = (self.category_cursor + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.category = self.categories[self.category_cursor];
                self.category_cursor = 0;
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
                self.difficulty_cursor = self.difficulty_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.difficulty_cursor = (self.difficulty_cursor + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.difficulty = self.difficulties[self.difficulty_cursor];
                self.start_loading();
            }
            KeyCode::Esc => {
                self.difficulty_cursor = 0;
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
            KeyCode::Char('1') => {
                self.option_cursor = 0;
            }
            KeyCode::Char('2') => {
                self.option_cursor = 1;
            }
            KeyCode::Char('3') if options_len > 2 => self.option_cursor = 2,
            KeyCode::Char('4') if options_len > 3 => self.option_cursor = 3,
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
                match add_high_score(initials, self.score) {
                    Ok(_) => {
                        self.scores = load_high_scores().unwrap_or_default();
                        self.saved_scores_error = None;
                        self.screen = Screen::GameOver;
                    }
                    Err(e) => {
                        self.saved_scores_error = Some(e.to_string());
                        self.screen = Screen::GameOver;
                    }
                }
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
            KeyCode::Char('h') => {
                self.scores = load_high_scores().unwrap_or_default();
                self.screen = Screen::HighScores;
            }
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
        let limit_ticks = secs_to_ticks(self.difficulty.time_limit_secs());
        let max_ticks = limit_ticks + secs_to_ticks(self.bonus_bank_secs);
        if self.question_time >= max_ticks {
            self.commit_bank_usage();
            self.answered = true;
            self.last_correct = false;
            self.time_bonus = 0;
            self.bank_deposit = 0;
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

    /// Seconds left on the per-question countdown (not including the bank).
    pub fn secs_remaining(&self) -> u64 {
        let limit = self.difficulty.time_limit_secs();
        let elapsed = ticks_to_secs(self.question_time);
        limit.saturating_sub(elapsed)
    }

    /// Seconds still available from the bonus bank on this question.
    pub fn bank_secs_remaining(&self) -> u64 {
        let limit_ticks = secs_to_ticks(self.difficulty.time_limit_secs());
        if self.question_time <= limit_ticks {
            return self.bonus_bank_secs;
        }
        let bank_used = ticks_to_secs(self.question_time - limit_ticks);
        self.bonus_bank_secs.saturating_sub(bank_used)
    }

    /// Combined time left (question clock + bank).
    pub fn total_secs_remaining(&self) -> u64 {
        self.secs_remaining() + self.bank_secs_remaining()
    }

    fn commit_bank_usage(&mut self) {
        let limit_ticks = secs_to_ticks(self.difficulty.time_limit_secs());
        let bank_used = ticks_to_secs(self.question_time.saturating_sub(limit_ticks));
        self.bonus_bank_secs = self.bonus_bank_secs.saturating_sub(bank_used);
    }

    fn deposit_into_bank(&mut self, secs: u64) {
        if secs == 0 {
            self.bank_deposit = 0;
            return;
        }
        let deposit = secs / 2;
        self.bank_deposit = deposit;
        self.bonus_bank_secs = (self.bonus_bank_secs + deposit).min(BONUS_BANK_CAP);
    }

    fn submit_answer(&mut self) {
        let Some(question) = self.current_question.as_ref() else {
            return;
        };

        self.answered = true;
        self.last_correct = self.option_cursor == question.correct_answer_index();

        self.commit_bank_usage();
        if self.last_correct {
            // Base points for difficulty + 1 point per second still on the question clock
            let base = self.difficulty.points_value();
            let bonus = self.secs_remaining() as u32;
            self.time_bonus = bonus;
            self.score += base + bonus;
            self.correct_count += 1;
            self.deposit_into_bank(self.secs_remaining());
        } else {
            self.time_bonus = 0;
            self.bank_deposit = 0;
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
        self.loading_dots_tick += 1;
        if self.loading_dots_tick.is_multiple_of(3) {
            self.loading_dots = (self.loading_dots + 1) % ascii::LOADING_FRAMES.len();
        }
       

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
                    self.bonus_bank_secs = 0;
                    self.bank_deposit = 0;
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
