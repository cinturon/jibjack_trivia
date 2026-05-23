// All the ASCII art used throughout the game.
//
// `pub const` declares a compile-time constant visible to other modules.
// `&str` is an immutable string slice — perfect for static text.
//
// `r#"..."#` is a raw string literal: backslashes are literal characters,
// not escape sequences.

/// The main title shown on the splash screen.
pub const TITLE_ART: &str = r#"
      _ _ _     _            _    
     | (_) |__ (_) __ _  ___| | __
  _  | | | '_ \| |/ _` |/ __| |/ /
 | |_| | | |_) | | (_| | (__|   < 
  \___/|_|_.__// |\__,_|\___|_|\_\
             |__/                 

 ████████ ████████  ████ ██     ██ ████    ███    
    ██    ██     ██  ██  ██     ██  ██    ██ ██   
    ██    ██     ██  ██  ██     ██  ██   ██   ██  
    ██    ████████   ██  ██     ██  ██  ██     ██ 
    ██    ██   ██    ██   ██   ██   ██  █████████ 
    ██    ██    ██   ██    ██ ██    ██  ██     ██ 
    ██    ██     ██ ████    ███    ████ ██     ██ 
"#;

/// Shown briefly when the player answers correctly.
pub const CORRECT_ART: &str = r#"
  +-+-+-+-+-+-+-+
  |C|O|R|R|E|C|T|
  +-+-+-+-+-+-+-+
        (•‿•)
"#;

/// Shown briefly when the player answers incorrectly.
pub const WRONG_ART: &str = r#"
  +-+-+-+-+
  |W|R|O|N|G|
  +-+-+-+-+
       (×_×)
"#;

/// Shown briefly when the timer runs out.
pub const TIME_UP_ART: &str = r#"
  _____ ___ __  __ _____   _   _ ____  _
 |_   _|_ _|  \/  | ____| | | | |  _ \| |
   | |  | || |\/| |  _|   | | | | |_) | |
   | |  | || |  | | |___  | |_| |  __/|_|
   |_| |___|_|  |_|_____|  \___/|_|   (_)
"#;

/// Trophy shown on the high scores screen.
pub const TROPHY_ART: &str = r#"
         ___________
        '._==_==_=_.'
        .-\:      /-.
       | (|:.     |) |
        '-|:.     |-'
          \::.    /
           '::. .'
             ) (
           _.' '._
          `"""""""`
"#;

/// Shown on the Game Over screen.
pub const GAME_OVER_ART: &str = r#"
   ____    _    __  __ _____    _____     _______ ____
  / ___|  / \  |  \/  | ____|  / _ \ \   / / ____|  _ \
 | |  _  / _ \ | |\/| |  _|   | | | \ \ / /|  _| | |_) |
 | |_| |/ ___ \| |  | | |___  | |_| |\ V / | |___|  _ <
  \____/_/   \_\_|  |_|_____|  \___/  \_/  |_____|_| \_\
"#;

/// Spinner frames shown during the loading screen.
pub const LOADING_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A small brain icon used in the "AI Questions" menu option.
pub const BRAIN_ART: &str = r#"
    .--.  .--.
   /    \/    \
  | ^  ^ ^  ^ |
  |  \  /\  / |
   \  \/  \/  /
    `--------'
"#;
