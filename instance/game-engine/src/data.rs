
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // data/script getters
    #[wasm_bindgen(js_name=getMetadataIcon)]
    pub fn get_metadata_icon() -> Box<[u8]>;
    #[wasm_bindgen(js_name=getMetadataScript)]
    pub fn get_metadata_script() -> String;
    #[wasm_bindgen(js_name=getAssetData)]
    pub fn get_asset_data(rowid: u32) -> Box<[u8]>;
    #[wasm_bindgen(js_name=getElementScript)]
    pub fn get_element_script(rowid: u32) -> String;
    // config getters
    #[wasm_bindgen(js_name=getMetadataConfig)]
    pub fn get_metadata_config() -> String;
    #[wasm_bindgen(js_name=getAssetConfig)]
    pub fn get_asset_config(rowid: u32) -> String;
    #[wasm_bindgen(js_name=getElementConfig)]
    pub fn get_element_config(rowid: u32) -> String;
    // id to name and vice versa
    #[wasm_bindgen(js_name=getElementID)]
    pub fn get_element_id(name: &str) -> u32;
    #[wasm_bindgen(js_name=getAssetID)]
    pub fn get_asset_id(name: &str) -> u32;
    #[wasm_bindgen(js_name=getElementName)]
    pub fn get_element_name(id: u32) -> String;
    #[wasm_bindgen(js_name=getAssetName)]
    pub fn get_asset_name(id: u32) -> String;
    // type getters
    #[wasm_bindgen(js_name=getElementType)]
    pub fn get_element_type(id: u32) -> u8;
    #[wasm_bindgen(js_name=getAssetType)]
    pub fn get_asset_type(id: u32) -> u8;
    // IDs to load getters
    #[wasm_bindgen(js_name=assetsToLoad)]
    pub fn assets_to_load() -> Box<[JsValue]>;
    #[wasm_bindgen(js_name=elementsToLoad)]
    pub fn elements_to_load() -> Box<[JsValue]>;
}