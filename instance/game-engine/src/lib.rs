use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    // data/script getters
    pub fn getGameIcon() -> Box<[u8]>;
    pub fn getGameScript() -> String;
    pub fn getAssetData(rowid: u32) -> Box<[u8]>;
    pub fn getEntityScript(rowid: u32) -> String;
    // config getters
    pub fn getProjectConfig() -> String;
    pub fn getGameConfig() -> String;
    pub fn getAssetConfig(rowid: u32) -> String;
    pub fn getEntityConfig(rowid: u32) -> String;
    // web functions
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn run_game() {
    alert(&getGameScript());
}
