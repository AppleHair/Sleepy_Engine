use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    // data/script getters
    #[wasm_bindgen(js_name=getMetadataData)]
    pub fn get_metadata_data(rowid: u8) -> Box<[u8]>;
    #[wasm_bindgen(js_name=getMetadataScript)]
    pub fn get_metadata_script(rowid: u8) -> String;
    #[wasm_bindgen(js_name=getAssetData)]
    pub fn get_asset_data(rowid: u32) -> Box<[u8]>;
    #[wasm_bindgen(js_name=getEntityScript)]
    pub fn get_entity_script(rowid: u32) -> String;
    // config getters
    #[wasm_bindgen(js_name=getMetadataConfig)]
    pub fn get_metadata_config(rowid: u8) -> String;
    #[wasm_bindgen(js_name=getAssetConfig)]
    pub fn get_asset_config(rowid: u32) -> String;
    #[wasm_bindgen(js_name=getEntityConfig)]
    pub fn get_entity_config(rowid: u32) -> String;
    // id to name and vice versa
    #[wasm_bindgen(js_name=getEntityID)]
    pub fn get_entity_id(name: &str) -> u32;
    #[wasm_bindgen(js_name=getAssetID)]
    pub fn get_asset_id(name: &str) -> u32;
    #[wasm_bindgen(js_name=getEntityName)]
    pub fn get_entity_name(id: u32) -> String;
    #[wasm_bindgen(js_name=getAssetName)]
    pub fn get_asset_name(id: u32) -> String;
}