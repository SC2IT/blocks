use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowState {
    pub width: f32, // logical pixels
    pub height: f32,
    pub x: Option<i32>, // physical pixels, outer position
    pub y: Option<i32>,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: crate::draw::VIEW_WIDTH,
            height: crate::draw::VIEW_HEIGHT,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// options and window state saved between runs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub mode: String,      // the mode's name before there were four, for older versions
    pub game_mode: String, // Normal, Medium, Hard or Extreme
    pub theme: String,
    pub background: String,
    pub blocks: String,
    pub board: String,
    pub music_level: u8,
    pub sound_level: u8,
    pub play_unfocused: bool, // keep the music playing when the window loses focus
    pub low_sensory: bool,    // backgrounds and glitches held still
    pub initials: String,
    pub window: WindowState,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: String::new(),
            game_mode: String::new(),
            theme: String::new(),
            background: String::new(),
            blocks: String::new(),
            board: String::new(),
            music_level: crate::game::DEFAULT_VOLUME_LEVEL,
            sound_level: crate::game::DEFAULT_VOLUME_LEVEL,
            play_unfocused: false,
            low_sensory: false,
            initials: String::new(),
            window: WindowState::default(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        config_path(SETTINGS_FILE)
            .and_then(|path| load_toml(&path))
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(path) = config_path(SETTINGS_FILE) {
            save_toml(&path, self);
        }
    }
}

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "Scott Cummings", "Bitrot")
}

/// the folders Blocks saved to before the rename
fn old_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "Scott Cummings", "Blocks")
}

/// the first time Bitrot runs, bring over the settings and high scores Blocks saved
pub fn carry_over() {
    let (Some(old), Some(new)) = (old_project_dirs(), project_dirs()) else {
        return;
    };
    for (from, to) in [
        (old.config_dir(), new.config_dir()),
        (old.data_dir(), new.data_dir()),
    ] {
        if let Err(e) = copy_new_folder(from, to) {
            log::warn!("could not copy {from:?} to {to:?}: {e}");
        }
    }
}

/// copy the files in from into to, only when to doesn't exist yet
fn copy_new_folder(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    if to.exists() || !from.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn config_path(file: &str) -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.config_dir().join(file))
}

pub fn data_path(file: &str) -> Option<PathBuf> {
    project_dirs().map(|dirs| dirs.data_dir().join(file))
}

/// None if missing or unreadable
pub fn load_toml<T: DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("could not read {path:?}: {e}");
            return None;
        }
    };
    toml::from_str(&text)
        .inspect_err(|e| log::warn!("could not parse {path:?}: {e}"))
        .ok()
}

/// write to a temp file then rename
pub fn save_toml<T: Serialize>(path: &PathBuf, value: &T) {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, toml::to_string(value)?)?;
        fs::rename(&tmp, path)?;
        Ok(())
    })();
    match result {
        Ok(()) => log::info!("saved {path:?}"),
        Err(e) => log::warn!("could not save {path:?}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let settings = Settings {
            mode: "Hard".into(),
            game_mode: "Extreme".into(),
            theme: "Bounce".into(),
            background: "Stars".into(),
            blocks: "Neon".into(),
            board: "Dots".into(),
            music_level: 3,
            sound_level: 7,
            play_unfocused: true,
            low_sensory: true,
            initials: "ABC".into(),
            window: WindowState {
                width: 1280.0,
                height: 960.0,
                x: Some(-20),
                y: Some(40),
                maximized: true,
            },
        };
        let text = toml::to_string(&settings).unwrap();
        assert_eq!(toml::from_str::<Settings>(&text).unwrap(), settings);
    }

    #[test]
    fn carry_over_copies_once() {
        let root = std::env::temp_dir().join(format!("bitrot-carry-{}", std::process::id()));
        let (from, to) = (root.join("Blocks"), root.join("Bitrot"));
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("settings.toml"), "music_level = 2").unwrap();
        copy_new_folder(&from, &to).unwrap();
        assert_eq!(
            fs::read_to_string(to.join("settings.toml")).unwrap(),
            "music_level = 2"
        );
        // once Bitrot has its own folder, Blocks' changes stay out of it
        fs::write(from.join("settings.toml"), "music_level = 7").unwrap();
        copy_new_folder(&from, &to).unwrap();
        assert_eq!(
            fs::read_to_string(to.join("settings.toml")).unwrap(),
            "music_level = 2"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn missing_fields_default() {
        let settings: Settings = toml::from_str("music_level = 2").unwrap();
        assert_eq!(settings.music_level, 2);
        assert_eq!(settings.sound_level, 5);
        // older settings files keep the music focus only
        assert!(!settings.play_unfocused);
        assert!(!settings.low_sensory);
        assert_eq!(settings.window, WindowState::default());
    }
}
