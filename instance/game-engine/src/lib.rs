
mod webapp;
mod game;

use std::panic;

use crate::webapp::alert;
use crate::game::{run_game, EntityRow};

use wasm_bindgen::prelude::*;
use console_error_panic_hook;

#[wasm_bindgen]
pub fn handle_game() {
    //
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    //
    let error = run_game().err();
    //
    if error.is_some() {
        //
        let info = error.unwrap();
        //
        let s = match info.0 {
            EntityRow::Manager => String::from("\non the script of 'State Manager'"),
            EntityRow::Object(id) => format!("\non the script of the '{}' object", webapp::getEntityName(id)),
            EntityRow::Scene(id) => format!("\non the script of the '{}' scene", webapp::getEntityName(id)),
        };
        //
        alert(&format!("{a}{b}", a = info.1.to_string(), b = s));
    }
}