use serde::{Serialize, Deserialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    
    /// Returns the string representation of the difficulty
    pub fn as_str(&self) -> &str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }

    /// Returns the time limit in seconds for the difficulty
    pub fn time_limit_secs(&self) -> u64 {
        match self {
            Difficulty::Easy => 20,
            Difficulty::Medium => 15,
            Difficulty::Hard => 10,
        }
    }

    /// Returns the points value for the difficulty
    pub fn points_value(&self) -> u32 {
        match self {
            Difficulty::Easy => 100,
            Difficulty::Medium => 200,
            Difficulty::Hard => 300,
        }
    }

    /// Returns all the difficulties in a vector 
    pub fn all() -> Vec<Difficulty> {
        vec![Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize)]
pub struct Category {
    pub id: u32,
    pub name: &'static str,
}

pub const CATEGORIES: &[Category] = &[
    Category { id: 0, name: "Any Category" },
    Category { id: 9, name: "General Knowledge" },
    Category { id: 10, name: "Entertainment: Books" },
    Category { id: 11, name: "Entertainment: Film" },
    Category { id: 12, name: "Entertainment: Music" },
    Category { id: 13, name: "Entertainment: Musicals & Theatres" },
    Category { id: 14, name: "Entertainment: Television" },
    Category { id: 15, name: "Entertainment: Video Games" },
    Category { id: 16, name: "Entertainment: Board Games" },
    Category { id: 17, name: "Science & Nature" },
    Category { id: 18, name: "Science: Computers" },
    Category { id: 19, name: "Science: Mathematics" },
    Category { id: 20, name: "Mythology" },
    Category { id: 21, name: "Sports" },
    Category { id: 22, name: "Geography" },
    Category { id: 23, name: "History" },
    Category { id: 24, name: "Politics" },
    Category { id: 25, name: "Art" },
    Category { id: 26, name: "Celebrities" },
    Category { id: 27, name: "Animals" },
    Category { id: 28, name: "Vehicles" },
    Category { id: 29, name: "Entertainment: Comics" },
    Category { id: 30, name: "Science: Gadgets" },
    Category { id: 31, name: "Entertainment: Japanese Anime & Manga" },
    Category { id: 32, name: "Entertainment: Cartoon & Animations" },
];

impl Category {
    /// Returns all the categories in a static array
    pub fn all() -> &'static[Category] {
        CATEGORIES
    }
}
   

#[derive(Debug, Serialize, Deserialize)]
pub enum QuestionType {
    MultipleChoice,
    TrueFalse,
}

#[derive(Debug)]
pub struct Question {
    pub question_type: QuestionType,
    pub category: Category,
    pub difficulty: Difficulty,
    pub question: String,
    pub wrong_answers: Vec<String>,
    pub correct_answer: String,
}


impl Question {
    /// Returns a new question
    pub fn new(question_type: QuestionType, category: Category, difficulty: Difficulty, question: String, wrong_answers: Vec<String>, correct_answer: String) -> Self {
        Self { question_type, category, difficulty, question, wrong_answers, correct_answer }
    }

    /// Returns all the answers in a vector
    pub fn all_answers(&self) -> Vec<String> {
        let mut answers = self.wrong_answers.clone();
        answers.push(self.correct_answer.clone());
        answers
    }

    /// Returns the index of the correct answer
    pub fn correct_answer_index(&self) -> usize {
        self.all_answers().iter().position(|answer| answer == &self.correct_answer).unwrap()
    }
}