use html_escape::decode_html_entities;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriviaSource {
    OpenTriviaDB,
    OpenAI,
    Anthropic,
}

impl TriviaSource {
    pub fn as_str(&self) -> &str {
        match self {
            TriviaSource::OpenTriviaDB => "Open Trivia DB",
            TriviaSource::OpenAI => "OpenAI",
            TriviaSource::Anthropic => "Anthropic",
        }
    }

    pub fn all() -> &'static [TriviaSource] {
        &[TriviaSource::OpenTriviaDB, TriviaSource::OpenAI, TriviaSource::Anthropic]
    }
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
            Difficulty::Easy => 10,
            Difficulty::Medium => 5,
            Difficulty::Hard => 3,
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

    /// Returns all difficulties as a static slice (safe to store on `App`).
    pub fn all() -> &'static [Difficulty] {
        const DIFFICULTIES: [Difficulty; 3] =
            [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        &DIFFICULTIES
    }
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<String> for Difficulty {
    fn from(difficulty: String) -> Self {
        match difficulty.as_str() {
            "easy" => Difficulty::Easy,
            "medium" => Difficulty::Medium,
            "hard" => Difficulty::Hard,
            _ => Difficulty::Easy,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Category {
    pub id: u32,
    pub name: &'static str,
}

pub const CATEGORIES: &[Category] = &[
    Category {
        id: 0,
        name: "Any Category",
    },
    Category {
        id: 9,
        name: "General Knowledge",
    },
    Category {
        id: 10,
        name: "Entertainment: Books",
    },
    Category {
        id: 11,
        name: "Entertainment: Film",
    },
    Category {
        id: 12,
        name: "Entertainment: Music",
    },
    Category {
        id: 13,
        name: "Entertainment: Musicals & Theatres",
    },
    Category {
        id: 14,
        name: "Entertainment: Television",
    },
    Category {
        id: 15,
        name: "Entertainment: Video Games",
    },
    Category {
        id: 16,
        name: "Entertainment: Board Games",
    },
    Category {
        id: 17,
        name: "Science & Nature",
    },
    Category {
        id: 18,
        name: "Science: Computers",
    },
    Category {
        id: 19,
        name: "Science: Mathematics",
    },
    Category {
        id: 20,
        name: "Mythology",
    },
    Category {
        id: 21,
        name: "Sports",
    },
    Category {
        id: 22,
        name: "Geography",
    },
    Category {
        id: 23,
        name: "History",
    },
    Category {
        id: 24,
        name: "Politics",
    },
    Category {
        id: 25,
        name: "Art",
    },
    Category {
        id: 26,
        name: "Celebrities",
    },
    Category {
        id: 27,
        name: "Animals",
    },
    Category {
        id: 28,
        name: "Vehicles",
    },
    Category {
        id: 29,
        name: "Entertainment: Comics",
    },
    Category {
        id: 30,
        name: "Science: Gadgets",
    },
    Category {
        id: 31,
        name: "Entertainment: Japanese Anime & Manga",
    },
    Category {
        id: 32,
        name: "Entertainment: Cartoon & Animations",
    },
];

impl Category {
    /// Returns all the categories in a static array
    pub fn all() -> &'static [Category] {
        CATEGORIES
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    MultipleChoice,
    TrueFalse,
}

impl From<String> for QuestionType {
    fn from(question_type: String) -> Self {
        match question_type.as_str() {
            "multiple" => QuestionType::MultipleChoice,
            "boolean" => QuestionType::TrueFalse,
            _ => QuestionType::MultipleChoice,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Question {
    #[serde(rename = "type")]
    pub question_type: QuestionType,
    pub category: Category,
    pub difficulty: Difficulty,
    pub question: String,
    pub answers: Vec<String>,
    pub correct_answer: String,
}

impl Question {
    /// Returns a new question
    pub fn new(
        question_type: QuestionType,
        category: Category,
        difficulty: Difficulty,
        question: String,
        answers: Vec<String>,
        correct_answer: String,
    ) -> Self {
        Self {
            question_type,
            category,
            difficulty,
            question,
            answers,
            correct_answer,
        }
    }

    /// Returns the index of the correct answer
    pub fn correct_answer_index(&self) -> usize {
        self.answers
            .iter()
            .position(|answer| answer == &self.correct_answer)
            .unwrap()
    }
}

#[derive(Deserialize)]
struct OpenTDBResponse {
    response_code: u32,
    results: Vec<OpenTDBQuestion>,
}

#[derive(Deserialize)]
struct OpenTDBQuestion {
    #[serde(rename = "type")]
    question_type: String,
    difficulty: String,
    category: String,
    question: String,
    correct_answer: String,
    incorrect_answers: Vec<String>,
}

impl From<OpenTDBQuestion> for Question {
    fn from(question: OpenTDBQuestion) -> Self {
        // Find the category by name
        let category = find_category_by_name(&question.category);

        let correct_answer = decode_html(&question.correct_answer);
        let mut incorrect_answers: Vec<String> = question
            .incorrect_answers
            .into_iter()
            .map(|answer| decode_html(&answer))
            .collect();
        incorrect_answers.push(correct_answer.clone());
        incorrect_answers.shuffle(&mut thread_rng());

        Question::new(
            QuestionType::from(question.question_type),
            category,
            Difficulty::from(question.difficulty),
            decode_html(&question.question),
            incorrect_answers,
            correct_answer,
        )
    }
}

/// Finds a category by name from the static array of categories
fn find_category_by_name(name: &str) -> Category {
    Category::all()
        .iter()
        .find(|category| category.name == name)
        .copied()
        .unwrap_or(Category {
            id: 0,
            name: "Any Category",
        })
}

/// Fetches questions from the OpenTDB API
pub async fn fetch_questions_from_opentdb(
    category: Category,
    difficulty: Difficulty,
    count: u32,
) -> Result<Vec<Question>, String> {
    let mut url = format!(
        "https://opentdb.com/api.php?amount={}&difficulty={}",
        count,
        difficulty.as_str()
    );

    if category.id != 0 {
        url += &format!("&category={}", category.id);
    }

    // Get the response from the API
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to fetch questions: {}", e))?;

    // Get the body of the response
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to fetch questions: {}", e))?;

    // Parse the response into a vector of questions
    let trivia_db_response: OpenTDBResponse =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse questions: {}", e))?;

    // Check if the response code is not 0
    if trivia_db_response.response_code != 0 {
        return Err(format!(
            "Failed to fetch questions: {}",
            trivia_db_response.response_code
        ));
    }

    // Convert the vector of OpenTDB questions to a vector of questions
    let questions = trivia_db_response
        .results
        .into_iter()
        .map(Question::from)
        .collect();
    Ok(questions)
}

/// Decodes HTML entities in a string
fn decode_html(s: &str) -> String {
    decode_html_entities(s).to_string()
}
