
use std::panic;

use wasm_bindgen::prelude::*;
use console_error_panic_hook;

use crate::game::run_game;

mod data;
mod game;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    //
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    //
    run_game()
}