use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    pub fn getEntityName(rowid: u32) -> String;
    pub fn getAssetName(rowid: u32) -> String;
    pub fn getMetadataName(rowid: u32) -> String;
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn get_name(num: u32) {
    alert(&format!("{}", getEntityName(num)));
}
