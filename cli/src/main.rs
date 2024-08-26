#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]


mod ankiconnect;
mod config;
mod db;
mod kanji;
mod card_manager;
#[cfg(test)]
mod test;
mod tui;
mod freya;
mod word_manager;
/*
Imports
*/

use ankiconnect::get_card_content;
use ankiconnect::get_cards;
use ankiconnect::get_decks;
use card_manager::CardManager;
use clap::Subcommand;
use config::add_deck;
use config::read_config;
use db::cards_with_status;
use db::ensure_card_db;
use db::wipe_srs_db;
use db::KanjiSrs;
use freya::start_ui;
use fsrs::Card;
use kanji::is_kanji;
use kanji::recommended_level;
use word_manager::WordManager;
use std::fmt;
use std::fs;
use std::io;
use std::thread;
use tui::CanHaveKanjiList;
use tui::CanHaveSelection;
//Use Directories crate to get app data dir
use directories::ProjectDirs;
//Using Clap to parse CLI calls
use clap::Parser;
//Using Rusqlite as an sqlite handler
use rusqlite::{Connection, Result};
//Use Colored for prettier text
use colored::Colorize;

/*
Custom Error Type
*/

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::SQL(ref err) => write!(
                f,
                "An error occurred: {}",
                colored::Colorize::red(err.to_string().as_str())
            ),
            CliError::Custom(ref err) => write!(
                f,
                "An error occurred: {}",
                colored::Colorize::red(err.as_str())
            ),
            CliError::IO(ref err) => write!(f, "An error occurred: {}", err.to_string().red()),
            CliError::HTTP(ref err) => write!(f, "An error occurred: {}", err.to_string().red()),
            CliError::JSON(ref err) => write!(f, "An error occurred: {}", err.to_string().red()),
            CliError::BSON(ref err) => write!(f, "An error occurred: {}", err.to_string().red()),
        }
    }
}

#[derive(Debug)]
pub enum CliError {
    SQL(rusqlite::Error),
    IO(io::Error),
    Custom(String),
    HTTP(reqwest::Error),
    JSON(serde_json::Error),
    BSON(bson::de::Error),
}

impl From<rusqlite::Error> for CliError {
    fn from(err: rusqlite::Error) -> Self {
        CliError::SQL(err)
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::IO(err)
    }
}
impl From<bson::de::Error> for CliError {
    fn from(err: bson::de::Error) -> Self {
        CliError::BSON(err)
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::HTTP(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::JSON(err)
    }
}


/*
Command type defs
*/

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    AnkiConnectTest,
    DBRead,
    AnkiSync,
    GetDBKanji,
    KanjiCount,
    WipeDB,
    ListNewCards,
    Review
}

/*
Main Function
*/

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.cmd {
        Commands::AnkiConnectTest => {
            anki_connect_test().await;
        }
        Commands::DBRead => {
            db_read();
        }
        Commands::AnkiSync => match anki_sync().await {
            Ok(_) => {}
            Err(ref err) => eprintln!("{}", err),
        },
        Commands::GetDBKanji => match get_db_kanji().await {
            Ok(_) => {}
            Err(ref err) => eprintln!("{}", err),
        },
        Commands::KanjiCount => match kanji_count() {
            Ok(_) => {}
            Err(ref err) => eprintln!("{}", err),
        },
        Commands::WipeDB => match wipe_srs_db() {
            Ok(_) => {
                println!("SRS Data Wiped!")
            }
            Err(ref err) => eprintln!("{}", err),
        },
        Commands::ListNewCards =>  {
            let manager = CardManager::new().unwrap();
            match manager.new_cards(20) {
                Ok(cards) => {
                    for card in cards {
                        println!("New: {}", card.kanji);
                    }
                },
                Err(ref err) => eprintln!("{}", err),
            }
        },
        Commands::Review => {
            start_ui();

        }
    }
}
/*
Subcommand implementation
*/

async fn get_db_kanji() -> Result<(), CliError> {
    let mut kanji = crate::db::get_all_kanji()?;
    let mut terminal = crate::tui::init()?;
    kanji.sort_by(|a, b| b.level.cmp(&a.level));
    terminal.kanji_list(kanji, "Database Kanji").await?;
    Ok(())
}

fn kanji_count() -> Result<(), CliError> {
    let len = crate::db::kanji_count()?;
    println!(
        "Found {} entries",
        Colorize::green(len.to_string().as_str())
    );
    Ok(())
}

async fn anki_sync() -> Result<(), CliError> {
    //Get list of anki decks
    let decklist = get_decks().await?.result;

    //Ask user to select deck
    let mut terminal = crate::tui::init()?;
    let selection = terminal.selection_list(decklist, "Select A Deck:").await?;

    //Pull the content of all cards in the deck
    println!("Getting cards in deck...");
    let result = get_cards(selection.clone()).await?;
    println!("Getting card content...");
    let info = get_card_content(result.result).await?;

    //Track cards that we can't process
    let mut skipped = 0;

    // Check if we have a stored config field to parse words from.
    //If not, ask user to select one and then save to config.

    let config = read_config()?;
    let mut word_field: String = String::new();
    let mut def_field: String = String::new();
    let mut furigana_field: String = String::new();
    if let Some(deck) = config.decks.into_iter().find(|deck| deck.name == selection) {
        word_field = deck.word_field;
        def_field = deck.def_field;
        furigana_field = deck.furigana_field;
    } else {
        terminal = crate::tui::init()?;
        word_field = terminal
            .selection_list(
                info.result[0].fields.keys().cloned().collect(),
                "Choose a field to parse word from:",
            )
            .await?;
        terminal = crate::tui::init()?;
        def_field = terminal
            .selection_list(
                info.result[0].fields.keys().cloned().collect(),
                "Choose a field to parse definition from:",
            )
            .await?;
        terminal = crate::tui::init()?;
        furigana_field = terminal
            .selection_list(
                info.result[0].fields.keys().cloned().collect(),
                "Choose a field to parse furigana from:",
            )
            .await?;
        add_deck(selection, word_field.clone(), def_field.clone(), furigana_field.clone())?;
    }

    //Amount of cards in deck for later processed count
    let len = info.result.len();

    let mut manager = CardManager::new()?;
    let mut word_manager = WordManager::new()?;
    // Add the kanji to both the srs and kanji tables
    for card in info.result {

        let word = card.fields.get(&word_field);
        let def = card.fields.get(&def_field);
        let furigana = card.fields.get(&furigana_field);
        if word.is_some() && def.is_some() && furigana.is_some() {
            word_manager.insert_or_update_word(card.cardId, &word.unwrap().value, &def.unwrap().value, &furigana.unwrap().value)?;
        } else {
            return Err(CliError::Custom("Card did not contain expected field".to_string()))
        }
        match word {
            Some(word) => {
                for kanji in word.value.chars() {
                    if !is_kanji(kanji) {
                        break;
                    }
                    match crate::db::add_kanji(kanji, recommended_level(card.interval)) {
                        Ok(_) => {}
                        Err(ref err) => println!("{}", err),
                    }
                    let srscard = Card::new();
                    manager.create_card(KanjiSrs {
                        kanji,
                        card: srscard,
                    })?;
                }
            }
            None => {
                skipped += 1;
            }
        }
    }
    println!("Skipped {} cards that couldn't be processed", skipped);
    println!(
        "Synced {} entries",
        Colorize::green((len - skipped).to_string().as_str())
    );
    Ok(())
}

async fn anki_connect_test() {
    match crate::ankiconnect::get_decks().await {
        Ok(_) => {}
        Err(err) => eprintln!("{}", err),
    };
}

fn db_read() {
    let dbread = read_db();
    match dbread {
        Ok(msg) => println!("{}", colored::Colorize::green(msg)),
        Err(err) => eprintln!("{}", err),
    }
}

/*
TODO: Move this
*/

fn read_db() -> Result<&'static str, CliError> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Immerse", "Immerse") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir())?;
        }
        let connection = Connection::open(proj_dirs.data_dir().join("data.db"))?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS kanji (
            kanji TEXT NOT NULL
        )",
            [],
        )?;
        Ok("Successfully located Database file!")
    } else {
        Err(CliError::Custom("Failed to locate db file!".to_string()))
    }
}
