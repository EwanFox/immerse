mod ankiconnect;
mod config;
mod db;
mod kanji;
mod rocket;
#[cfg(test)]
mod test;
mod tui;
/*
Imports
*/

use ankiconnect::get_card_content;
use ankiconnect::get_cards;
use ankiconnect::get_decks;
use clap::Subcommand;
use config::add_deck;
use config::read_config;
use db::cards_with_status;
use db::due_cards;
use db::ensure_card_db;
use db::wipe_srs_db;
use db::KanjiSrs;
use fsrs::Card;
use kanji::is_kanji;
use kanji::recommended_level;
use rocket::rocket;
use std::fmt;
use std::fs;
use std::io;
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
            CliError::Rocket(ref err) => write!(f, "An error occurred: {}", err.to_string().red()),
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
    Rocket(::rocket::Error),
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

impl From<::rocket::Error> for CliError {
    fn from(err: ::rocket::Error) -> Self {
        CliError::Rocket(err)
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
    Rocket,
    WipeDB,
    ListNewCards,
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
        Commands::Rocket => {
            if let Ok(_) = rocket().launch().await {};
        }
        Commands::WipeDB => match wipe_srs_db() {
            Ok(_) => {
                println!("SRS Data Wiped!")
            }
            Err(ref err) => eprintln!("{}", err),
        },
        Commands::ListNewCards => match cards_with_status(fsrs::State::New) {
            Ok(res) => {
                for card in res.iter() {
                    println!("{}", card.kanji)
                }
            }
            Err(ref err) => eprintln!("{}", err),
        },
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
    let mut field: String = String::new();
    if let Some(deck) = config.decks.into_iter().find(|deck| deck.name == selection) {
        field = deck.word_field
    } else {
        terminal = crate::tui::init()?;
        let nf = terminal
            .selection_list(
                info.result[0].fields.keys().cloned().collect(),
                "Choose a field to parse word from:",
            )
            .await?;
        add_deck(selection, nf.clone())?;
        field = nf;
    }

    //Amount of cards in deck for later processed count
    let len = info.result.len();

    //Make  sure the SRS table exists before we write to it
    ensure_card_db()?;

    // Add the kanji to both the srs and kanji tables
    for card in info.result {
        let word = card.fields.get(&field);
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
                    crate::db::card_to_db(KanjiSrs {
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
