use std::{fs, io::Write};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::CliError;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub decks: Vec<DeckConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct DeckConfig {
    pub name: String,
    pub word_field: String,
    pub furigana_field: String,
    pub def_field: String
}

pub fn read_config() -> Result<Config, CliError> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Immerse", "Immerse") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir())?;
        }
        if !proj_dirs.data_dir().join("config.json").exists() {
            let mut config_file = fs::File::create(proj_dirs.data_dir().join("config.json"))?;
            let config = Config { decks: vec![] };
            config_file.write_all(serde_json::to_string(&config)?.as_bytes())?;
            return Ok(config);
        }
        let config_content = fs::read_to_string(proj_dirs.data_dir().join("config.json"))?;
        let config: Config = serde_json::from_str(&config_content)?;
        Ok(config)
    } else {
        Err(CliError::Custom("Error writing to data dir".to_string()))
    }
}

pub fn add_deck(deck_name: String, word_field: String, def_field: String, furigana_field: String ) -> Result<(), CliError> {
    let mut config = read_config()?;
    if config.decks.iter().any(|deck| deck.name == deck_name) {
        return Ok(());
    } else {
        config.decks.push(DeckConfig {
            name: deck_name,
            word_field,
            def_field,
            furigana_field
        });
        let data_dir = ProjectDirs::from("com", "Immerse", "Immerse");
        match data_dir {
            Some(dir) => {
                let mut config_file = fs::File::create(dir.data_dir().join("config.json"))?;
                config_file.write_all(serde_json::to_string(&config)?.as_bytes())?;
            }
            None => return Err(CliError::Custom("Error Writing to Data Dir".into())),
        }
    }
    Ok(())
}
