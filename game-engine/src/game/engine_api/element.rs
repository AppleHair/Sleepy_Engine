
use rhai::{Map, Dynamic};

use crate::{data::get_element_type, game::dynamic_to_number};

use super::asset::*;

/// Receives a string borrow with a\
/// hex color code (#RRGGBBAA / #RRGGBB),\
/// and converts it into a slice of bytes.
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

/// Used for storing point data\
/// in element properties.
#[derive(Clone)]
pub struct ElemPoint {
    pub x: f32,
    pub y: f32,
}

impl ElemPoint {
    pub fn get_x(&mut self) -> rhai::FLOAT { self.x.clone() as rhai::FLOAT }
    pub fn get_y(&mut self) -> rhai::FLOAT { self.y.clone() as rhai::FLOAT }

    pub fn set_x(&mut self, value: rhai::FLOAT) { self.x = value as f32; }
    pub fn set_y(&mut self, value: rhai::FLOAT) { self.y = value as f32; }
}

/// Used for storing RGBA\
/// color data in element\
/// properties.
#[derive(Clone)]
pub struct ElemColor {
    pub r: u8, pub g: u8,
    pub b: u8, pub a: u8,
}

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

/// This struct is used for
/// storing the information\
/// which needs to be provided
/// by the scene's config in\
/// order to create an object.
/// 
/// This Information might also
/// be provided by the global\
/// `add_object_to_stack` function,
/// but in that case, only the position\
/// is being provided by the function's
/// arguments, and the rest of\
/// the information uses default values.
pub struct ObjectInitInfo {
    pub idx_in_stack: u32,
    pub init_x: f32, pub init_y: f32,
    pub init_scale_x: f32, pub init_scale_y: f32,
    pub init_color: String, pub init_alpha: u8,
}

impl ObjectInitInfo {
    /// Using the index of the object
    /// in the object stack, and the\
    /// object instance's config map,
    /// this function extracts the\
    /// necessary information for
    /// creating an object.
    pub fn new(idx: u32, map: &Map) -> Self { Self {
        idx_in_stack: idx,
        init_x: dynamic_to_number(&map["x"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'x' attribute.")),
        init_y: dynamic_to_number(&map["y"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'y' attribute.")),
        init_scale_x: dynamic_to_number(&map["scale-x"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'scale-x' attribute.")),
        init_scale_y: dynamic_to_number(&map["scale-y"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an float 'scale-y' attribute.")),
        init_color: map["color"].clone().into_string()
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an string 'color' attribute.")),
        init_alpha: dynamic_to_number(&map["alpha"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an scene's config should contain an integer 'alpha' attribute.")) as u8,
    } }
}

/// This struct defines the
/// properties of an object,\
/// and the local API which
/// is used for accessing\
/// and modifying them. 
#[derive(Clone)]
pub struct Object {
    pub sprites: AssetList<Sprite>,
    pub position: ElemPoint,
    pub scale: ElemPoint,
    pub color: ElemColor,

    pub index_in_stack: u32,
}

impl Object {
    pub fn get_index_in_stack(&mut self) -> rhai::INT { self.index_in_stack.clone() as rhai::INT }
    pub fn get_position(&mut self) -> ElemPoint { self.position.clone() }
    pub fn get_scale(&mut self) -> ElemPoint { self.scale.clone() }
    pub fn get_color(&mut self) -> ElemColor { self.color.clone() }
    pub fn get_sprites(&mut self) -> AssetList<Sprite> { self.sprites.clone() }

    pub fn set_position(&mut self, value: ElemPoint) { self.position = value; }
    pub fn set_scale(&mut self, value: ElemPoint) { self.scale = value; }
    pub fn set_color(&mut self, value: ElemColor) { self.color = value; }
    /// `AssetList` setters need to
    /// check if the new list has\
    /// the same assets as the old
    /// one, and if it doesn't, then\
    /// the setting should be prevented.
    pub fn set_sprites(&mut self, value: AssetList<Sprite>) {
        if self.sprites.len == value.len &&
        self.sprites.members.iter().enumerate().all(
        |(idx, spr)| {value.members[idx].id == spr.id})
        {
            self.sprites = value;
        }
    }

    /// Using the object's config, and
    /// the provided object init info,\ 
    /// this function defines properties
    /// for the object in a new `Object`\
    /// API instance.
    pub fn new(config: &Map, info: ObjectInitInfo) -> Self {
        // Create a vector of `Sprite` instances
        let mut sprites_vec: Vec<Sprite> = Vec::new();
        // Add every sprite whos id is
        // included in the `sprites` list
        // of the object's config
        for id in config["sprites"].clone().into_typed_array::<rhai::INT>()
        .expect("Every object's config should contain a 'sprites' array, which should only have integer members.") {
            sprites_vec.push(Sprite::new(id as u32));
        }
        // Convert the hex color string
        // into a slice of bytes
        let color = hex_color_to_rgba(&info.init_color);
        // Return the new `Object` API instance,
        // while setting its properties using
        // the provided object init info
        Self {
            // Create a new `AssetList` instance
            // using the vector of `Sprite` instances
            // which was created earlier
            sprites: AssetList::new(sprites_vec),
            index_in_stack: info.idx_in_stack,
            position: ElemPoint { x: info.init_x, y: info.init_y },
            scale: ElemPoint { x: info.init_scale_x, y: info.init_scale_y },
            // Use the color slice of bytes
            // to create a new `ElemColor` instance
            // for the object's color property
            color: ElemColor { r: color[0], g: color[1],
                b: color[2], a: info.init_alpha }
        }
    }

    /// Using the object's config, and
    /// the provided object init info,\
    /// this function recycles an existing
    /// `Object` API instance to define\
    /// properties for a new object.
    pub fn recycle(&mut self, config: &Map, info: ObjectInitInfo) {
        // Recycle the `AssetList` instance
        // using the object config's `sprites` list
        self.sprites.recycle(&config["sprites"].read_lock::<Vec<Dynamic>>()
        .expect("Every object's config should contain a 'sprites' array, which should only have integer members."));
        // Convert the hex color string
        // into a slice of bytes
        let color = hex_color_to_rgba(&info.init_color);
        // Set the object's properties
        // using the provided object init info
        self.position.x = info.init_x;
        self.position.y = info.init_y;
        self.scale.x = info.init_scale_x;
        self.scale.y = info.init_scale_y;
        // Use the color slice of bytes
        // to set the object's color property
        self.color.r = color[0];
        self.color.g = color[1];
        self.color.b = color[2];
        self.color.a = info.init_alpha;
    }
}

/// This struct used for storing
/// information about a scene's layer.
///  
/// The scene's layers will be iterated 
/// through by the renderer, an it will\
/// draw the objects according to the order 
/// of the layers they are placed in.
#[derive(Clone)]
pub struct Layer {
    pub name: String,
    pub instances: Vec<u32>,
}

impl Layer {
    pub fn get_name(&mut self) -> String { self.name.clone() }
    pub fn get_instances(&mut self) -> Dynamic { self.instances.clone().into() }
}

/// This struct used for storing
/// information about the scene's camera.
/// 
/// The camera's properties are similar
/// to the properties of an object, but\
/// the camera's properties applys to every
/// object in the scene. For example,\
/// when the camera's position is changed,
/// every object in the scene will appear\
/// to move in the opposite direction
/// and when the camera's zoom is changed,\
/// every object in the scene will appear to scale.
#[derive(Clone)]
pub struct Camera {
    pub position: ElemPoint,
    pub zoom: f32,
    pub color: ElemColor,
}

impl Camera {
    pub fn get_position(&mut self) -> ElemPoint { self.position.clone() }
    pub fn get_zoom(&mut self) -> rhai::FLOAT { self.zoom.clone() as rhai::FLOAT }
    pub fn get_color(&mut self) -> ElemColor { self.color.clone() }

    pub fn set_position(&mut self, value: ElemPoint) { self.position = value; }
    pub fn set_zoom(&mut self, value: rhai::FLOAT) { self.zoom = value as f32; }
    pub fn set_color(&mut self, value: ElemColor) { self.color = value; }
}

/// This struct defines the
/// properties of a scene,\
/// and the local API which
/// is used for accessing\
/// and modifying them.
#[derive(Clone)]
pub struct Scene {
    pub camera: Camera,

    pub layers: Vec<Layer>,
    pub runtime_vacants: Vec<u32>,

    pub objects_len: usize,
    pub runtimes_len: usize,
    pub layers_len: usize,
}

impl Scene {
    pub fn get_objects_len(&mut self) -> rhai::INT { self.objects_len.clone() as rhai::INT }
    pub fn get_runtimes_len(&mut self) -> rhai::INT { self.runtimes_len.clone() as rhai::INT }
    pub fn get_runtime_vacants(&mut self)  -> Dynamic { self.runtime_vacants.clone().into() }
    pub fn get_camera(&mut self) -> Camera { self.camera.clone() }
    pub fn get_layers(&mut self) -> Dynamic { self.layers[0..self.layers_len].to_vec().into() }

    pub fn set_camera(&mut self, value: Camera) { self.camera = value; }

    /// This function removes an
    /// object instance from one of\
    /// the scene's layers if it already
    /// exists in any of them. It also\
    /// returns a boolean value which
    /// indicates if the instance was removed.
    pub fn remove_instance(&mut self, idx: rhai::INT) -> bool {
        // Find the layer in which
        // the instance is placed
        if let Some((layer_idx, &_)) = self.layers[0..self.layers_len].iter()
        .enumerate().find(|&(_, layer)| { layer.instances.contains(&(idx as u32)) }) {
            // Find the index in which
            // the instance is placed in the layer
            if let Some((index_to_remove, &_)) = self.layers[layer_idx].instances.iter()
            .enumerate().find(|&(_, &instance_index)| { instance_index as rhai::INT == idx }) {
                // Remove the instance
                let _ = self.layers[layer_idx].instances.swap_remove(index_to_remove);
                // Check if the instance 
                // was a runtime object
                if (idx as usize) >= self.objects_len && (idx as usize) < self.objects_len+self.runtimes_len {
                    // Add the instance's index
                    // to the "vacant runtime objects list"
                    self.runtime_vacants.push(idx as u32);
                }
                return true;
            }
        }
        false
    }

    /// This function adds an object
    /// instance to one of the scene's
    /// layers if it doesn't already exist\
    /// in any of them. It also returns a
    /// boolean value which indicates if
    /// the instance was added.
    pub fn add_instance(&mut self, idx: rhai::INT, layer_idx: rhai::INT) -> bool {
        // Check if the layer index received is
        // in bounds, if the instance index
        // received isn't already in a layer,
        // and if the instance index is in bounds.
        if layer_idx < (self.layers_len as rhai::INT) && layer_idx >= 0 &&
        !self.layers.iter().any(|layer| { layer.instances.contains(&(idx as u32)) }) &&
        (idx as usize) < self.objects_len+self.runtimes_len {
            // Add the instance to the layer
            self.layers[layer_idx as usize].instances.push(idx as u32);
            // Check if the instance index was
            // in the "vacant runtime objects list"
            if let Some((index,&_)) = self.runtime_vacants.iter().enumerate()
            .find(|(_, &instance_index)| { instance_index as rhai::INT == idx }) {
                // Remove the instance's index
                // from the "vacant runtime objects list"
                let _ = self.runtime_vacants.swap_remove(index);
            }
            return true;
        }
        false
    }

    /// Using the scene's config, this\
    /// function defines properties for\
    /// the scene in a new `Scene` API\
    /// instance.
    pub fn new(config: &Map) -> Self {
        // Create a vector of `Layer` instances
        let mut layers_vec: Vec<Layer> = Vec::new();
        // Add every layer whos name is
        // included in the `layers` list
        // of the scene's config
        for name in config["layers"].clone().into_typed_array::<String>()
        .expect("Every scene's config should contain a 'layers' array, which should only have strings.") {
            layers_vec.push( Layer { name, instances: { Vec::new() } } );
        }
        // Get the camera's properties
        let camera_info: &Map = &config["camera"].read_lock::<Map>()
        .expect("Every scene's config should contain a 'camera' object-like attrbute.");
        // Convert the hex color string
        // into a slice of bytes
        let color = hex_color_to_rgba(&camera_info["color"].read_lock::<rhai::ImmutableString>()
        .expect("Every 'camera' object in a scene's config should contain a 'color' string attribute."));
        // Return the new `Scene` API instance,
        // while setting its properties using
        // the provided configuration
        Self {
            objects_len: config["object-instances"].read_lock::<Vec<Dynamic>>()
            .expect("Every scene's config should contain an array 'object-instances' attribute.").len(),
            layers_len: layers_vec.len(),
            runtimes_len: 0,
            runtime_vacants: Vec::new(),
            // Create a new camera instance
            // for the scene's `camera` property
            camera: Camera {
                position: ElemPoint { 
                    x: dynamic_to_number(&camera_info["x"])
                    .expect("Every 'camera' object in a scene's config should contain a 'x' float attribute."), 
                    y: dynamic_to_number(&camera_info["y"])
                    .expect("Every 'camera' object in a scene's config should contain a 'y' float attribute.") 
                },
                zoom: dynamic_to_number(&camera_info["zoom"])
                .expect("Every 'camera' object in a scene's config should contain a 'zoom' float attribute."),
                color: ElemColor {
                    // Use the color slice of bytes
                    // to set the camera's color property
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: dynamic_to_number(&camera_info["alpha"])
                    .expect(concat!("Every 'camera' object in a scene's config should",
                    " contain a 'alpha' integer attribute.")) as u8
                }
            },
            // Use the vector of `Layer` instances
            // as the scene's `layers` property
            layers: layers_vec,
        }
    }

    /// Using the scene's config, this
    /// function recycles an existing\
    /// `Scene` API instance to define
    /// properties for a new object.
    pub fn recycle(&mut self, config: &Map) {
        // Use this counter to keep track
        // of th number of layers this scene
        // should have after recycling
        let mut i = 0_usize;
        // Iterate through the scene's config's
        // `layers` list, and add every layer
        // whos name is included in the list
        for name in &config["layers"].read_lock::<Vec<Dynamic>>()
        .expect("Every scene's config should contain a 'layers' array, which should only have strings.")
        as &Vec<Dynamic>  {
            let name: &str = &name.read_lock::<rhai::ImmutableString>()
            .expect("Every scene's config should contain a 'layers' array, which should only have strings.");
            // If this layer name 's index
            // is still in the bounds of
            // the scene's `layers` property,
            // then recycle the layer in its
            // index, replacing it with a new
            // clear layer with the new name.
            if i < self.layers.len() {
                self.layers[i].name.clear();
                self.layers[i].name.push_str(name);
                self.layers[i].instances.clear();
                i += 1;
                continue;
            }
            // Otherwise, extend the scene's
            // `layers` property with a new
            // clear layer with the new name.
            self.layers.push( Layer {  name: String::from(name), instances: Vec::new() } );
            i += 1;
        }
        // Get the camera's properties
        let camera_info: &Map = &config["camera"].read_lock::<Map>()
        .expect("Every scene's config should contain a 'camera' object-like attrbute.");
        // Convert the hex color string
        // into a slice of bytes
        let color = hex_color_to_rgba(&camera_info["color"].read_lock::<rhai::ImmutableString>()
        .expect("Every 'camera' object in a scene's config should contain a 'color' string attribute."));
        // Set the scene's properties
        // using the provided configuration
        self.layers_len = i;
        self.runtimes_len = 0;
        self.runtime_vacants.clear();
        self.objects_len = config["object-instances"].read_lock::<Vec<Dynamic>>()
        .expect("Every scene's config should contain an array 'object-instances' attribute.").len();
        // Create a new camera instance
        // for the scene's `camera` property
        self.camera = Camera {
            position: ElemPoint { 
                x: dynamic_to_number(&camera_info["x"])
                .expect("Every 'camera' object in a scene's config should contain a 'x' float attribute."), 
                y: dynamic_to_number(&camera_info["y"])
                .expect("Every 'camera' object in a scene's config should contain a 'y' float attribute.") 
            },
            zoom: dynamic_to_number(&camera_info["zoom"])
            .expect("Every 'camera' object in a scene's config should contain a 'zoom' float attribute."),
            color: ElemColor {
                // Use the color slice of bytes
                // to set the camera's color property
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

/// This struct defines the
/// properties of the state\
/// manager, and the local
/// API which is used for\
/// accessing and modifying them.
#[derive(Clone)]
pub struct Game {
    pub cur_scene: u32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub version: Vec<u8>,
    pub clear_red: u8,
    pub clear_green: u8,
    pub clear_blue: u8,
    pub fps: u16,
}

impl Game {
    pub fn get_cur_scene(&mut self) -> rhai::INT { self.cur_scene as rhai::INT }
    pub fn get_canvas_width(&mut self) -> rhai::FLOAT { self.canvas_width as rhai::FLOAT }
    pub fn get_canvas_height(&mut self) -> rhai::FLOAT { self.canvas_height as rhai::FLOAT }
    pub fn get_version(&mut self) -> Dynamic { self.version.clone().into() }
    pub fn get_clear_red(&mut self) -> rhai::INT { self.clear_red as rhai::INT }
    pub fn get_clear_green(&mut self) -> rhai::INT { self.clear_green as rhai::INT }
    pub fn get_clear_blue(&mut self) -> rhai::INT { self.clear_blue as rhai::INT }
    pub fn get_fps(&mut self) -> rhai::INT { self.fps as rhai::INT }

    // The `cur_scene` property
    // setter needs to check if
    // the requested scene rowid
    // is a valid scene rowid.
    pub fn set_cur_scene(&mut self, value: rhai::INT) -> Result<(), Box<rhai::EvalAltResult>> { 
        let kind = get_element_type(value as u32);
        if kind == 2 {
            self.cur_scene = value as u32;
            Ok(())
        } else {
            Err("Tried to switch to a scene that doesn't exist.".into())
        }
    }
    pub fn set_canvas_width(&mut self, value: rhai::FLOAT) { self.canvas_width = value as f32; }
    pub fn set_canvas_height(&mut self, value: rhai::FLOAT) { self.canvas_height = value as f32; }
    pub fn set_clear_red(&mut self, value: rhai::INT) { self.clear_red = value as u8; }
    pub fn set_clear_green(&mut self, value: rhai::INT) { self.clear_green = value as u8; }
    pub fn set_clear_blue(&mut self, value: rhai::INT) { self.clear_blue = value as u8; }
    pub fn set_fps(&mut self, value: rhai::INT) { self.fps = value as u16; }

    /// Using the state manager's\
    /// config, this function defines\
    /// properties for the state manager\
    /// in a new `Game` API instance.
    pub fn new(config: &Map) -> Self {
        // Convert the hex color string
        // into a slice of bytes
        let color = hex_color_to_rgba(&config["clear-color"].read_lock::<rhai::ImmutableString>()
        .expect("The state manager's config should contain a 'clear-color' string attribute."));
        // Get the version numbers as
        // an vector of `Dynamic`s.
        let version_vec: &Vec<Dynamic> = &config["version"].read_lock::<Vec<Dynamic>>()
        .expect("The state manager's config should contain a 'version' integer array attribute.");
        // Return the new `Game` API instance,
        // while setting its properties using
        // the provided configuration
        Self {
            cur_scene: dynamic_to_number(&config["initial-scene"])
            .expect("The state manager's config should contain a 'initial-scene' integer attribute.") as u32,
            canvas_width: dynamic_to_number(&config["canvas-width"])
            .expect("The state manager's config should contain a 'canvas-width' float attribute.") as f32,
            canvas_height: dynamic_to_number(&config["canvas-height"])
            .expect("The state manager's config should contain a 'canvas-height' float attribute.") as f32,
            fps: dynamic_to_number(&config["fps"])
            .expect("The state manager's config should contain a 'fps' integer attribute.") as u16,
            // Use the version numbers vector
            // to set the state manager's `version``
            // property
            version: vec![
                dynamic_to_number(&version_vec[0])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8,
                dynamic_to_number(&version_vec[1])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8,
                dynamic_to_number(&version_vec[2])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8,
                dynamic_to_number(&version_vec[3])
                .expect("The state manager's config should contain a 'version' integer array attribute.") as u8
            ],
            // Use the color slice of bytes
            // to set the clear color properties
            clear_red: color[0],
            clear_green: color[1],
            clear_blue: color[2],
        }
    }
}