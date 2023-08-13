use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    // data/script getters
    pub fn getMetadataData(rowid: u8) -> Box<[u8]>;
    pub fn getMetadataScript(rowid: u8) -> String;
    pub fn getAssetData(rowid: u32) -> Box<[u8]>;
    pub fn getEntityScript(rowid: u32) -> String;
    // config getters
    pub fn getMetadataConfig(rowid: u8) -> String;
    pub fn getAssetConfig(rowid: u32) -> String;
    pub fn getEntityConfig(rowid: u32) -> String;
    // id to name and vice versa
    pub fn getEntityID(name: &str) -> u32;
    pub fn getAssetID(name: &str) -> u32;
    pub fn getEntityName(id: u32) -> String;
    pub fn getAssetName(id: u32) -> String;
    // window functions
    pub fn alert(s: &str);
    #[wasm_bindgen(js_namespace=console, js_name=log)]
    pub fn log(s: &str);
}