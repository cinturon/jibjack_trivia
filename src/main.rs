mod app;
mod ascii;
mod scores;
mod trivia;
mod ui;

fn main() {
    // Temporary: verify the module loads. Remove when ui.rs draws the splash screen.
    println!("{}", ascii::TITLE_ART);

    let high_scores = scores::load_high_scores();

    println!("High scores: {:?}", high_scores);
}
