use fsrs::{Card, Rating, ReviewLog, State};
use rusqlite::{params, Connection};
use chrono::{TimeZone, Utc};
use crate::{db::{connect, KanjiSrs}, CliError};

pub struct CardManager {
    connection: Connection,
    next_id: u32
}

impl CardManager {
    pub fn new() -> Result<Self, CliError> {
        let mut manager = CardManager {
            connection: connect()?,
            next_id: 0
        };
        manager.ensure_card_db()?;
        manager.next_id = manager.connection.query_row(
            "SELECT IFNULL(MAX(id), 0) + 1 FROM srs",
            [],
            |row| row.get(0),
        )?;
        Ok(manager)
    }

    pub fn ensure_card_db(&self) -> Result<(), CliError> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS srs (
                kanji TEXT NOT NULL PRIMARY KEY,
                id INTEGER UNIQUE,
                due INTEGER,
                stability REAL,
                difficulty REAL,
                elapsed INTEGER,
                scheduled INTEGER,
                lapses INTEGER,
                reps INTEGER,
                state INTEGER,
                last_review INTEGER,
                prev_state INTEGER
            )",
            [],
        )?;
        self.connection.execute("CREATE TABLE IF NOT EXISTS revlog (
            kanji TEXT NOT NULL PRIMARY KEY,
            rating INTEGER,
            elapsed INTEGER,
            scheduled INTEGER,
            state INTEGER,
            reviewed INTEGER
        )",[])?;
        Ok(())
    }

    pub fn create_card(&mut self, c: KanjiSrs) -> Result<(), CliError> {
        let mut q = self.connection.prepare("INSERT OR IGNORE INTO srs (kanji, due, stability, difficulty, elapsed, scheduled, lapses, reps, state, last_review, prev_state, id) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)")?;
        let id = self.next_id;
        let _card = q.execute(params![c.kanji.to_string(),c.card.due.timestamp(),c.card.stability as f64, c.card.difficulty, c.card.elapsed_days, c.card.scheduled_days, c.card.lapses, c.card.reps, c.card.state as u8, c.card.last_review.timestamp(), c.card.previous_state as u8, id])?;
        if c.card.log.is_some() {
            let log = c.card.log.unwrap();
            let _revlog = self.connection.execute("INSERT OR IGNORE INTO revlog (kanji, rating, elapsed, scheduled, state, reviewed) VALUES (?,?,?,?,?,?)",params![c.kanji.to_string(),log.rating as u8,log.elapsed_days,log.scheduled_days,log.state as u8,log.reviewed_date.timestamp()])?;
        }
        self.next_id+=1;
        Ok(())
    }

    pub fn new_cards(&self, limit: u8) -> Result<Vec<KanjiSrs>, CliError> {
        let mut res = self.connection.prepare("SELECT kanji, id, due, stability, difficulty, elapsed, scheduled, lapses, reps, state, last_review, prev_state FROM srs WHERE state = 0 ORDER BY id ASC LIMIT ?")?;
        let mut rows = res.query(params![limit])?;
        let mut cards: Vec<KanjiSrs> = vec!();
        while let Some(row) = rows.next()? {
            let card = KanjiSrs {
                kanji: row.get::<usize, String>(0)?.chars().next().expect(""),
                card: Card {
                    due: Utc.timestamp(row.get(2)?, 0),
                    stability: row.get(3)?,
                    difficulty: row.get(4)?,
                    elapsed_days: row.get(5)?,
                    scheduled_days: row.get(6)?,
                    lapses: row.get(7)?,
                    reps: row.get(8)?,
                    state: state_u8(row.get(9)?),
                    last_review: Utc.timestamp(row.get(10)?,0),
                    previous_state: state_u8(row.get(11)?),
                    log: self.revlog(row.get(0)?)?,
                },

            };
            cards.push(card);
        }
        Ok(cards)
    }

    fn revlog(&self, kanji: String) -> Result<Option<ReviewLog>, CliError> {
        let mut stmt = self.connection.prepare(
            "SELECT kanji, rating, elapsed, scheduled, state, reviewed
             FROM revlog
             WHERE kanji = ?1"
        )?;
        let mut rows = stmt.query(params![kanji])?;

        if let Some(row) = rows.next()? {
            Ok(Some(ReviewLog {
                rating: rating_u8(row.get(1)?),
                elapsed_days: row.get(2)?,
                scheduled_days: row.get(3)?,
                state: state_u8(row.get(4)?),
                reviewed_date: Utc.timestamp(row.get(5)?,0),
            }))
        } else {
            Ok(None)
        }

    }
}


fn state_u8(i: u8) -> State {
    match i {
        0 => State::New,
        1 => State::Learning,
        2 => State::Review,
        3 => State::Relearning,
        _ => {
            println!("Invalid values have been inserted into the db!");
            State::New
        }
    }
}

fn rating_u8(i: u8) -> Rating {
    match i {
        0 => Rating::Again,
        1 => Rating::Hard,
        2 => Rating::Good,
        3 => Rating::Easy,
        _ => {
            println!("Invalid values have been inserted into the db!");
            Rating::Again
        }
    }
}