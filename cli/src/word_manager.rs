use rusqlite::{params, Connection};
use crate::{db::connect, CliError};

//Helper for managing word database
pub struct WordManager {
    connection: Connection
}

#[derive(Clone)]
pub struct Word {
    pub word: String,
    pub def: String,
    pub furigana: String
}

impl WordManager {
    pub fn new() -> Result<Self, CliError> {
        let manager = WordManager {
            connection: connect()?
        };
        manager.ensure_word_db()?;
        return Ok(manager)
    }

    pub fn ensure_word_db(&self) -> Result<(), CliError> {
        self.connection.execute("
        CREATE TABLE IF NOT EXISTS words (
        id INTEGER NOT NULL PRIMARY KEY,
        word TEXT NOT NULL,
        def TEXT NOT NULL,
        furigana TEXT NOT NULL)",[])?;
        Ok(())
    }

    pub fn insert_or_update_word(&self, id: u64, word: &String, def: &String, furigana: &String) -> Result<(), CliError> {
        self.connection.execute("INSERT INTO words (id, word, def, furigana)
        VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET
            word = excluded.word,
            def = excluded.def,
            furigana = excluded.furigana", params![id, word, def, furigana])?;
        Ok(())
    }

    pub fn find_words(&self, kanji: String) -> Result<Vec<Word>,CliError> {
        let mut query = self.connection.prepare("SELECT word, def, furigana FROM words WHERE word LIKE ?")?;
        let mut rows = query.query(params![format!("%{}%", kanji)])?;
        let mut words: Vec<Word> = vec![];
        while let Some(row) = rows.next()? {
            words.push(Word { word: row.get(0)?, def: row.get(1)?, furigana: row.get(2)? })
        }
        Ok(words)
    }


}

