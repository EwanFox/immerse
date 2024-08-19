#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use freya::prelude::*;
use freya::components::Button;
use reqwest::Url;
use std::sync::Arc;
use crate::config::read_config;
use crate::CliError;
use crate::{ankiconnect::card_with_kanji, db::{new_cards, KanjiSrs}};

pub fn start_ui() {
    launch_with_props(rapp, "Immerse", (700., 600.))
}

fn rapp() -> Element {
    let app = app();
    match app {
        Ok(el) => return el,
        Err(err) => {
            eprintln!("{}", err);
            return rsx!(
                label {
                    "An error occurred!"
                }
            )
        },
    }
}


fn app() -> Result<Element, CliError> {

    let mut current: Signal<String> = use_signal(|| String::new());
    let mut kanji: Signal<char> = use_signal(|| ' ');
    let n = new_cards();
    if n.is_err() {
        println!("{}", n.as_ref().unwrap_err());
    }
    let mut new: Signal<Vec<KanjiSrs>> = use_signal(|| n.unwrap_or_else(|_| vec!()));

    let conf = Arc::new(read_config()?);


    
    
    Ok(rsx!(
        rect {
            height: "50%",
            width: "100%",
            main_align: "center",
            cross_align: "center",
            background: "rgb(0, 119, 182)",
            color: "white",
            shadow: "0 4 20 5 rgb(0, 0, 0, 80)",
            rect {
                main_align: "center",
                cross_align: "center",
                label {
                    font_size: "25",
                    "{kanji}"
                }
                label {
                    font_size: "75",
                    font_weight: "bold",
                    "{current}"
                }
            }
        }
        /*rect {
            height: "fill",
            direction: "vertical",
            width: "100%",
            main_align: "center"
        }*/
        rect {
            direction: "horizontal",
            width: "100%",
            height: "fill",
            main_align: "center",
            cross_align: "center",
            Button {
                onclick: move |_| {
                    if let Some(kanji_vec) = new.pop() {
                        let conf_clone = Arc::clone(&conf);
                        spawn(async move {
                            let e = card_with_kanji(kanji_vec.kanji, &conf_clone).await;
                            kanji.set(kanji_vec.kanji);
                            match e {
                                Ok(result) => {
                                    *current.write() = result.result[0].fields.get("Word").unwrap().value.clone();
                                },
                                Err(ref err) => {eprintln!("{}", err);},
                            }
                        });
                    }
                },
                label {
                    "Next"
                }
            }
            /*Button {
                label {
                    "Again"
                }
            }
            Button {
                label {
                    "Hard"
                }
            }
            Button {
                label {
                    "Good"
                },
                onclick: |_| {
                    println!("Good");
                }
            }
            Button {
                label {
                    "Easy"
                }
            }*/
        }
    ))
}