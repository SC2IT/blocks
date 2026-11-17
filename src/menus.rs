use ggez::graphics;

// options menu rows
pub const OPTION_MODE: usize = 0;
pub const OPTION_BACKGROUND: usize = 1;
pub const OPTION_BLOCKS: usize = 2;
pub const OPTION_BOARD: usize = 3;
pub const OPTION_LOW_SENSORY: usize = 4;
pub const OPTION_AUDIO: usize = 5;
pub const OPTION_BACK: usize = 6;

// audio submenu rows
pub const AUDIO_THEME: usize = 0;
pub const AUDIO_MUSIC: usize = 1;
pub const AUDIO_SOUND: usize = 2;
pub const AUDIO_UNFOCUSED: usize = 3;
pub const AUDIO_BACK: usize = 4;

pub trait Menu {
    fn items(&self) -> &Vec<graphics::Text>;
    fn selected(&self) -> usize;
    fn next(&mut self);
    fn previous(&mut self);
    fn reset_selection(&mut self);
}

pub struct MenuList {
    items: Vec<graphics::Text>,
    selected: usize,
}

impl MenuList {
    fn new(entries: &[&str]) -> Self {
        Self {
            items: entries.iter().map(|e| graphics::Text::new(*e)).collect(),
            selected: 0,
        }
    }

    pub fn main() -> Self {
        Self::new(&["Start Game", "Options", "High Scores", "Quit Game"])
    }

    pub fn paused() -> Self {
        Self::new(&["Resume Game", "Options", "Exit to Menu", "Quit Game"])
    }

    pub fn options() -> Self {
        Self::new(&[
            "Mode",
            "Background",
            "Blocks",
            "Board",
            "Low Sensory",
            "Audio",
            "Back",
        ])
    }

    pub fn audio() -> Self {
        Self::new(&["Song", "Music", "Sound", "Unfocused", "Back"])
    }
}

impl Menu for MenuList {
    fn items(&self) -> &Vec<graphics::Text> {
        &self.items
    }
    fn next(&mut self) {
        self.selected = (self.selected + 1) % self.items.len();
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.items.len() - 1
        } else {
            self.selected - 1
        };
    }
    fn reset_selection(&mut self) {
        self.selected = 0;
    }
}
