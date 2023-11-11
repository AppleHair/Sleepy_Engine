
use rhai::{Map, Dynamic};

use crate::{data::get_element_type, game::dynamic_to_number};

use super::asset::*;

// Receives a string borrow with a
// hex color code (#RRGGBBAA / #RRGGBB),
// and converts it into a slice of bytes.
pub fn hex_color_to_rgba(hex: &str) -> [u8; 4] {
    // Result slice with
    // place-holder values
    let mut result: [u8; 4] = [0, 0, 0, 255];
    // Result slice index counter
    let mut i = 0;
    // Hex color string
    // index counter
    let mut si = 1;

    while si < hex.len() {
        // Convert the hex string
        // into an unsigned byte.
        result[i] = u8::from_str_radix(&hex[si..si+2], 16)
        .expect("hex to u8 parse should succeed");
        // Increase hex color string
        // index counter by 2.
        si += 2;
        // Increase result slice
        // index counter by 1.
        i += 1;
    }
    // Return the 
    // result slice
    result
}

//
#[derive(Clone)]
pub struct ElemPoint {
    pub x: f32,
    pub y: f32,
}

//
impl ElemPoint {
    pub fn get_x(&mut self) -> rhai::FLOAT { self.x.clone() as rhai::FLOAT }
    pub fn get_y(&mut self) -> rhai::FLOAT { self.y.clone() as rhai::FLOAT }

    pub fn set_x(&mut self, value: rhai::FLOAT) { self.x = value as f32; }
    pub fn set_y(&mut self, value: rhai::FLOAT) { self.y = value as f32; }
}

//
#[derive(Clone)]
pub struct ElemColor {
    pub r: u8, pub g: u8,
    pub b: u8, pub a: u8,
}

//
impl ElemColor {
    pub fn get_r(&mut self) -> rhai::INT { self.r.clone() as rhai::INT }
    pub fn get_g(&mut self) -> rhai::INT { self.g.clone() as rhai::INT }
    pub fn get_b(&mut self) -> rhai::INT { self.b.clone() as rhai::INT }
    pub fn get_a(&mut self) -> rhai::INT { self.a.clone() as rhai::INT }

    pub fn set_r(&mut self, value: rhai::INT) { self.r = value as u8; }
    pub fn set_g(&mut self, value: rhai::INT) { self.g = value as u8; }
    pub fn set_b(&mut self, value: rhai::INT) { self.b = value as u8; }
    pub fn set_a(&mut self, value: rhai::INT) { self.a = value as u8; }
}

//
pub struct ObjectInitInfo {
    pub idx_in_stack: u32,
    pub init_x: f32, pub init_y: f32,
    pub init_scale_x: f32, pub init_scale_y: f32,
    pub init_color: String, pub init_alpha: u8,
}

impl ObjectInitInfo {
    pub fn new(idx: u32, map: &Map) -> Self { Self {
        idx_in_stack: idx,
        //
        init_x: dynamic_to_number(&map["x"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'x' attribute.")),
        //
        init_y: dynamic_to_number(&map["y"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'y' attribute.")),
        //
        init_scale_x: dynamic_to_number(&map["scale-x"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'scale-x' attribute.")),
        //
        init_scale_y: dynamic_to_number(&map["scale-y"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'scale-y' attribute.")),
        //
        init_color: map["color"].clone().into_string()
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an string 'color' attribute.")),
        //
        init_alpha: dynamic_to_number(&map["alpha"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an integer 'alpha' attribute.")) as u8,
    } }
}

//
#[derive(Clone)]
pub struct Object {
    pub sprites: AssetList<Sprite>,
    pub index_in_stack: u32,
    pub position: ElemPoint,
    pub scale: ElemPoint,
    pub color: ElemColor,
}

//
impl Object {
    pub fn get_index_in_stack(&mut self) -> rhai::INT { self.index_in_stack.clone() as rhai::INT }
    pub fn get_position(&mut self) -> ElemPoint { self.position.clone() }
    pub fn get_scale(&mut self) -> ElemPoint { self.scale.clone() }
    pub fn get_color(&mut self) -> ElemColor { self.color.clone() }
    pub fn get_sprites(&mut self) -> AssetList<Sprite> { self.sprites.clone() }

    pub fn set_position(&mut self, value: ElemPoint) { self.position = value; }
    pub fn set_scale(&mut self, value: ElemPoint) { self.scale = value; }
    pub fn set_color(&mut self, value: ElemColor) { self.color = value; }
    pub fn set_sprites(&mut self, value: AssetList<Sprite>) {
        if self.sprites.len == value.len &&
        self.sprites.members.iter().enumerate().all(
        |(idx, spr)| {value.members[idx].id == spr.id})
        {
            self.sprites = value;
        }
    }

    //
    pub fn new(config: &Map, info: ObjectInitInfo) -> Self {
        //
        let mut sprites_vec: Vec<Sprite> = Vec::new();
        //
        for id in config["sprites"].clone().into_typed_array::<rhai::INT>()
        .expect("Every object's config should contain a 'sprites' array, which should only have integer members.") {
            //
            sprites_vec.push(Sprite::new(id as u32));
        }
        //
        let color = hex_color_to_rgba(&info.init_color);
        //
        Self {
            //
            sprites: AssetList::new(sprites_vec),
            //
            index_in_stack: info.idx_in_stack,
            //
            position: ElemPoint { x: info.init_x, y: info.init_y },
            //
            scale: ElemPoint { x: info.init_scale_x, y: info.init_scale_y },
            //
            color: ElemColor { r: color[0], g: color[1],
                b: color[2], a: info.init_alpha }
        }
    }
    //
    pub fn recycle(&mut self, config: &Map, info: ObjectInitInfo) {
        //
        self.sprites.recycle(config["sprites"].clone().into_typed_array::<rhai::INT>()
        .expect("Every object's config should contain a 'sprites' array, which should only have integer members."));
        //
        let color = hex_color_to_rgba(&info.init_color);
        //
        self.position.x = info.init_x;
        self.position.y = info.init_y;
        //
        self.scale.x = info.init_scale_x;
        self.scale.y = info.init_scale_y;
        //
        self.color.r = color[0];
        self.color.g = color[1];
        self.color.b = color[2];
        self.color.a = info.init_alpha;
    }
}

//
#[derive(Clone)]
pub struct Layer {
    pub name: String,
    pub instances: Vec<u32>,
}

//
impl Layer {
    pub fn get_name(&mut self) -> String { self.name.clone() }
    pub fn get_instances(&mut self) -> Dynamic { self.instances.clone().into() }
}

//
#[derive(Clone)]
pub struct Camera {
    pub position: ElemPoint,
    pub zoom: f32,
    pub color: ElemColor,
}

//
impl Camera {
    pub fn get_position(&mut self) -> ElemPoint { self.position.clone() }
    pub fn get_zoom(&mut self) -> rhai::FLOAT { self.zoom.clone() as rhai::FLOAT }
    pub fn get_color(&mut self) -> ElemColor { self.color.clone() }

    pub fn set_position(&mut self, value: ElemPoint) { self.position = value; }
    pub fn set_zoom(&mut self, value: rhai::FLOAT) { self.zoom = value as f32; }
    pub fn set_color(&mut self, value: ElemColor) { self.color = value; }
}

//
#[derive(Clone)]
pub struct Scene {
    pub objects_len: usize,
    pub runtimes_len: usize,
    pub runtime_vacants: Vec<u32>,
    pub layers_len: usize,
    pub layers: Vec<Layer>,
    pub camera: Camera,
}

//
impl Scene {
    pub fn get_layers(&mut self) -> Dynamic { self.layers[0..self.layers_len].to_vec().into() }
    pub fn get_objects_len(&mut self) -> rhai::INT { self.objects_len.clone() as rhai::INT }
    pub fn get_runtimes_len(&mut self) -> rhai::INT { self.runtimes_len.clone() as rhai::INT }
    pub fn get_runtime_vacants(&mut self)  -> Dynamic { self.runtime_vacants.clone().into() }
    pub fn get_camera(&mut self) -> Camera { self.camera.clone() }

    pub fn set_camera(&mut self, value: Camera) { self.camera = value; }

    pub fn remove_instance(&mut self, idx: rhai::INT) -> bool {
        // Find the layer in which
        // the instance is placed
        if let Some((layer_idx, &_)) = self.layers[0..self.layers_len].iter()
        .enumerate().find(|&(_, layer)| { layer.instances.contains(&(idx as u32)) }) {
            // Find the index in which
            // the instance is placed in the layer
            if let Some((index_to_remove, &_)) = self.layers[layer_idx].instances.iter()
            .enumerate().find(|&(_, &instance_index)| { instance_index as rhai::INT == idx }) {
                //
                let _ = self.layers[layer_idx].instances.swap_remove(index_to_remove);
                // Check if the instance 
                // was a runtime object
                if (idx as usize) >= self.objects_len && (idx as usize) < self.objects_len+self.runtimes_len {
                    //
                    self.runtime_vacants.push(idx as u32);
                }
                //
                return true;
            }
        }
        //
        false
    }

    pub fn add_instance(&mut self, idx: rhai::INT, layer_idx: rhai::INT) -> bool {
        // Check if the layer index received is
        // in bounds, if the instance index
        // received isn't already in a layer,
        // and if the instance index is in bounds.
        if layer_idx < (self.layers_len as rhai::INT) && layer_idx >= 0 &&
        !self.layers.iter().any(|layer| { layer.instances.contains(&(idx as u32)) }) &&
        (idx as usize) < self.objects_len+self.runtimes_len {
            //
            self.layers[layer_idx as usize].instances.push(idx as u32);
            // Check if the instance index was
            // in the "vacant runtime objects list"
            if let Some((index,&_)) = self.runtime_vacants.iter().enumerate()
            .find(|(_, &instance_index)| { instance_index as rhai::INT == idx }) {
                //
                let _ = self.runtime_vacants.swap_remove(index);
            }
            //
            return true;
        }
        //
        false
    }

    //
    pub fn new(config: &Map) -> Self {
        //
        let mut layers_vec: Vec<Layer> = Vec::new();
        //
        for name in config["layers"].clone().into_typed_array::<String>()
        .expect("Every scene's config should contain a 'layers' array, which should only have strings.") {
            //
            layers_vec.push( Layer { name, instances: { Vec::new() } } );
        }
        //
        let camera_info = config["camera"].clone_cast::<Map>();
        //
        let color = hex_color_to_rgba(&camera_info["color"].clone().into_string()
        .expect("Every 'camera' object in a scene's config should contain a 'color' string attribute."));
        //
        Self {
            //
            objects_len: config["object-instances"].clone().into_array()
            .expect("Every scene's config should contain an array 'object-instances' attribute.").len(),
            //
            layers_len: layers_vec.len(),
            //
            runtimes_len: 0,
            //
            runtime_vacants: Vec::new(),
            //
            camera: Camera {
                //
                position: ElemPoint { 
                    x: dynamic_to_number(&camera_info["x"])
                    .expect("Every 'camera' object in a scene's config should contain a 'x' float attribute."), 
                    y: dynamic_to_number(&camera_info["y"])
                    .expect("Every 'camera' object in a scene's config should contain a 'y' float attribute.") 
                }, 
                //
                zoom: dynamic_to_number(&camera_info["zoom"])
                .expect("Every 'camera' object in a scene's config should contain a 'zoom' float attribute."),
                //
                color: ElemColor {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: dynamic_to_number(&camera_info["alpha"])
                    .expect(concat!("Every 'camera' object in a scene's config should",
                    " contain a 'alpha' integer attribute.")) as u8
                }
            },
            //
            layers: layers_vec,
        }
    }

    //
    pub fn recycle(&mut self, config: &Map) {
        //
        let mut i = 0_usize;
        //
        for name in config["layers"].clone().into_typed_array::<String>()
        .expect("Every scene's config should contain a 'layers' array, which should only have strings.") {
            //
            if i < self.layers.len() {
                //
                self.layers[i].name = name;
                
                //
                self.layers[i].instances.clear();
                //
                i += 1;
                continue;
            }
            //
            self.layers.push( Layer { name, instances: Vec::new() } );
            //
            i += 1;
        }
        //
        self.layers_len = i;
        //
        self.runtimes_len = 0;
        //
        self.runtime_vacants.clear();
        //
        self.objects_len = config["object-instances"].clone().into_array()
        .expect("Every scene's config should contain an array 'object-instances' attribute.").len();
        //
        let camera_info = config["camera"].clone_cast::<Map>();
        //
        let color = hex_color_to_rgba(&camera_info["color"].clone().into_string()
        .expect("Every 'camera' object in a scene's config should contain a 'color' string attribute."));
        //
        self.camera = Camera {
            //
            position: ElemPoint { 
                x: dynamic_to_number(&camera_info["x"])
                .expect("Every 'camera' object in a scene's config should contain a 'x' float attribute."), 
                y: dynamic_to_number(&camera_info["y"])
                .expect("Every 'camera' object in a scene's config should contain a 'y' float attribute.") 
            }, 
            //
            zoom: dynamic_to_number(&camera_info["zoom"])
            .expect("Every 'camera' object in a scene's config should contain a 'zoom' float attribute."),
            //
            color: ElemColor {
                r: color[0],
                g: color[1],
                b: color[2],
                a: dynamic_to_number(&camera_info["alpha"])
                .expect(concat!("Every 'camera' object in a scene's config should",
                " contain a 'alpha' integer attribute.")) as u8
            }
        };
    }
}

//
#[derive(Clone)]
pub struct Game {
    pub cur_scene: u32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub version: [u8; 4],
    pub clear_red: u8,
    pub clear_green: u8,
    pub clear_blue: u8,
    pub fps: u16,
}

impl Game {
    //
    pub fn get_cur_scene(&mut self) -> rhai::INT { self.cur_scene as rhai::INT }
    pub fn get_canvas_width(&mut self) -> rhai::FLOAT { self.canvas_width as rhai::FLOAT }
    pub fn get_canvas_height(&mut self) -> rhai::FLOAT { self.canvas_height as rhai::FLOAT }
    pub fn get_version(&mut self) -> Dynamic { self.version.iter().map(|&num|
        { Dynamic::from_int(num as rhai::INT) }).collect::<Vec<Dynamic>>().into() }
    pub fn get_clear_red(&mut self) -> rhai::INT { self.clear_red as rhai::INT }
    pub fn get_clear_green(&mut self) -> rhai::INT { self.clear_green as rhai::INT }
    pub fn get_clear_blue(&mut self) -> rhai::INT { self.clear_blue as rhai::INT }
    pub fn get_fps(&mut self) -> rhai::INT { self.fps as rhai::INT }

    //
    pub fn set_cur_scene(&mut self, value: rhai::INT) -> Result<(), Box<rhai::EvalAltResult>> { 
        //
        let kind = get_element_type(value as u32);
        //
        if kind == 2 {
            //
            self.cur_scene = value as u32;
            //
            Ok(())
        } else {
            //
            Err("Tried to switch to a scene that doesn't exist.".into())
        }
    }
    //
    pub fn set_canvas_width(&mut self, value: rhai::FLOAT) { self.canvas_width = value as f32; }
    pub fn set_canvas_height(&mut self, value: rhai::FLOAT) { self.canvas_height = value as f32; }
    pub fn set_clear_red(&mut self, value: rhai::INT) { self.clear_red = value as u8; }
    pub fn set_clear_green(&mut self, value: rhai::INT) { self.clear_green = value as u8; }
    pub fn set_clear_blue(&mut self, value: rhai::INT) { self.clear_blue = value as u8; }
    pub fn set_fps(&mut self, value: rhai::INT) { self.fps = value as u16; }
    //
    pub fn new(config: &Map) -> Self {
        //
        let color = hex_color_to_rgba(&config["clear-color"].clone().into_string()
        .expect("The state manager's config should contain a 'clear-color' string attribute."));
        //
        let version_vec = config["version"].clone().into_array()
        .expect("The state manager's config should contain a 'version' integer array attribute.");
        //
        Self {
            cur_scene: dynamic_to_number(&config["initial-scene"])
            .expect("The state manager's config should contain a 'initial-scene' integer attribute.") as u32,
            canvas_width: dynamic_to_number(&config["canvas-width"])
            .expect("The state manager's config should contain a 'canvas-width' float attribute.") as f32,
            canvas_height: dynamic_to_number(&config["canvas-height"])
            .expect("The state manager's config should contain a 'canvas-height' float attribute.") as f32,
            fps: dynamic_to_number(&config["fps"])
            .expect("The state manager's config should contain a 'fps' integer attribute.") as u16,
            version: [
                dynamic_to_number(&version_vec[0])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8,
                dynamic_to_number(&version_vec[1])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8,
                dynamic_to_number(&version_vec[2])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8,
                dynamic_to_number(&version_vec[3])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8
            ],
            clear_red: color[0],
            clear_green: color[1],
            clear_blue: color[2],
        }
    }
}