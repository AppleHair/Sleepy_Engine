
mod webapp;
mod game;

use std::panic;

use crate::webapp::alert;
use crate::game::run_game;

use wasm_bindgen::prelude::*;
use console_error_panic_hook;

#[wasm_bindgen]
pub fn handle_game() {
    //
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    //
    if let Err(err) = run_game() {
        //
        alert(&err);
    }
}