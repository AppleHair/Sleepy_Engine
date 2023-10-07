
use rhai::{Map, Dynamic};

use super::{dynamic_to_number, asset::*};

//
#[derive(Clone)]
pub struct PositionPoint {
    pub x: f32,
    pub y: f32,
}

//
impl PositionPoint {
    pub fn get_x(&mut self) -> rhai::FLOAT { self.x.clone() as rhai::FLOAT }
    pub fn get_y(&mut self) -> rhai::FLOAT { self.y.clone() as rhai::FLOAT }

    pub fn set_x(&mut self, value: rhai::FLOAT) { self.x = value as f32; }
    pub fn set_y(&mut self, value: rhai::FLOAT) { self.y = value as f32; }

    pub fn to_string(&mut self) -> String { 
        format!("x - {x} y - {y}", x = self.x, y = self.y)
    }
}

//
#[derive(Clone)]
pub struct CollisionBox {
    pub point1: PositionPoint,
    pub point2: PositionPoint,
}

//
impl CollisionBox {
    pub fn get_point1(&mut self) -> PositionPoint { self.point1.clone() }
    pub fn get_point2(&mut self) -> PositionPoint { self.point2.clone() }

    pub fn to_string(&mut self) -> String { 
        format!("Point 1: {p1}\nPoint 2:{p2}", p1 = self.point1.to_string(), p2 = self.point2.to_string())
    }
}

//
#[derive(Clone)]
pub struct Object {
    pub sprites: AssetList<Sprite>,
    pub active: bool,
    pub index_in_stack: u32,
    pub index_of_layer: usize,
    pub index_in_layer: usize,
    pub position: PositionPoint,
    pub origin_offset: PositionPoint,
    pub collision_boxes: Vec<CollisionBox>,
}

//
impl Object {
    pub fn get_active(&mut self) -> bool { self.active.clone() }
    pub fn get_index_in_layer(&mut self) -> rhai::INT { self.index_in_layer.clone() as rhai::INT }
    pub fn get_index_of_layer(&mut self) -> rhai::INT { self.index_of_layer.clone() as rhai::INT }
    pub fn get_index_in_stack(&mut self) -> rhai::INT { self.index_in_stack.clone() as rhai::INT }
    pub fn get_position(&mut self) -> PositionPoint { self.position.clone() }
    pub fn get_origin_offset(&mut self) -> PositionPoint { self.origin_offset.clone() }
    pub fn get_collision_boxes(&mut self) -> Dynamic { self.collision_boxes.clone().into() }
    pub fn get_sprites(&mut self) -> AssetList<Sprite> { self.sprites.clone() }

    pub fn set_position(&mut self, value: PositionPoint) { self.position = value; }
    pub fn set_sprites(&mut self, value: AssetList<Sprite>) { self.sprites = value; }

    pub fn to_string(&mut self) -> String {
        //
        let mut collision_boxes_str = String::new(); let mut i = 1;
        //
        for colli_box in &self.collision_boxes {
            //
            let mut s = colli_box.clone().to_string().replace("\n", "\n\t");
            //
            s.insert_str( 0, &format!("\n\n\t#{}\n\t", i));
            i += 1;
            //
            collision_boxes_str.push_str(&s);
        }
        //       
        format!("Position:\n\t{pos}\nOrigin Offset:\n\t{orig_off}\nCollision Box:{colli}", 
            pos = self.position.to_string(), orig_off = self.origin_offset.to_string(), 
            colli = collision_boxes_str)
    }

    //
    pub fn new(config: &Map, idx_in_stack: u32, idx_of_layer: usize,
    idx_in_layer: usize, init_x: f32, init_y: f32) -> Self {
        //
        let mut collision_boxes_vec: Vec<CollisionBox> = Vec::new();
        //
        for map in config["collision-boxes"].clone().into_typed_array::<Map>()
        .expect("Every object's config should contain a 'collision-boxes' array, which should only have object-like members.") {
            //
            collision_boxes_vec.push( CollisionBox {
                point1: PositionPoint { 
                    x: dynamic_to_number(&map["x1"])
                    .expect("Every collision box should contain an float 'x1' attribute."), 
                    y: dynamic_to_number(&map["y1"])
                    .expect("Every collision box should contain an float 'y1' attribute.") 
                },
                point2: PositionPoint { 
                    x: dynamic_to_number(&map["x2"])
                    .expect("Every collision box should contain an float 'x2' attribute."), 
                    y: dynamic_to_number(&map["y2"])
                    .expect("Every collision box should contain an float 'y2' attribute.") 
                }
            } );
        }
        //
        let mut sprites_vec: Vec<Sprite> = Vec::new();
        //
        for id in config["sprites"].clone().into_typed_array::<rhai::INT>()
        .expect("Every object's config should contain a 'sprites' array, which should only have integer members.") {
            //
            sprites_vec.push(Sprite::new(id as u32));
        }
        //
        Self {
            //
            sprites: AssetList::new(sprites_vec),
            //
            active: true,
            //
            index_in_stack: idx_in_stack,
            //
            index_of_layer: idx_of_layer,
            //
            index_in_layer: idx_in_layer,
            //
            position: PositionPoint { x: init_x, y: init_y },
            //
            origin_offset: PositionPoint { 
                x: dynamic_to_number(&config["origin-offset"].clone_cast::<Map>()["x"])
                .expect("Every object's config should contain a 'origin-offset' object with 'x' and 'y' float attributes."), 
                y: dynamic_to_number(&config["origin-offset"].clone_cast::<Map>()["y"])
                .expect("Every object's config should contain a 'origin-offset' object with 'x' and 'y' float attributes.") 
            },
            //
            collision_boxes: collision_boxes_vec,
        }
    }
    //
    pub fn recycle(&mut self, config: &Map, idx_of_layer: usize,
    idx_in_layer: usize, init_x: f32, init_y: f32) {
        //
        self.collision_boxes.clear();
        //
        for map in config["collision-boxes"].clone().into_typed_array::<Map>()
        .expect("Every object's config should contain a 'collision-boxes' array, which should only have object-like members.") {
            //
            self.collision_boxes.push( CollisionBox {
                point1: PositionPoint { 
                    x: dynamic_to_number(&map["x1"])
                    .expect("Every collision box should contain an float 'x1' attribute."), 
                    y: dynamic_to_number(&map["y1"])
                    .expect("Every collision box should contain an float 'y1' attribute.") 
                },
                point2: PositionPoint { 
                    x: dynamic_to_number(&map["x2"])
                    .expect("Every collision box should contain an float 'x2' attribute."), 
                    y: dynamic_to_number(&map["y2"])
                    .expect("Every collision box should contain an float 'y2' attribute.") 
                }
            } );
        }
        //
        self.sprites.recycle(config["sprites"].clone().into_typed_array::<rhai::INT>()
        .expect("Every object's config should contain a 'sprites' array, which should only have integer members."));
        //
        self.active = true;
        //
        self.index_of_layer = idx_of_layer;
        //
        self.index_in_layer = idx_in_layer;
        //
        self.position.x = init_x;
        //
        self.position.y = init_y;
        //
        self.origin_offset.x = dynamic_to_number(&config["origin-offset"].clone_cast::<Map>()["x"])
        .expect("Every object's config should contain a 'origin-offset' object with 'x' and 'y' float attributes.");
        //
        self.origin_offset.y = dynamic_to_number(&config["origin-offset"].clone_cast::<Map>()["y"])
        .expect("Every object's config should contain a 'origin-offset' object with 'x' and 'y' float attributes.");
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

    pub fn to_string(&mut self) -> String {
        let mut instances_str = String::new(); let mut i = 1;
        //
        for inst in &self.instances {
            //
            instances_str.push_str(&format!("\n\n\t#{i} - {}", inst));
            //
            i += 1;
        }
        //
        format!("Name: {name}\nInstances:{instances}", name = self.name.clone(), instances = instances_str )
    }
}

//
#[derive(Clone)]
pub struct Camera {
    pub position: PositionPoint,
    pub zoom: f32,
}

//
impl Camera {
    pub fn get_position(&mut self) -> PositionPoint { self.position.clone() }
    pub fn get_zoom(&mut self) -> rhai::FLOAT { self.zoom.clone() as rhai::FLOAT }

    pub fn set_position(&mut self, value: PositionPoint) { self.position = value; }
    pub fn set_zoom(&mut self, value: rhai::FLOAT) { self.zoom = value as f32; }

    pub fn to_string(&mut self) -> String { 
        format!("{position} zoom - {zoom}", position = self.position.to_string(), zoom = self.zoom)
    }
}

//
#[derive(Clone)]
pub struct Scene {
    pub width: f32,
    pub height: f32,
    pub objects_len: usize,
    pub runtimes_len: usize,
    pub layers_len: usize,
    pub in_color: String,
    pub out_color: String,
    pub layers: Vec<Layer>,
    pub camera: Camera,
}

//
impl Scene {
    pub fn get_width(&mut self) -> rhai::FLOAT { self.width.clone() as rhai::FLOAT }
    pub fn get_height(&mut self) -> rhai::FLOAT { self.height.clone() as rhai::FLOAT }
    pub fn get_inside_color(&mut self) -> String { self.in_color.clone() }
    pub fn get_outside_color(&mut self) -> String { self.out_color.clone() }
    pub fn get_layers(&mut self) -> Dynamic { self.layers[0..self.layers_len].to_vec().into() }
    pub fn get_objects_len(&mut self) -> rhai::INT { self.objects_len.clone() as rhai::INT }
    pub fn get_runtimes_len(&mut self) -> rhai::INT { self.runtimes_len.clone() as rhai::INT }
    pub fn get_camera(&mut self) -> Camera { self.camera.clone() }

    pub fn set_width(&mut self, value: rhai::FLOAT) { self.width = value as f32; }
    pub fn set_height(&mut self, value: rhai::FLOAT) { self.height = value as f32; }
    pub fn set_inside_color(&mut self, value: &str) { self.in_color.clear(); self.in_color.push_str(value); }
    pub fn set_outside_color(&mut self, value: &str) { self.out_color.clear(); self.out_color.push_str(value); }
    pub fn set_camera(&mut self, value: Camera) { self.camera = value; }

    pub fn to_string(&mut self) -> String {
        let mut layers_str =  String::new(); let mut i = 1;
        //
        for layer in &self.layers {
            //
            let mut s = layer.clone().to_string().replace("\n", "\n\t\t");
            //
            s.insert_str( 0, &format!("\n\n\t\t#{}\n\t\t", i));
            i += 1;
            //
            layers_str.push_str(&s);
        }
        //
        format!("Scene:\n\twidth - {width} height - {height}\n\tinside color - {in_color} outside color - {out_color}\n\tCamera:\n\t\t{camera}\n\tLayers:{layers}", 
            width = self.width, height = self.height, in_color = self.in_color, out_color = self.out_color, 
            camera = &mut self.camera.to_string(), layers = layers_str)
    }

    //
    pub fn new(config: &Map) -> Self {
        //
        let mut layers_vec: Vec<Layer> = Vec::new();
        //
        for map in config["layers"].clone().into_typed_array::<Map>()
        .expect("Every scene's config should contain a 'layers' array, which should only have object-like members.") {
            //
            layers_vec.push( Layer { 
                name: map["name"].clone().into_string()
                .expect("Every member in the 'layers' array of a scene's config should have a string 'name' attribute."),
                instances: {
                    //
                    let mut instances_vec: Vec<u32> = Vec::new();
                    //
                    for index in map["instances"].clone().into_array()
                    .expect(concat!("Every member in the 'layers' array of a scene's config should",
                    " have a 'instances' array, which should only have object-like members.")) {
                        //
                        instances_vec.push( dynamic_to_number(&index)
                            .expect(concat!("Every member in an 'instances' array of a 'layers' array",
                            " of a scene's config should contain an integer 'index' attribute.")) as u32
                        );
                    }
                    instances_vec
                }
            } );
        }
        //
        Self {
            //
            width: dynamic_to_number(&config["width"])
            .expect("Every scene's config should contain an integer 'width' attribute.") as f32,
            //
            height: dynamic_to_number(&config["height"])
            .expect("Every scene's config should contain an integer 'height' attribute.") as f32,
            //
            objects_len: config["object-instances"].clone().into_array()
            .expect("Every scene's config should contain an array 'object-instances' attribute.").len(),
            //
            layers_len: layers_vec.len(),
            //
            runtimes_len: 0,
            //
            in_color: config["background-color"].clone().into_string()
            .expect("Every scene's config should contain a string 'background-color' attribute."),
            //
            out_color: config["outside-color"].clone().into_string()
            .expect("Every scene's config should contain a string 'outside-color' attribute."),
            //
            camera: Camera {
                //
                position: PositionPoint { 
                    x: dynamic_to_number(&config["camera-position"].clone_cast::<Map>()["x"])
                    .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes."), 
                    y: dynamic_to_number(&config["camera-position"].clone_cast::<Map>()["y"])
                    .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes.") 
                }, 
                //
                zoom: 1 as f32 
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
        for map in config["layers"].clone().into_typed_array::<Map>()
        .expect("Every scene's config should contain a 'layers' array, which should only have object-like members.") {
            //
            if i < self.layers.len() {
                //
                self.layers[i].name.clear();
                self.layers[i].name.push_str(&map["name"].clone().into_string()
                .expect("Every member in the 'layers' array of a scene's config should have a string 'name' attribute."));
                
                //
                self.layers[i].instances.clear();
                for index in map["instances"].clone().into_array()
                .expect(concat!("Every member in the 'layers' array of a scene's config should",
                " have a 'instances' array, which should only have object-like members.")) {
                    //
                    self.layers[i].instances.push(dynamic_to_number(&index)
                    .expect(concat!("Every member in an 'instances' array of a 'layers' array",
                    " of a scene's config should contain an integer 'index' attribute.")) as u32);
                }
                //
                i += 1;
                continue;
            }
            //
            self.layers.push( Layer { 
                name: map["name"].clone().into_string()
                .expect("Every member in the 'layers' array of a scene's config should have a string 'name' attribute."),
                instances: {
                    //
                    let mut instances_vec: Vec<u32> = Vec::new();
                    //
                    for index in map["instances"].clone().into_array()
                    .expect(concat!("Every member in the 'layers' array of a scene's config should",
                    " have a 'instances' array, which should only have object-like members.")) {
                        //
                        instances_vec.push(dynamic_to_number(&index)
                        .expect(concat!("Every member in an 'instances' array of a 'layers' array",
                        " of a scene's config should contain an integer 'index' attribute.")) as u32);
                    }
                    instances_vec
                }
            } );
            //
            i += 1;
        }
        //
        self.layers_len = i;
        //
        self.runtimes_len = 0;
        //
        self.objects_len = config["object-instances"].clone().into_array()
        .expect("Every scene's config should contain an array 'object-instances' attribute.").len();
        //
        self.width = dynamic_to_number(&config["width"])
        .expect("Every scene's config should contain an integer 'width' attribute.") as f32;
        //
        self.height = dynamic_to_number(&config["height"])
        .expect("Every scene's config should contain an integer 'height' attribute.") as f32;
        //
        self.in_color.clear();
        self.in_color.push_str(&config["background-color"].clone().into_string()
        .expect("Every scene's config should contain a string 'background-color' attribute."));
        //
        self.out_color.clear();
        self.out_color.push_str(&config["outside-color"].clone().into_string()
        .expect("Every scene's config should contain a string 'outside-color' attribute."));
        //
        self.camera = Camera {
            //
            position: PositionPoint { 
                x: dynamic_to_number(&config["camera-position"].clone_cast::<Map>()["x"])
                .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes."), 
                y: dynamic_to_number(&config["camera-position"].clone_cast::<Map>()["y"])
                .expect("Every scene's config should contain a 'camera-position' object with 'x' and 'y' float attributes.") 
            }, 
            //
            zoom: 1 as f32,
        };
    }
}