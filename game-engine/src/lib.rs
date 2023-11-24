
use std::panic;

use wasm_bindgen::prelude::*;
use console_error_panic_hook;

mod data;
mod game;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Will make panic messages appear in the browser console.
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    let mut game = game::Game::new()?;
    game.start_main_loop()?;
    game.start_draw_loop()?;
    
    Ok(())
}