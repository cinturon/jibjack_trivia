use crate::scores::{add_high_score, load_high_scores, is_high_score, HighScore};
use crate::trivia::{
    Category, Difficulty, Question, TriviaSource, fetch_questions_from_openai,
    fetch_questions_from_opentdb, fetch_questions_from_anthropic, validate_source_config,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use std::error::Error;
use tokio::sync::oneshot;
use crate::ascii;

const MS_PER_TICK: u64 = 50;
const MENU_ITEMS: &[&str] = &["Play", "High Scores", "Quit"];

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
    pub question_source: TriviaSource,
    pub question_sources: &'static [TriviaSource],
    pub question_source_cursor: usize,
    pub questions: Vec<Question>,
    pub current_q: usize,
    pub current_question: Option<Question>,
    pub option_cursor: usize,
    pub score: u32,
    pub earned_high_score: bool,
    pub correct_count: u32,
    pub answered: bool,
    pub last_correct: bool,
    pub question_time: u64,
    pub score_multiplier: u32,
    pub last_multiplier_gain: u32,
    pub reveal_time: u64,
    pub questions_rx: Option<oneshot::Receiver<Result<Vec<Question>, String>>>,
    pub loading_error: Option<String>,
    pub scores: Vec<HighScore>,
    pub name_input: String,
    pub loading_dots: usize,
    pub loading_dots_tick: usize,
    pub should_quit: bool,
    pub saved_scores_error: Option<String>,
    pub last_timed_out: bool,
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
            question_source: TriviaSource::OpenTriviaDB,
            question_sources: TriviaSource::all(),
            question_source_cursor: 0,
            category_cursor: 0,
            scores: load_high_scores().unwrap_or_default(),
            name_input: String::new(),
            should_quit: false,
            questions: vec![],
            current_q: 0,
            current_question: None,
            option_cursor: 0,
            score: 0,
            earned_high_score: false,
            correct_count: 0,
            answered: false,
            last_correct: false,
            question_time: 0,
            score_multiplier: 0,
            last_multiplier_gain: 0,
            reveal_time: 0,
            questions_rx: None,
            loading_error: None,
            loading_dots: 0,
            loading_dots_tick: 0,
            last_timed_out: false,
            saved_scores_error: None,
        }
    }

    pub fn start_loading(&mut self) {
        self.loading_error = None;
        self.loading_dots = 0;
        self.loading_dots_tick = 0;
        self.screen = Screen::Loading;

        if let Err(error) = validate_source_config(self.question_source) {
            self.loading_error = Some(error);
            self.questions_rx = None;
            return;
        }

        let (tx, rx) = oneshot::channel();
        self.questions_rx = Some(rx);

        let category = self.category;
        let difficulty = self.difficulty;
        let question_source = self.question_source;

        tokio::spawn(async move {
            let result = match question_source {
                TriviaSource::OpenTriviaDB => {
                    fetch_questions_from_opentdb(category, difficulty, 10).await
                }
                TriviaSource::OpenAI => fetch_questions_from_openai(category, difficulty, 10).await,
                TriviaSource::Anthropic => fetch_questions_from_anthropic(category, difficulty, 10).await,
            };
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
            Screen::QuestionSource => self.handle_question_source_key(key),
            Screen::Playing => self.handle_playing_key(key),
            Screen::AnswerReveal => {}
            Screen::NameInput => self.handle_name_input_key(key),
            Screen::GameOver => self.handle_game_over_key(key),
            Screen::HighScores => self.handle_high_scores_key(key),
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
                // self.start_loading();
                self.screen = Screen::QuestionSource;
            }
            KeyCode::Esc => {
                self.difficulty_cursor = 0;
                self.screen = Screen::CategorySelect;
            }
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn handle_question_source_key(&mut self, key: KeyEvent) {
        let max = self.question_sources.len().saturating_sub(1);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.question_source_cursor = self.question_source_cursor.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.question_source_cursor = (self.question_source_cursor + 1).min(max);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.question_source = self.question_sources[self.question_source_cursor];
                self.start_loading();
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::DifficultySelect;
            }
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
        // Increment the question time by 50ms or 1 tick
        self.question_time += 1;
        let limit_ticks = secs_to_ticks(self.difficulty.time_limit_secs());
        if self.question_time >= limit_ticks {
            self.answered = true;
            self.last_correct = false;
            self.score_multiplier = 0;
            self.last_multiplier_gain = 0;
            self.reveal_time = 0;
            self.last_timed_out = true;
            self.screen = Screen::AnswerReveal;
        }
    }

    pub fn handle_answer_reveal_tick(&mut self) {
        self.reveal_time += 1;
        if self.reveal_time >= secs_to_ticks(2) {
            self.advance_question();
        }
    }

    pub fn secs_remaining(&self) -> u64 {
        let limit = self.difficulty.time_limit_secs();
        let elapsed = ticks_to_secs(self.question_time);
        limit.saturating_sub(elapsed)
    }

    fn multiplier_gain(&self) -> u32 {
        let elapsed = ticks_to_secs(self.question_time);
        10_u64.saturating_sub(elapsed) as u32
    }

    fn submit_answer(&mut self) {
        self.last_timed_out = false;
        let Some(question) = self.current_question.as_ref() else {
            return;
        };

        self.answered = true;
        self.last_correct = self.option_cursor == question.correct_answer_index();

        if self.last_correct {
            let base = self.difficulty.points_value();
            self.last_multiplier_gain = self.multiplier_gain();
            self.score_multiplier += self.last_multiplier_gain;
            self.score += base + self.score_multiplier;
            self.correct_count += 1;
        } else {
            self.score_multiplier = 0;
            self.last_multiplier_gain = 0;
        }

        self.reveal_time = 0;
        self.screen = Screen::AnswerReveal;
    }

    fn advance_question(&mut self) {
        self.current_q += 1;
        if self.current_q >= self.questions.len() {
            self.earned_high_score = is_high_score(&self.scores, self.score);
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
                    self.score_multiplier = 0;
                    self.last_multiplier_gain = 0;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trivia::QuestionType;

    fn test_question() -> Question {
        Question::new(
            QuestionType::MultipleChoice,
            Category::all()[0],
            Difficulty::Easy,
            "Question?".to_string(),
            vec![
                "Correct".to_string(),
                "Wrong 1".to_string(),
                "Wrong 2".to_string(),
                "Wrong 3".to_string(),
            ],
            "Correct".to_string(),
        )
    }

    fn playing_app() -> App {
        let mut app = App::new();
        app.difficulty = Difficulty::Easy;
        app.current_question = Some(test_question());
        app.screen = Screen::Playing;
        app
    }

    #[test]
    fn correct_answer_adds_speed_to_multiplier_and_scores_base_plus_multiplier() {
        let mut app = playing_app();
        app.question_time = secs_to_ticks(3);

        app.submit_answer();

        assert!(app.last_correct);
        assert_eq!(app.last_multiplier_gain, 7);
        assert_eq!(app.score_multiplier, 7);
        assert_eq!(app.score, 107);
    }

    #[test]
    fn correct_answers_accumulate_multiplier() {
        let mut app = playing_app();
        app.question_time = secs_to_ticks(3);
        app.submit_answer();

        app.answered = false;
        app.current_question = Some(test_question());
        app.question_time = secs_to_ticks(4);
        app.submit_answer();

        assert_eq!(app.last_multiplier_gain, 6);
        assert_eq!(app.score_multiplier, 13);
        assert_eq!(app.score, 220);
    }

    #[test]
    fn wrong_answer_resets_multiplier_without_changing_score() {
        let mut app = playing_app();
        app.score = 107;
        app.score_multiplier = 7;
        app.option_cursor = 1;

        app.submit_answer();

        assert!(!app.last_correct);
        assert_eq!(app.score_multiplier, 0);
        assert_eq!(app.last_multiplier_gain, 0);
        assert_eq!(app.score, 107);
    }

    #[test]
    fn timeout_resets_multiplier() {
        let mut app = playing_app();
        app.score_multiplier = 7;
        app.question_time = secs_to_ticks(app.difficulty.time_limit_secs()) - 1;

        app.handle_playing_tick();

        assert!(!app.last_correct);
        assert!(app.last_timed_out);
        assert_eq!(app.score_multiplier, 0);
        assert_eq!(app.last_multiplier_gain, 0);
    }
}
