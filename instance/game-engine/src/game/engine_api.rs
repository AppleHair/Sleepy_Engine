
use crate::data;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use web_sys::console::log_1;
use rhai::{Engine, Scope, AST, Map, packages::{Package, StandardPackage}, EvalAltResult, Dynamic};

pub mod element;
pub mod asset;

//
pub fn dynamic_to_number(dynam: &Dynamic) -> Result<f32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f32);
    }
    Ok(dynam.as_float()? as f32)
}

//
pub struct KeyState {
    pub is_held: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

//
#[derive(Clone, Copy)]
pub enum TableRow {
    Metadata(u8),
    Element(u32, u8),
    Asset(u32, u8),
}

impl TableRow {
    fn to_err_string(&self, err: &str) -> String {
        let self_str = match self.clone() {
            //
            Self::Metadata(1) => String::from("\non 'Project Configurations'."),
            //
            Self::Metadata(2) => String::from("\non 'State Manager'."),
            //
            Self::Element(id, kind) => format!("\non the '{}' {kind_str}.", data::get_element_name(id.clone()),
            kind_str = match kind { 1 => "object", 2 => "scene", _ => "element" }),
            //
            Self::Asset(id, kind) => format!("\non the '{}' {kind_str}.", data::get_element_name(id.clone()),
            kind_str = match kind { 1 => "sprite", 2 => "audio", 3 => "font", _ => "asset" }),
            //
            _ => String::new(),
        };
        format!("{}{}", err, self_str)
    }
}

impl Default for TableRow {
    fn default() -> Self { Self::Element(Default::default(), Default::default()) }
}

//
#[derive(Default)]
pub struct ElementDefinition {
    pub row: TableRow,
    pub ast: AST,
    pub config: Map,
}

//
impl ElementDefinition {
    //
    pub fn new(engine: &Engine, row: TableRow) -> Result<Self, String> {
        //
        let ast = engine.compile(&match row {
            TableRow::Metadata(id) => data::get_metadata_script(id),
            TableRow::Element(id, _) => data::get_element_script(id),
            TableRow::Asset(_, _) => { return Err("Can't define an asset as an element.".into()) },
        });
        //
        if let Some(err) = ast.as_ref().err() {
            //
            return Err(row.to_err_string(&err.to_string()));
        }
        //
        let json = engine.parse_json(&match row {
            TableRow::Metadata(id) => data::get_metadata_config(id),
            TableRow::Element(id, _) => data::get_element_config(id),
            TableRow::Asset(_, _) => { return Err("Can't define an asset as an element.".into()) },
        }, false);
        //
        if let Some(err) = json.as_ref().err() {
            //
            return Err(row.to_err_string(&err.to_string()));
        }
        //
        Ok(
            Self {
                //
                row,
                //
                ast: ast.expect("This Err should have been caught by this function beforehand"),
                //
                config: json.expect("This Err should have been caught by this function beforehand"),
            }
        )
    }
}

//
#[derive(Default)]
pub struct ElementScript {
    pub definition: Rc<ElementDefinition>,
    pub scope: Scope<'static>,
}

//
impl ElementScript {
    //
    pub fn new(engine: &Engine, def: Rc<ElementDefinition>) -> Result<Self, String> {
        //
        let mut inst = Self {
            //
            definition: def,
            //
            scope: Scope::new(),
        };
        //
        if let Some(err) = engine.run_ast_with_scope(&mut inst.scope,
        &inst.definition.ast).err() {
            //
            return Err(inst.definition.row.to_err_string(&err.to_string()));
        }
        //
        Ok(inst)
    }

    //
    pub fn recycle(&mut self, engine: &Engine, def: Rc<ElementDefinition>) -> Result<(), String> {
        //
        self.definition = def;
        //
        self.scope.clear();
        //
        if let Some(err) = engine.run_ast_with_scope(&mut self.scope,
        &self.definition.ast).err() {
            //
            return Err(self.definition.row.to_err_string(&err.to_string()));
        }
        //
        Ok(())
    }

    //
    pub fn call_fn(&mut self, engine: &Engine, name: &str, args: impl rhai::FuncArgs) -> Result<(), String> {
        //
        if let Some(err) = engine.call_fn::<()>(&mut self.scope,
        &self.definition.ast, name, args).err() {
            //
            return Err(self.definition.row.to_err_string(&err.to_string()));
        }
        //
        Ok(())
    }
}

pub struct State(pub Rc<RefCell<Dynamic>>);
pub struct Scene(pub Rc<RefCell<Dynamic>>);
pub struct Object(pub Rc<RefCell<Dynamic>>);

pub struct Element<T> {
    pub map: T,
    pub script: Rc<RefCell<ElementScript>>,
}

impl Element<State> {
    pub fn new_state(engine: &Engine, def: Rc<ElementDefinition>) -> Result<Self, String> {
        //
        let mut script = ElementScript::new(&engine, def)?;
        //
        match script.definition.row {
            //
            TableRow::Metadata(2) => {
                //
                let shared_map: State = State(Rc::new(RefCell::new(Dynamic::from_map(Map::default()))));
                //
                if let Some(map) = script.scope.remove::<Map>("State") {
                    let _ = shared_map.0.replace(Dynamic::from_map(map));
                }
                //
                script.scope.push_dynamic("State", Dynamic::from(Rc::clone(&shared_map.0)));
                //
                Ok(Self {
                    map: shared_map,
                    script: Rc::new(RefCell::new(script)),
                })
            },
            //
            _ => Err(script.definition.row.to_err_string(concat!("Fatal Error: Tried to create the",
            " state manager with the wrong definition.")))
        }
    }
}

impl Element<Scene> {
    //
    pub fn new_scene(engine: &Engine, def: Rc<ElementDefinition>) -> Result<Self, String> {
        //
        let mut script = ElementScript::new(&engine, def)?;
        //
        match script.definition.row {
            //
            TableRow::Element(_, 2) => {
                //
                let shared_map: Scene = Scene(Rc::new(RefCell::new(
                    Dynamic::from(element::Scene::new(&script.definition.config))
                )));
                //
                script.scope.push_dynamic("Scene", Dynamic::from(Rc::clone(&shared_map.0)));
                //
                Ok(Self {
                    map: shared_map,
                    script: Rc::new(RefCell::new(script)),
                })
            },
            //
            _ => Err(script.definition.row.to_err_string(concat!("Fatal Error: Tried to create a",
            " scene with the wrong definition.")))
        }
    }
    //
    pub fn recycle_scene(&self, engine: &Engine, def: Rc<ElementDefinition>) -> Result<(), String> {
        //
        let script = Rc::clone(&self.script);
        //
        let map = Rc::clone(&self.map.0);
        //
        map.borrow_mut().write_lock::<element::Scene>().expect("write_lock cast should succeed")
        .recycle(&def.config);
        //
        script.borrow_mut().recycle(&engine, def)?;
        //
        script.borrow_mut().scope.push_dynamic("Scene", Dynamic::from(Rc::clone(&map)));
        //
        Ok(())
    }
}

impl Element<Object> {
    //
    pub fn new_object(engine: &Engine, def: Rc<ElementDefinition>,
    object_info: (u32, usize, usize, f32, f32)) -> Result<Self, String> {
        //
        let mut script = ElementScript::new(&engine, def)?;
        //
        match script.definition.row {
            //
            TableRow::Element(_, 1) => {
                //
                let shared_map: Object;
                //
                shared_map = Object(Rc::new(RefCell::new(
                        Dynamic::from(element::Object::new(&script.definition.config,
                        object_info.0, object_info.1,
                        object_info.2, object_info.3, object_info.4))
                )));
                //
                script.scope.push_dynamic("Object", Dynamic::from(Rc::clone(&shared_map.0)));
                //
                Ok(Self {
                    map: shared_map,
                    script: Rc::new(RefCell::new(script)),
                })
            },
            //
            _ => Err(script.definition.row.to_err_string(concat!("Fatal Error: Tried to create an",
            " object with the wrong definition.")))
        }
    }
    //
    pub fn recycle_object(&self, engine: &Engine, def: Rc<ElementDefinition>,
    object_info: (usize, usize, f32, f32)) -> Result<(), String> {
        //
        let script = Rc::clone(&self.script);
        //
        let map = Rc::clone(&self.map.0);
        //
        map.borrow_mut().write_lock::<element::Object>().expect("write_lock cast should succeed")
        .recycle(&def.config, object_info.0, object_info.1,
        object_info.2, object_info.3);
        //
        script.borrow_mut().recycle(&engine, def)?;
        //
        script.borrow_mut().scope.push_dynamic("Object", Dynamic::from(Rc::clone(&map)));
        //
        Ok(())
    }
}

//
pub fn create_api(object_defs: &mut HashMap<u32,Rc<ElementDefinition>>) -> Result<(Engine, Element<State>, Element<Scene>,
Rc<RefCell<u32>>, Rc<RefCell<u32>>, Rc<RefCell<Vec<Element<Object>>>>, Rc<RefCell<HashMap<String, KeyState>>>), String> {
    // Create an 'Engine'
    let mut engine = Engine::new_raw();

    // Register API features to the 'Engine'
    engine.on_print(|text| { log_1(&wasm_bindgen::JsValue::from_str(text)); })
          .register_type_with_name::<element::PositionPoint>("Position")
          .register_get_set("x", element::PositionPoint::get_x, element::PositionPoint::set_x)
          .register_get_set("x_int", element::PositionPoint::get_x_int, element::PositionPoint::set_x_int)
          .register_get_set("y", element::PositionPoint::get_y, element::PositionPoint::set_y)
          .register_get_set("y_int", element::PositionPoint::get_y_int, element::PositionPoint::set_y_int)
          .register_fn("to_string", element::PositionPoint::to_string)
          .register_type_with_name::<element::CollisionBox>("CollisionBox")
          .register_get("point1", element::CollisionBox::get_point1)
          .register_get("point2", element::CollisionBox::get_point2)
          .register_fn("to_string", element::CollisionBox::to_string)
          .register_type_with_name::<asset::Sprite>("Sprite")
          .register_get("id", asset::Sprite::get_id)
          .register_get("id_float", asset::Sprite::get_id_float)
          .register_get("cur_animation", asset::Sprite::get_cur_animation)
          .register_set("cur_animation", asset::Sprite::set_cur_animation)
          .register_get_set("cur_frame", asset::Sprite::get_cur_frame, asset::Sprite::set_cur_frame)
          .register_get_set("cur_frame_float", asset::Sprite::get_cur_frame_float,asset::Sprite::set_cur_frame_float)
          .register_get("animation_time", asset::Sprite::get_animation_time)
          .register_get("animation_time_int", asset::Sprite::get_animation_time_int)
          .register_get_set("repeat", asset::Sprite::get_repeat, asset::Sprite::set_repeat)
          .register_get("is_animation_finished", asset::Sprite::get_is_animation_finished)
          .register_fn("play_animation", asset::Sprite::play_animation)
          .register_fn("play_animation", asset::Sprite::play_animation_on_time)
          .register_type_with_name::<asset::AssetList<asset::Sprite>>("AssetList<Sprite>")
          .register_indexer_get(asset::AssetList::<asset::Sprite>::get_asset)
          .register_indexer_set(asset::AssetList::<asset::Sprite>::set_asset)
          .register_fn("lock", asset::AssetList::<asset::Sprite>::lock)
          .register_fn("lock", asset::AssetList::<asset::Sprite>::lock_with_indcies)
          .register_fn("len", asset::AssetList::<asset::Sprite>::len)
          .register_get("len", asset::AssetList::<asset::Sprite>::len)
          .register_fn("contains", asset::AssetList::<asset::Sprite>::contains)
          .register_type_with_name::<element::Object>("Object")
          .register_get_set("position", element::Object::get_position, element::Object::set_position)
          .register_get_set("sprites", element::Object::get_sprites, element::Object::set_sprites)
          .register_get("origin_offset", element::Object::get_origin_offset)
          .register_get("collision_boxes", element::Object::get_collision_boxes)
          .register_get("active", element::Object::get_active)
          .register_get("index_in_stack", element::Object::get_index_in_stack)
          .register_get("index_of_layer", element::Object::get_index_of_layer)
          .register_get("index_in_layer", element::Object::get_index_in_layer)
          .register_fn("to_string", element::Object::to_string)
          .register_type_with_name::<element::Camera>("Camera")
          .register_get_set("position", element::Camera::get_position, element::Camera::set_position)
          .register_get_set("zoom", element::Camera::get_zoom, element::Camera::set_zoom)
          .register_get_set("zoom_int", element::Camera::get_zoom_int, element::Camera::set_zoom_int)
          .register_fn("to_string", element::Camera::to_string)
          .register_type_with_name::<element::Layer>("Layer")
          .register_get("name", element::Layer::get_name)
          .register_get("instances", element::Layer::get_instances)
          .register_fn("to_string", element::Layer::to_string)
          .register_type_with_name::<element::Scene>("Scene")
          .register_get_set("width", element::Scene::get_width, element::Scene::set_width)
          .register_get_set("width_int", element::Scene::get_width_int, element::Scene::set_width_int)
          .register_get_set("height", element::Scene::get_height, element::Scene::set_height)
          .register_get_set("height_int", element::Scene::get_height_int, element::Scene::set_height_int)
          .register_get("in_color", element::Scene::get_inside_color)
          .register_set("in_color", element::Scene::set_inside_color)
          .register_get("out_color", element::Scene::get_outside_color)
          .register_set("out_color", element::Scene::set_outside_color)
          .register_get_set("camera", element::Scene::get_camera, element::Scene::set_camera)
          .register_get("objects_len", element::Scene::get_objects_len)
          .register_get("runtimes_len", element::Scene::get_runtimes_len)
          .register_get("layers", element::Scene::get_layers)
          .register_fn("to_string", element::Scene::to_string);

    // Register the standard packages
    let std_package = StandardPackage::new();
    // Load the standard packages into the 'Engine'
    std_package.register_into_engine(&mut engine);

    // Create a new element instance for the state manager.
    // This instance will borrow its definition and contain
    // the element's 'Scope'.
    let state_manager = Element::new_state(&engine, 
        Rc::new(
            ElementDefinition::new(&engine, 
                TableRow::Metadata(2)
            )?
        )
    )?;

    // Register a variable definition filter.
    engine.on_def_var(|is_runtime, info, context| {
        match info.name {
            // If the name of the
            // defined variable is 'State'
            "State" => {
                if !context.scope().contains(info.name) {
                    // If the variable doesn't
                    // exist in the scope already
                    // (which means it's not the 
                    // state manager's scope)
                    return Err("Can't define State outside the State Manager script.".into());
                } else if info.nesting_level > 0 as usize  {
                    // If the variable is being
                    // defined outside the global scope
                    return Err("Can't define State outside the global scope.".into());
                } else {
                    return Ok(true);
                } 
            },
            // Otherwise, continue with the normal variable definition process,
            // where script runtime definitions of 'Scene' or 'Object' are forbidden.
            _ => Ok((info.name != "Scene" && info.name != "Object") || !is_runtime)
        }
    });
    

    // Receive the rowid of the initial scene from the the state manager's config
    let cur_scene_id = Rc::new(RefCell::new(
        state_manager.script.borrow().definition.config["initial-scene"].as_int()
        .expect("The value of 'initial-scene' in the state manager's config should be an integer") as u32
    ));
    //
    let prv_scene_id = Rc::new(RefCell::new(0_u32));
    //
    let cur_scene = Element::new_scene(&engine, 
        Rc::new(
            ElementDefinition::new(&engine, 
                TableRow::Element(cur_scene_id.borrow().clone(), 2)
            )?
        )
    )?;
    //
    let object_stack: Rc<RefCell<Vec<Element<Object>>>> = Rc::new(RefCell::new(Vec::new()));

    //
    {
        let instances = cur_scene.script.borrow().definition.config["object-instances"].clone()
        .into_typed_array::<Map>().expect(concat!("Every object's config should contain a 'object-instances'",
        " array, which should only have object-like members."));

        object_stack.borrow_mut().resize_with(instances.len(), || { Element { map: Object(Default::default()), script: Default::default() } });

        //
        let mut i = 0_usize;
        //
        for layer in cur_scene.map.0.borrow().read_lock::<element::Scene>()
        .expect("read_lock cast should succeed").layers.clone() {
            //
            let mut j = 0_usize;
            //
            for idx in layer.instances {
                //
                let inst_info = instances.get(idx as usize)
                .expect("The indexes specified in every element of every layer's instances array should be correct.");
                //
                let ent_id = dynamic_to_number(&inst_info["id"])
                .expect(concat!("Every instance in the 'object-instances' array of an object's",
                " config should contain an integer 'id' attribute.")) as u32;
                let (init_x, init_y) = (
                    dynamic_to_number(&inst_info["x"])
                    .expect(concat!("Every instance in the 'object-instances' array of an object's",
                    " config should contain an float 'x' attribute.")), 
                    dynamic_to_number(&inst_info["y"])
                    .expect(concat!("Every instance in the 'object-instances' array of an object's",
                    " config should contain an float 'y' attribute.")),
                );
                //
                if !object_defs.contains_key(&ent_id) {
                    //
                    object_defs.insert(ent_id, 
                        Rc::new(
                            ElementDefinition::new(&engine, 
                                TableRow::Element(ent_id, 1)
                            )?
                        )
                    );
                }
                //
                *object_stack.borrow_mut().get_mut(idx as usize)
                .expect("The indexes specified in every element of every layer's instances array should be correct.") = 
                Element::new_object(&engine,
                    Rc::clone(
                        object_defs.get(&ent_id)
                        .expect("object_defs.get(&inst_id_u32) should have had the object's definition by now")
                    ), (idx, i, j, init_x, init_y)
                )?;
                //
                j += 1;
            }
            //
            i += 1;
        }
    }

    let api_state_map = Rc::clone(&state_manager.map.0);
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    // Register a variable resolver.
    engine.on_var(move |name, _, context| {
        match name {
            // If the name of the
            // accessed variable is 'State'
            "State" => {
                if context.scope().contains(name) {
                    // If the variable exists
                    // in the scope already
                    // (which means it's the
                    // state manager's scope)
                    Ok(None) 
                } else {
                    // Otherwise, return a clone
                    // of the value of the state map
                    Ok(Some(api_state_map.borrow().flatten_clone())) 
                }
            },
            "Scene" => {
                if context.scope().contains(name) {
                    // If the variable exists
                    // in the scope already
                    // (which means it's the
                    // state manager's scope)
                    Ok(None) 
                } else {
                    // Otherwise, return a clone
                    // of the value of the state map
                    Ok(Some(api_scene_map.borrow().flatten_clone()))
                }
            },
            // Otherwise, continue with the normal variable resolution process.
            _ => Ok(None)
        }
    });

    //
    let key_states: Rc<RefCell<HashMap<String, KeyState>>> = Rc::new(RefCell::new(HashMap::new()));

    //
    let api_key_states = Rc::clone(&key_states);
    engine.register_fn("key_is_held", move |key: &str| -> bool {
        //
        if let Some(state) = api_key_states.borrow().get(key) {
            //
            state.is_held
        } else {
            //
            false
        }
    });

    //
    let api_key_states = Rc::clone(&key_states);
    engine.register_fn("key_just_pressed", move |key: &str| -> bool {
        //
        if let Some(state) = api_key_states.borrow().get(key) {
            //
            state.just_pressed
        } else {
            //
            false
        }
    });

    //
    let api_key_states = Rc::clone(&key_states);
    engine.register_fn("key_just_released", move |key: &str| -> bool {
        //
        if let Some(state) = api_key_states.borrow().get(key) {
            //
            state.just_released
        } else {
            //
            false
        }
    });

    //
    let api_cur_scene_script = Rc::clone(&cur_scene.script);
    engine.register_fn("is_cur_scene", move |name: &str| -> Result<bool, Box<EvalAltResult>> {
        //
        if let Ok(borrow) = api_cur_scene_script.try_borrow() {
            //
            Ok(if let TableRow::Element(id,2) = borrow.definition.row {
                //
                data::get_element_name(id) == name
            } else { false })
        } else {
            //
            Err(concat!("Can't use the 'is_cur_scene' function inside a scene",
            " script! Note: This script only runs when its scene is the current scene,",
            " so you don't need to use this function inside this script.").into())
        }
    });
    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    engine.register_fn("get_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<element::Object, Box<EvalAltResult>> {
        //
        let objects_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").objects_len;
        //
        let runtimes_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").runtimes_len;
        //
        if idx >= (objects_len+runtimes_len) as rhai::INT {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(objects_len+runtimes_len, idx, context.position())));
        }
        //
        let object_stack_borrow = api_object_stack.borrow();
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            Ok(element.map.0.borrow().clone_cast::<element::Object>())
        } else {
            //
            Err(Box::new(EvalAltResult::ErrorArrayBounds(objects_len+runtimes_len, idx, context.position())))
        }
    });
    //
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    engine.register_fn("object_is_valid", move |idx: rhai::INT| -> bool {
        //
        let objects_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").objects_len;
        //
        let runtimes_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").runtimes_len;
        //
        idx < (objects_len+runtimes_len) as rhai::INT && idx > -1
    });
    //
    let api_object_stack = Rc::clone(&object_stack);
    engine.register_fn("object_is_active", move |context: rhai::NativeCallContext, 
    idx: rhai::INT| -> Result<bool, Box<EvalAltResult>> {
        //
        let object_stack_borrow = api_object_stack.borrow();
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            Ok(element.map.0.borrow().read_lock::<element::Object>()
            .expect("read_lock cast should succeed").active)
        } else {
            //
            Err(Box::new(EvalAltResult::ErrorArrayBounds(object_stack_borrow.len(), idx, context.position())))
        }
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    engine.register_fn("activate_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<(), Box<EvalAltResult>> {
        //
        let objects_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").objects_len;
        //
        let runtimes_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").runtimes_len;
        //
        if idx >= (objects_len+runtimes_len) as rhai::INT {
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(objects_len+runtimes_len, idx, context.position())));
        }
        //
        let object_stack_borrow = api_object_stack.borrow();
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            element.map.0.borrow_mut().write_lock::<element::Object>()
            .expect("write_lock cast should succeed").active = true;
            //
            Ok(())
        } else {
            //
            Err(Box::new(EvalAltResult::ErrorArrayBounds(objects_len+runtimes_len, idx, context.position())))
        }
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    engine.register_fn("deactivate_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<(), Box<EvalAltResult>> {
        //
        let object_stack_borrow = api_object_stack.borrow();
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            element.map.0.borrow_mut().write_lock::<element::Object>()
            .expect("write_lock cast should succeed").active = false;
            //
            Ok(())
        } else {
            //
            Err(Box::new(EvalAltResult::ErrorArrayBounds(object_stack_borrow.len(), idx, context.position())))
        }
    });

    //
    let api_state_manager_script = Rc::clone(&state_manager.script);
    engine.register_fn("message_state_manager", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if let Ok(mut borrow) = api_state_manager_script.try_borrow_mut() {
            //
            if let Some(err) = borrow.call_fn(context.engine(),&format!("message_{}", name), args).err() {
                //
                Err(format!("{}\nas a result of a call to 'message_state_manager'", err).into())
            } else { Ok(()) }
        } else {
            //
            Err(concat!("Can't use the 'message_state_manager' function while the state manager's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an element, which tried to message you back in the scope of that same message.").into())
        }
    });

    //
    let api_cur_scene_script = Rc::clone(&cur_scene.script);
    engine.register_fn("message_cur_scene", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if let Ok(mut borrow) = api_cur_scene_script.try_borrow_mut() {
            //
            if let Some(err) = borrow.call_fn(context.engine(),&format!("message_{}", name), args).err() {
                //
                Err(format!("{}\nas a result of a call to 'message_cur_scene'", err).into())
            } else { Ok(()) }
        } else {
            //
            Err(concat!("Can't use the 'message_cur_scene' function while the current scene's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an element, which tried to message you back in the scope of that same message.").into())
        }
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    engine.register_fn("message_object", move |context: rhai::NativeCallContext, idx: rhai::INT, 
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        let objects_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").objects_len;
        //
        let runtimes_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").runtimes_len;
        //
        if idx >= (objects_len+runtimes_len) as rhai::INT {
            //
            let info = "Argument 'idx' was out of bounds in call to 'message_object'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx, objects_len+runtimes_len);
            //
            return Err(full_s.into());
        }
        //
        let object_stack_borrow = api_object_stack.borrow();
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            if let Ok(mut borrow) = element.script.try_borrow_mut() {
                //
                if let Some(err) = borrow.call_fn(context.engine(),&format!("message_{}", name), args).err() {
                    //
                    Err(format!("{}\nas a result of a call to 'message_object'", err).into())
                } else { Ok(()) }
            } else {
                //
                Err(concat!("Can't use the 'message_object' function while that object's script is running",
                " (is handling another callback). Note: This might have happened because you tried to message yourself,",
                " or messaged an element, which tried to message you back in the scope of that same message.").into())
            }
        } else {
            //
            let info = "Argument 'idx' was out of bounds in call to 'message_object'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx, objects_len+runtimes_len);
            //
            return Err(full_s.into());
        }
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    engine.register_fn("instance_switch_layer", move |layer_from: rhai::INT, 
    layer_idx_from: rhai::INT, layer_to: rhai::INT| -> Result<(), Box<EvalAltResult>> {
        //
        let object_stack_borrow = api_object_stack.borrow();
        //
        let mut scene_map_borrow = api_scene_map.borrow_mut();
        //
        let moving_stack_index: usize;
        //
        let to_update_stack_index: usize;
        //
        let new_layer_idx: rhai::INT;
        //
        if scene_map_borrow.read_lock::<element::Scene>()
        .expect("read_lock cast should succeed").layers.get(layer_to as usize).is_none() {
            //
            let info = "Argument 'layer_to' was out of bounds in call to 'instance_switch_layer'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on 'Scene.layers', when it only had {} elements.",
            info, layer_to, scene_map_borrow.read_lock::<element::Scene>()
            .expect("read_lock cast should succeed").layers.len());
            //
            return Err(full_s.into());
        }
        //
        if scene_map_borrow.write_lock::<element::Scene>()
        .expect("write_lock cast should succeed").layers.get_mut(layer_from as usize).is_some() {
            //
            let mut scene_mut = scene_map_borrow.write_lock::<element::Scene>()
            .expect("write_lock cast should succeed");
            //
            let layer = scene_mut.layers.get_mut(layer_from as usize).unwrap();
            //
            if layer.instances.get(layer_idx_from as usize).is_some() {
                //
                to_update_stack_index = layer.instances.last().unwrap().clone() as usize;
                //
                moving_stack_index = layer.instances.swap_remove(layer_idx_from as usize) as usize;
            } else {
                //
                let info = "Argument 'layer_idx_from' was out of bounds in call to 'instance_switch_layer'.";
                //
                let full_s = &format!("{}.\nTried to find index {} on 'Layer.instances', when it only had {} elements.",
                info, layer_idx_from, layer.instances.len());
                //
                return Err(full_s.into());
            }
        } else {
            //
            let info = "Argument 'layer_from' was out of bounds in call to 'instance_switch_layer'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on 'Scene.layers', when it only had {} elements.",
            info, layer_from, scene_map_borrow.read_lock::<element::Scene>()
            .expect("read_lock cast should succeed").layers.len());
            //
            return Err(full_s.into());
        }
        //
        new_layer_idx = scene_map_borrow.read_lock::<element::Scene>()
        .expect("read_lock cast should succeed").layers.get(layer_to as usize)
        .expect("The argument 'layer_to' should be a valid index if it passed the argument check.")
        .instances.len() as rhai::INT;
        //
        scene_map_borrow.write_lock::<element::Scene>()
        .expect("read_lock cast should succeed").layers.get_mut(layer_to as usize)
        .expect("The argument 'layer_to' should be a valid index if it passed the argument check.")
        .instances.push(moving_stack_index as u32);
        //
        let moving_object_map = Rc::clone(&object_stack_borrow.get(moving_stack_index)
            .expect("The indexes specified in every element of every layer's instances array should be correct.").map.0);
        //
        if moving_stack_index != to_update_stack_index {
            //
            let to_update_object_map = Rc::clone(&object_stack_borrow.get(to_update_stack_index)
                .expect("The indexes specified in every element of every layer's instances array should be correct.").map.0);
            //
            to_update_object_map.borrow_mut().write_lock::<element::Object>()
            .expect("write_lock cast should succeed").index_in_layer = layer_idx_from as usize; 
        }
        //
        let mut moving_object_borrow = moving_object_map.borrow_mut(); 
        //
        moving_object_borrow.write_lock::<element::Object>()
        .expect("write_lock cast should succeed").index_of_layer = layer_to as usize;
        //
        moving_object_borrow.write_lock::<element::Object>()
        .expect("write_lock cast should succeed").index_in_layer = new_layer_idx as usize;
        //
        Ok(())
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_scene_map = Rc::clone(&cur_scene.map.0);
    engine.register_fn("add_object_to_stack", move |context: rhai::NativeCallContext,
    idx: rhai::INT, layer_to: rhai::INT, init_x: rhai::FLOAT, init_y: rhai::FLOAT| -> Result<rhai::INT, Box<EvalAltResult>> {
        //
        let objects_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").objects_len;
        //
        let runtimes_len = api_scene_map.borrow()
        .read_lock::<element::Scene>().expect("read_lock cast should succeed").runtimes_len;
        //
        if idx >= (objects_len+runtimes_len) as rhai::INT {
            //
            let info = "Argument 'idx' was out of bounds in call to 'add_object_to_stack'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx, objects_len+runtimes_len);
            //
            return Err(full_s.into());
        }
        //
        let mut object_stack_borrow = api_object_stack.borrow_mut();
        //
        let mut scene_map_borrow = api_scene_map.borrow_mut();
        //
        let def_reference: Rc<ElementDefinition>;
        //
        let new_layer_idx: usize;
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            if let Ok(borrow) = element.script.try_borrow() {
                //
                def_reference = Rc::clone(&borrow.definition);
            } else {
                //
                return Err(concat!("Can't create a new object using a reference to an object",
                " whose script is running (is handling another callback). Note: It's recommended",
                " to deactivate objects which are used for making other objects, and not use",
                " 'message_' callbacks with this function, which might need to refer to objects",
                " involved in the current callback to create new objects.").into());
            }
        } else {
            //
            let info = "Argument 'idx' was out of bounds in call to 'add_object_to_stack'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx, objects_len+runtimes_len);
            //
            return Err(full_s.into());
        }
        //
        if let Some(layer) = scene_map_borrow.read_lock::<element::Scene>()
        .expect("read_lock cast should succeed").layers.get(layer_to as usize) {
            //
            new_layer_idx = layer.instances.len();
        } else {
            //
            let info = "Argument 'layer_to' was out of bounds in call to 'add_object_to_stack'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on 'Scene.layers', when it only had {} elements.",
            info, layer_to, scene_map_borrow.read_lock::<element::Scene>()
            .expect("read_lock cast should succeed").layers.len());
            //
            return Err(full_s.into());
        }
        //
        for object_idx in objects_len..object_stack_borrow.len() {
            //
            if object_idx == object_stack_borrow.len() {
                //
                break;
            }
            //
            let object = object_stack_borrow.get(object_idx)
            .expect("Shouldn't be out of bounds (scene_map_borrow.clone().stack_len..object_stack_borrow.len()-1).");
            //
            let object_map =  Rc::clone(&object.map.0);
            //
            if object_map.borrow().read_lock::<element::Object>()
            .expect("read_lock cast should succeed").active == false {
                //
                scene_map_borrow.write_lock::<element::Scene>()
                .expect("write_lock cast should succeed").layers.get_mut(layer_to as usize)
                .expect("The argument 'layer_to' should be a valid index if it passed the argument check.")
                .instances.push(object_map.borrow().read_lock::<element::Object>()
                .expect("read_lock cast should succeed").index_in_stack);
                //
                scene_map_borrow.write_lock::<element::Scene>()
                .expect("write_lock cast should succeed").layers.get_mut(object_map.borrow()
                .read_lock::<element::Object>().expect("read_lock cast should succeed").index_of_layer)
                .expect("The indexes specified in every object map should be correct.")
                .instances.swap_remove(object_map.borrow().read_lock::<element::Object>()
                .expect("read_lock cast should succeed").index_in_layer);
                //
                if let Err(err) = object.recycle_object(context.engine(), def_reference,
                (layer_to as usize, new_layer_idx, init_x as f32, init_y as f32)) {
                    //
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
                }
                //
                if objects_len+runtimes_len <= object_idx {
                    scene_map_borrow.write_lock::<element::Scene>()
                    .expect("write_lock cast should succeed").runtimes_len += 1;
                }
                //
                let index = object_map.borrow().read_lock::<element::Object>()
                .expect("read_lock cast should succeed").index_in_stack as rhai::INT;
                //
                return Ok(index);
            }
        }
        //
        let index = object_stack_borrow.len() as u32;
        //
        scene_map_borrow.write_lock::<element::Scene>()
        .expect("write_lock cast should succeed").layers.get_mut(layer_to as usize)
        .expect("The argument 'layer_to' should be a valid index if it passed the argument check.")
        .instances.push(index);
        //
        object_stack_borrow.push(
            {
                //
                let element = Element::new_object(context.engine(),
                def_reference, (index, layer_to as usize, new_layer_idx, init_x as f32, init_y as f32));
                //
                if let Some(err) = element.as_ref().err() {
                    //
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
                }
                //
                element.unwrap()
            }
        );
        //
        scene_map_borrow.write_lock::<element::Scene>()
        .expect("write_lock cast should succeed").runtimes_len += 1;
        //
        Ok(index as rhai::INT)
    });

    //
    let api_cur_scene_id = Rc::clone(&cur_scene_id);
    let api_prv_scene_id = Rc::clone(&prv_scene_id);
    engine.register_fn("switch_scene", move |id: rhai::INT| -> Result<(), Box<EvalAltResult>> {
        //
        if *api_prv_scene_id.borrow() != *api_cur_scene_id.borrow() {
            //
            return Err(concat!("Can't use the 'switch_scene' function while a scene is being created \\ switched.",
            " Note: This might have happened because you tried to use this function on the 'create' callback, which",
            " is being call within the process of creating a scene.").into());
        }
        //
        let kind = data::get_element_type(id as u32);
        //
        if kind == 2 {
            //
            *api_cur_scene_id.borrow_mut() = id as u32;
            //
            Ok(())
        } else {
            //
            Err("Tried to switch to a scene that doesn't exist using the 'switch_scene' function.".into())
        }
    });
    
    //
    engine.register_fn("element_name_to_id", |name: &str| -> Result<rhai::INT, Box<EvalAltResult>> {
        //
        let res = data::get_element_id(name) as rhai::INT;
        //
        if res == 0 {
            //
            Err(format!("Tried to use 'element_name_to_id' with an element name that doesn't exist ('{}').", name).into())
        } else {
            //
            Ok(res)
        }
    });

    // The API is done!
    Ok((engine, state_manager, cur_scene, cur_scene_id, prv_scene_id, object_stack, key_states))
}