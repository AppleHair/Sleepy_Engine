use rhai::{Engine, Map};

#[derive(Clone)]
pub struct Layer {
    name: String,
}

impl Layer {
    pub fn get_name(&mut self) -> String { self.name.clone() }

    pub fn to_string(layer: &mut Self) -> String { layer.get_name() }
}

#[derive(Clone)]
pub struct Camera {
    x: f64,
    y: f64,
    zoom: f32,
}

impl Camera {
    pub fn get_x(&mut self) -> f64 { self.x.clone() }
    pub fn get_y(&mut self) -> f64 { self.y.clone() }
    pub fn get_zoom(&mut self) -> f32 { self.zoom.clone() }

    pub fn set_x(&mut self, value: f64) { self.x = value; }
    pub fn set_y(&mut self, value: f64) { self.y = value; }
    pub fn set_zoom(&mut self, value: f32) { self.zoom = value; }

    pub fn to_string(camera: &mut Self) -> String { 
        format!("Camera:\n\tx - {x} y- {y} zoom - {zoom}", x = camera.x, y = camera.y, zoom = camera.zoom)
    }
}

#[derive(Clone)]
pub struct Scene {
    width: u64,
    height: u64,
    in_color: String,
    out_color: String,
    layers: Vec<Layer>,
    camera: Camera,
}

impl Scene {
    pub fn get_width(&mut self) -> u64 { self.width.clone() }
    pub fn get_height(&mut self) -> u64 { self.height.clone() }
    pub fn get_inside_color(&mut self) -> String { self.in_color.clone() }
    pub fn get_outside_color(&mut self) -> String { self.out_color.clone() }
    pub fn get_layers(&mut self) -> Vec<Layer> { self.layers.clone() }
    pub fn get_layer(&mut self, index: usize) -> Layer { self.layers[index].clone() }
    pub fn get_camera(&mut self) -> Camera { self.camera.clone() }

    pub fn set_width(&mut self, value: u64) { self.width = value; }
    pub fn set_height(&mut self, value: u64) { self.height = value; }
    pub fn set_inside_color(&mut self, value: String) { self.in_color = value; }
    pub fn set_outside_color(&mut self, value: String) { self.out_color = value; }

    pub fn to_string(scene: &mut Self) -> String {
        // let layers_str = String::new();
        format!("Scene:\n\twidth - {width} height - {height}\n\tinside color - {in_color} outside color -{out_color}\n\t{camera}\n", 
            width = scene.width, height = scene.height, in_color = scene.in_color, out_color = scene.out_color, 
            camera = Camera::to_string(&mut scene.camera))
    }
}

pub fn create_scene(engine: &Engine, config: &Map) -> Scene {
    let mut layers_vec: Vec<Layer> = Vec::new();
    Scene {
        width: config["width"].as_int().expect("The value of 'width' in a scene's config should be an integer") as u64,
        height: config["height"].as_int().expect("The value of 'height' in a scene's config should be an integer") as u64,
        in_color: config["background-color"].clone().as_string().expect("The value of 'background-color' in a scene's config should be an string"),
        out_color: config["outside-color"].clone().as_string().expect("The value of 'outside-color' in a scene's config should be an string"),
        camera: Camera { 
                x: config["camera-position.x"].as_int().expect("The x value of 'camera-position' in a scene's config should be an integer") as f64, 
                y: config["camera-position.y"].as_int().expect("The y value of 'camera-position' in a scene's config should be an integer") as f64, 
                zoom: 1 as f32 
            },
        layers: layers_vec,
    }
}