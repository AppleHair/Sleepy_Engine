
mod data;
mod game;

use std::panic;

use crate::game::{run_game, ClosuresHandle};

use wasm_bindgen::prelude::*;
use console_error_panic_hook;

#[wasm_bindgen]
pub fn handle_game() -> Result<ClosuresHandle, JsValue> {
    //
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    //
    run_game()
}