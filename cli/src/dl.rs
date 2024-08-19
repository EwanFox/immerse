use directories::ProjectDirs;
use std::fs::{self, File};
use crate::CliError;
use reqwest;

pub async fn ensure_kanjiindex() -> Result<String, CliError> {
    println!("Downloading KanjiVg Index...");
    if let Some(proj_dirs) = ProjectDirs::from("com", "Immerse", "Immerse") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir())?;
        }
        if !proj_dirs.data_dir().join("kvg-index.json").exists() {
            let client = reqwest::Client::new();
            let response = client.get("https://raw.githubusercontent.com/KanjiVG/kanjivg/master/kvg-index.json").send().await?;
            if response.status().is_success() {
                let path = proj_dirs.data_dir().join("kvg-index.json");
                let mut file = File::create(&path);
                let mut content = response.bytes()?;
            }
        }
    } else {
        Err(CliError::Custom("Error writing to data dir".to_string()))
    }
    todo!()
}