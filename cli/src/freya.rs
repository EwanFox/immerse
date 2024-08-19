#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use freya::prelude::*;
use freya::components::Button;
use reqwest::Url;

use crate::db::{new_cards, KanjiSrs};

pub fn start_ui() {
    launch_with_props(app, "Immerse", (700., 600.))
}




fn app() -> Element {

    let mut current: Signal<char> = use_signal(|| ' ');
    let n = new_cards();
    if n.is_err() {
        println!("{}", n.as_ref().unwrap_err());
    }
    let mut new: Signal<Vec<KanjiSrs>> = use_signal(|| n.unwrap_or_else(|_| vec!()));

    if let Some(kanji_vec) = new.pop() {
        current.set(kanji_vec.kanji);
    }

    rsx!(
        rect {
            height: "50%",
            width: "100%",
            main_align: "center",
            cross_align: "center",
            background: "rgb(0, 119, 182)",
            color: "white",
            shadow: "0 4 20 5 rgb(0, 0, 0, 80)",
            label {
                font_size: "75",
                font_weight: "bold",
                "{current}"
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
            }
        }
    )
}