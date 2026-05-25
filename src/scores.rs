use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HighScore {
    pub initials: String,
    pub score: u32,
    pub date: String,
}

// Implement the HighScore struct to create a new high score
impl HighScore {
    pub fn new(initials: String, score: u32) -> Self {
        Self { initials, score, date: today_string() }
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

    // Sort the scores by score in descending order
    scores.sort_by_key(|s| s.score);

    // Keep only the top 10 scores
    scores = scores.into_iter().rev().take(10).collect();


    save_high_scores(scores)?;
    Ok(())
}

pub fn is_high_score(scores: &[HighScore], score: u32) -> bool {
    scores.len() < 10 || scores.iter().any(|saved| score >= saved.score)
}

fn today_string() -> String {
    let now = Utc::now();
    now.format("%Y-%m-%d").to_string()
}