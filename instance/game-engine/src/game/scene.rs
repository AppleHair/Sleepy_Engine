
use rhai::{Engine, Map};

//
#[derive(Clone)]
pub struct Layer {
    name: String,
}

//
impl Layer {
    pub fn get_name(&mut self) -> String { self.name.clone() }

    pub fn to_string(&mut self) -> String { self.name.clone() }
}

//
#[derive(Clone)]
pub struct Camera {
    x: f64,
    y: f64,
    zoom: f32,
}

//
impl Camera {
    pub fn get_x(&mut self) -> f64 { self.x.clone() }
    pub fn get_y(&mut self) -> f64 { self.y.clone() }
    pub fn get_zoom(&mut self) -> f32 { self.zoom.clone() }

    pub fn set_x(&mut self, value: f64) { self.x = value; }
    pub fn set_x_rhai_int(&mut self, value: rhai::INT) { self.x = value as f64; }
    pub fn set_x_rhai_float(&mut self, value: rhai::FLOAT) { self.x = value as f64; }
    pub fn set_y(&mut self, value: f64) { self.y = value; }
    pub fn set_y_rhai_int(&mut self, value: rhai::INT) { self.y = value as f64; }
    pub fn set_y_rhai_float(&mut self, value: rhai::FLOAT) { self.y = value as f64; }
    pub fn set_zoom(&mut self, value: f32) { self.zoom = value; }
    pub fn set_zoom_rhai_int(&mut self, value: rhai::INT) { self.zoom = value as f32; }
    pub fn set_zoom_rhai_float(&mut self, value: rhai::FLOAT) { self.zoom = value as f32; }

    pub fn to_string(&mut self) -> String { 
        format!("Camera:\n\tx - {x} y - {y} zoom - {zoom}", x = self.x, y = self.y, zoom = self.zoom)
    }
}

//
#[derive(Clone)]
pub struct Scene {
    width: u64,
    height: u64,
    in_color: String,
    out_color: String,
    layers: Vec<Layer>,
    camera: Camera,
}

//
impl Scene {
    pub fn get_width(&mut self) -> u64 { self.width.clone() }
    pub fn get_height(&mut self) -> u64 { self.height.clone() }
    pub fn get_inside_color(&mut self) -> String { self.in_color.clone() }
    pub fn get_outside_color(&mut self) -> String { self.out_color.clone() }
    pub fn get_layers(&mut self) -> Vec<Layer> { self.layers.clone() }
    // pub fn index_layer(&mut self, index: usize) -> Layer { self.layers[index].clone() }
    pub fn get_camera(&mut self) -> Camera { self.camera.clone() }

    pub fn set_width(&mut self, value: u64) { self.width = value; }
    pub fn set_width_rhai_int(&mut self, value: rhai::INT) { self.width = value as u64; }
    pub fn set_width_rhai_float(&mut self, value: rhai::FLOAT) { self.width = value as u64; }
    pub fn set_height(&mut self, value: u64) { self.height = value; }
    pub fn set_height_rhai_int(&mut self, value: rhai::INT) { self.height = value as u64; }
    pub fn set_height_rhai_float(&mut self, value: rhai::FLOAT) { self.height = value as u64; }
    pub fn set_inside_color(&mut self, value: String) { self.in_color = value; }
    pub fn set_outside_color(&mut self, value: String) { self.out_color = value; }
    pub fn set_camera(&mut self, value: Camera) { self.camera = value; }

    pub fn to_string(&mut self) -> String {
        let mut layers_str = String::new();
        for layer in &self.layers {
            layers_str.insert_str(layers_str.len(), &format!("\n\t{}", &layer.name))
        }
        format!("Scene:\n\twidth - {width} height - {height}\n\tinside color - {in_color} outside color - {out_color}\n\t{camera}\n\tLayers:{layers}", 
            width = self.width, height = self.height, in_color = self.in_color, out_color = self.out_color, 
            camera = &mut self.camera.to_string(), layers = layers_str)
    }
}

//
pub fn create_scene(engine: &Engine, config: &Map) -> Scene {
    //
    let mut layers_vec: Vec<Layer> = Vec::new();
    //
    for map in config["layers"].clone().into_typed_array::<Map>()
    .expect("Every scene's config should contain a 'layers' array, which should only have object-like members.") {
        //
        layers_vec.push( Layer { name: map["name"].clone().into_string()
        .expect("Every member in the 'layers' array of a scene's config should have a string 'name' attribute.") } );
    }
    //
    Scene {
        //
        width: { 
            if config["width"].is_int() {
                config["width"].as_int()
                .expect("Every scene's config should contain an integer 'width' attribute.") as u64
            }
            else if config["width"].is_float() {
                config["width"].as_float()
                .expect("Every scene's config should contain an integer 'width' attribute.") as u64
            } else {
                config["width"].clone_cast::<u64>()
            }
        },
        //
        height: { 
            if config["height"].is_int() { 
                config["height"].as_int()
                .expect("Every scene's config should contain an integer 'height' attribute.") as u64
            }
            else if config["height"].is_float() {
                config["height"].as_float()
                .expect("Every scene's config should contain an integer 'height' attribute.") as u64
            } else {
                config["height"].clone_cast::<u64>()
            }
        },
        //
        in_color: config["background-color"].clone().into_string()
        .expect("Every scene's config should contain a string 'background-color' attribute."),
        //
        out_color: config["outside-color"].clone().into_string()
        .expect("Every scene's config should contain a string 'outside-color' attribute."),
        //
        camera: Camera {
            //
            x: {
                if config["camera-position"].clone_cast::<Map>()["x"].is_float() {
                    config["camera-position"].clone_cast::<Map>()["x"].as_float()
                    .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes.") as f64
                } else if config["camera-position"].clone_cast::<Map>()["x"].is_int() {
                    config["camera-position"].clone_cast::<Map>()["x"].as_int()
                    .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes.") as f64
                } else {
                    config["camera-position"].clone_cast::<Map>()["x"].clone_cast::<f64>()
                }
            }, 
            //
            y: {
                if config["camera-position"].clone_cast::<Map>()["y"].is_float() {
                    config["camera-position"].clone_cast::<Map>()["y"].as_float()
                    .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes.") as f64
                } else if config["camera-position"].clone_cast::<Map>()["y"].is_int() {
                    config["camera-position"].clone_cast::<Map>()["y"].as_int()
                    .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes.") as f64
                } else {
                    config["camera-position"].clone_cast::<Map>()["y"].clone_cast::<f64>()
                }
            }, 
            //
            zoom: 1 as f32 
        },
        //
        layers: layers_vec,
    }
}