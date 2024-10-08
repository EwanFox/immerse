use crate::CliError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDecksResult {
    pub result: Vec<String>,
}

pub async fn get_decks() -> Result<GetDecksResult, CliError> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8765")
        .body(
            r#"{
        "action": "deckNames",
        "version": 6
    }"#,
        )
        .send()
        .await?
        .json::<GetDecksResult>()
        .await?;
    Ok(resp)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCardsResult {
    pub result: Vec<u64>,
    pub err: Option<String>,
}

pub async fn get_cards(deck_name: String) -> Result<GetCardsResult, CliError> {
    let client = reqwest::Client::new();
    let body = format!(
        r#"{{
        "action": "findCards",
        "version": 6,
        "params": {{
            "query": "deck:\"{}\""
        }}
    }}"#,
        deck_name
    );
    let resp = client
        .post("http://localhost:8765")
        .body(body)
        .send()
        .await?
        .json::<GetCardsResult>()
        .await?;
    if resp.err.is_some() {
        return Err(CliError::Custom(resp.err.unwrap()));
    }
    Ok(resp)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCardsContentResult {
    pub result: Vec<CardContent>,
    pub err: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CardContent {
    pub interval: u16,
    pub fields: HashMap<String, Field>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Field {
    pub value: String,
    pub order: u8,
}
pub async fn get_card_content(ids: Vec<u64>) -> Result<GetCardsContentResult, CliError> {
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:8765")
        .body(
            json!({
                "action": "cardsInfo",
                "version": 6,
                "params": {
                    "cards": ids
                }
            })
            .to_string(),
        )
        .send()
        .await?
        .json::<GetCardsContentResult>()
        .await?;
    if resp.err.is_some() {
        return Err(CliError::Custom(resp.err.unwrap()));
    }
    Ok(resp)
}
