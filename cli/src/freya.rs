#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use crate::config::read_config;
use crate::CliError;
use crate::{
    ankiconnect::card_with_kanji,
    db::{new_cards, KanjiSrs},
};
use freya::components::Button;
use freya::prelude::*;
use reqwest::Url;
use std::sync::Arc;

use dioxus_router::prelude::{Outlet, Routable, Router};

fn route() -> Element {
    rsx!(Router::<Route> {})
}

pub fn start_ui() {
    launch_with_props(route, "Immerse", (700., 600.))
}

#[allow(non_snake_case)]
fn Rapp() -> Element {
    let app = app();
    match app {
        Ok(el) => return el,
        Err(err) => {
            eprintln!("{}", err);
            return rsx!(
                label {
                    "An error occurred!"
                }
            );
        }
    }
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {

    #[route("/review")]
    Rapp,
    #[route("/")]
    Home,

}

#[allow(non_snake_case)]
fn Home() -> Element {
    rsx!(
        rect {
            height: "50%",
            width: "100%",
            main_align: "center",
            cross_align: "center",
            Button {
                Link {
                    to: Route::Rapp,
                    label {
                        "Review!"
                    }
                }
            }
        }
    )
}

struct Question {
    word: String,
}

fn app() -> Result<Element, CliError> {
    let mut current: Signal<Option<KanjiSrs>> = use_signal(|| None);
    let n = new_cards();
    if n.is_err() {
        println!("{}", n.as_ref().unwrap_err());
    }
    let mut new: Signal<Vec<KanjiSrs>> = use_signal(|| n.unwrap_or_else(|_| vec![]));

    let conf = Arc::new(read_config()?);

    let q = use_resource(move || {
        let conf_clone = Arc::clone(&conf);
        return async move {
            if let Some(k) = current.read().as_ref() {
                let e = card_with_kanji(k.kanji, &conf_clone).await;
                match e {
                    Ok(result) => {
                        return Ok(Question {
                            word: result.result[0].fields.get("Word").unwrap().value.clone(),
                        })
                    }
                    Err(err) => return Err(err),
                }
            } else {
                return Err(CliError::Custom("Something is messed up".to_string()));
            }
        };
    });

    use_effect(move || {
        current.set(new.pop());
    });

    Ok(rsx!(match &*q.read() {
        Some(Ok(res)) => {
            rsx! {
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

                        }
                        label {
                            font_size: "75",
                            font_weight: "bold",
                            "{res.word}"
                        }
                    }
                }
                rect {
                    height: "50%",
                    width: "100%",
                    main_align: "center",
                    cross_align: "center",
                    background: "rgb(0, 119, 182)",
                    color: "white",
                    shadow: "0 4 20 5 rgb(0, 0, 0, 80)",
                    Button {
                        "Again"
                    }
                }

            }
        }
        Some(Err(err)) => {
            rsx! {
                label {
                    "{err}"
                }
            }
        }
        None => {
            rsx! {
                label {
                    "Loading..."
                }
            }
        }
    }))
}
