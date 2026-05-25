use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Serialize, Deserialize)]
pub struct HighScore {
    pub initials: String,
    pub score: u32,
    pub date: String,
}

// Implement the HighScore struct to create a new high score
impl HighScore {
    pub fn new(initials: String, score: u32) -> Self {
        Self {
            initials,
            score,
            date: today_string(),
        }
    }
}

pub fn load_high_scores() -> Result<Vec<HighScore>, Box<dyn Error>> {
    // Check if the home directory exists, if not, return an error
    if let Some(home_dir) = dirs::home_dir() {
        let path = home_dir.join(".jibjack_trivia/high_scores.json");
        if path.exists() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut scores: Vec<HighScore> = serde_json::from_reader(reader)?;
            scores.sort_by(|a, b| b.score.cmp(&a.score));
            Ok(scores)
        } else {
            Ok(vec![])
        }
    } else {
        Err(Box::new(std::io::Error::other("Home directory not found")))
    }
}

pub fn save_high_scores(scores: Vec<HighScore>) -> Result<(), Box<dyn Error>> {
    // Check if the home directory exists, if not, return an error, if it does, create the file and save the scores
    if let Some(home_dir) = dirs::home_dir() {
        // Create the directory if it doesn't exist
        let dir = home_dir.join(".jibjack_trivia");
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }

        // Save the scores to the file
        let path = home_dir.join(".jibjack_trivia/high_scores.json");
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &scores)?;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::other("Home directory not found")))
    }
}

pub fn add_high_score(initials: String, score: u32) -> Result<(), Box<dyn Error>> {
    let mut scores = load_high_scores()?;
    scores.push(HighScore::new(initials, score));

    save_high_scores(sorted_high_scores(scores))?;
    Ok(())
}

pub fn is_high_score(scores: &[HighScore], score: u32) -> bool {
    scores.len() < 10 || scores.iter().any(|saved| score >= saved.score)
}

fn sorted_high_scores(mut scores: Vec<HighScore>) -> Vec<HighScore> {
    scores.sort_by(|a, b| b.score.cmp(&a.score));
    scores.into_iter().take(10).collect()
}

fn today_string() -> String {
    let now = Utc::now();
    now.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_top_scores_orders_scores_descending() {
        let scores = vec![score("AAA", 100), score("BBB", 300), score("CCC", 200)];

        let sorted = sorted_high_scores(scores);

        assert_eq!(sorted[0].score, 300);
        assert_eq!(sorted[1].score, 200);
        assert_eq!(sorted[2].score, 100);
    }

    #[test]
    fn sorted_top_scores_limits_to_10() {
        let scores = vec![
            score("AAA", 100),
            score("BBB", 300),
            score("CCC", 200),
            score("DDD", 400),
            score("EEE", 500),
            score("FFF", 600),
            score("GGG", 700),
            score("HHH", 800),
            score("III", 900),
            score("JJJ", 1000),
            score("KKK", 1100),
            score("LLL", 1200),
        ];

        let sorted = sorted_high_scores(scores);

        assert_eq!(sorted.len(), 10);
        assert_eq!(sorted[0].score, 1200);
    }

    fn score(initials: &str, score: u32) -> HighScore {
        HighScore {
            initials: initials.to_string(),
            score,
            date: "2026-05-25".to_string(),
        }
    }

    #[test]
    fn is_high_score_returns_true_if_score_is_greater_than_10th_score() {
        let scores = vec![
            score("AAA", 100),
            score("BBB", 300),
            score("CCC", 200),
            score("DDD", 400),
            score("EEE", 500),
            score("FFF", 600),
            score("GGG", 700),
            score("HHH", 800),
            score("III", 900),
            score("JJJ", 1000),
        ];

        let is_added_high_score = is_high_score(&scores, 101);
        assert!(is_added_high_score);

        let is_added_high_score = is_high_score(&scores, 1100);
        assert!(is_added_high_score);


        let is_added_high_score = is_high_score(&scores, 500);
        assert!(is_added_high_score);

        let is_not_added_high_score = is_high_score(&scores, 50);
        assert!(!is_not_added_high_score);
    }
}
