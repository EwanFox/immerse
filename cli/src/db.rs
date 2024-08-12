use std::fs;

use crate::kanji::{is_kanji, KanjiEntry};
use crate::CliError;
use chrono::Utc;
use directories::ProjectDirs;
use fsrs::Card;
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct KanjiSrs {
    pub kanji: char,
    pub card: Card,
}

pub fn add_kanji(kanji: char, level: u8) -> Result<(), CliError> {
    let connection = connect()?;
    connection.execute(
        "
        CREATE TABLE IF NOT EXISTS kanji (
            kanji TEXT NOT NULL PRIMARY KEY,
            level INTEGER
        )",
        [],
    )?;
    if !is_kanji(kanji) {
        return Ok(());
    }

    let mut q = connection.prepare("SELECT kanji, level FROM kanji WHERE kanji = ?")?;
    let mut result = q.query([kanji.to_string()])?;
    match result.next() {
        Ok(r) => {
            if r.is_some() {
                let mut stmt =
                    connection.prepare("UPDATE kanji SET level = MAX(level, ?) WHERE kanji = ?")?;
                stmt.execute((level, kanji.to_string()))?;
            } else {
                let mut stmt =
                    connection.prepare("INSERT INTO kanji (kanji, level) VALUES (?,?)")?;
                stmt.execute((kanji.to_string(), level))?;
            }
        }
        Err(_) => {
            let mut stmt = connection.prepare("INSERT INTO kanji (kanji, level) VALUES (?,?)")?;
            stmt.execute((kanji.to_string(), level))?;
        }
    }
    Ok(())
}

pub fn card_to_db(kanji: KanjiSrs) -> Result<(), CliError> {
    let connection = connect()?;
    if !is_kanji(kanji.kanji) {
        return Ok(());
    }
    let mut q = connection.prepare(
        "
        INSERT INTO srs (kanji, card)
        VALUES (?, ?, ?)
        ON CONFLICT(kanji) DO UPDATE SET
            card = excluded.card
            due = excluded.due
    ",
    )?;
    let bson_data = bson::to_vec(&kanji.card).unwrap();
    q.execute((kanji.kanji.to_string(), bson_data, kanji.card.due.timestamp()))?;
    Ok(())
}

pub fn card_from_db(kanji: char) -> Result<KanjiSrs, CliError> {
    let connection = connect()?;
    let mut stmt = connection.prepare("SELECT card FROM srs WHERE kanji = ?")?;
    let blob: Vec<u8> = stmt.query_row(params![kanji.to_string()], |row| row.get(0))?;
    let card: Card = bson::from_slice(&blob)?;
    Ok(KanjiSrs { kanji, card })
}

pub fn due_cards() -> Result<Vec<KanjiSrs>, CliError> {
    let now = Utc::now().timestamp();
    let conn = connect()?;
    let mut stmt = conn.prepare("SELECT card, kanji FROM srs WHERE due < ?1")?;
    let mut res = stmt.query(params![now])?;
    let mut due: Vec<KanjiSrs> = vec!();
    while let Some(row) = res.next()? {
        let str: String = row.get(1)?;
        let kanji = str.chars().next().ok_or_else(|| CliError::Custom("DB KANJI ERROR".to_string()))?;
        let card_data: Vec<u8> = row.get(0)?; 
        due.push(KanjiSrs {
            kanji,
            card: bson::from_slice(&card_data)?,
        })
    }
    todo!()
}

pub fn ensure_card_db() -> Result<(), CliError> {
    let connection = connect()?;
    connection.execute(
        "CREATE TABLE IF NOT EXISTS srs (
            kanji TEXT NOT NULL PRIMARY KEY,
            card BLOB,
            due INTEGER
        )",
        [],
    )?;
    Ok(())
}

pub fn get_all_kanji() -> Result<Vec<KanjiEntry>, CliError> {
    let connection = connect()?;
    let mut stmt = connection.prepare("SELECT kanji, level FROM kanji")?;
    let kanjivec = stmt.query_map((), |row| {
        Ok(KanjiEntry {
            kanji: row.get(0)?,
            level: row.get(1)?,
        })
    })?;
    let mut kanji = Vec::new();
    for entry in kanjivec {
        kanji.push(entry?);
    }
    Ok(kanji)
}

pub fn kanji_count() -> Result<usize, CliError> {
    let connection = connect()?;
    let mut stmt = connection.prepare("SELECT kanji, level FROM kanji")?;
    let kanjivec = stmt.query_map((), |row| {
        Ok(KanjiEntry {
            kanji: row.get(0)?,
            level: row.get(1)?,
        })
    })?;
    let mut kanji = Vec::new();
    for entry in kanjivec {
        kanji.push(entry?);
    }
    Ok(kanji.len())
}

fn connect() -> Result<Connection, CliError> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Immerse", "Immerse") {
        if !proj_dirs.data_dir().exists() {
            fs::create_dir_all(proj_dirs.data_dir())?;
        }
        let connection = Connection::open(proj_dirs.data_dir().join("data.db"))?;
        Ok(connection)
    } else {
        Err(CliError::Custom("Failed to locate db file!".to_string()))
    }
}
