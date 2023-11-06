
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use web_sys::console::log_1;
use rhai::{Engine, Scope, AST, Map, EvalAltResult, Dynamic,
        packages::{Package, StandardPackage}};

use crate::{data, game::TableRow};

pub mod element;
pub mod asset;

//
pub fn dynamic_to_number(dynam: &Dynamic) -> Result<f32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f32);
    }
    Ok(dynam.as_float()? as f32)
}

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
pub struct KeyState {
    pub is_held: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

//
pub struct ElementDefinition {
    pub config: Map,
    pub script: AST,
    pub row: TableRow,
}

//
impl ElementDefinition {
    //
    pub fn new(engine: &Engine, row: TableRow) -> Result<Rc<Self>, String> {
        //
        let ast = engine.compile(&match row {
            TableRow::Metadata => data::get_metadata_script(),
            TableRow::Element(rowid, _) => data::get_element_script(rowid),
            TableRow::Asset(rowid, type_num) => { return Err(
            format!("Can't define an asset as an element (on ElementDefinition::new())(name: '{}', id: {}, type: {})",
            data::get_asset_name(rowid), rowid, type_num)); },
        });
        //
        if let Some(err) = ast.as_ref().err() {
            //
            return Err(row.to_err_string(&err.to_string()));
        }
        //
        let json = engine.parse_json(&match row {
            TableRow::Metadata => data::get_metadata_config(),
            TableRow::Element(rowid, _) => data::get_element_config(rowid),
            TableRow::Asset(rowid, type_num) => { return Err(
            format!("Can't define an asset as an element (on ElementDefinition::new())(name: '{}', id: {}, type: {})",
            data::get_asset_name(rowid), rowid, type_num)); },
        }, false);
        //
        if let Some(err) = json.as_ref().err() {
            //
            return Err(row.to_err_string(&err.to_string()));
        }
        //
        Ok(Rc::new(Self {
            config: json.expect(
            concat!("This Err should",
            " have been caught by this function",
            " beforehand")), 
            script: ast.expect(
            concat!("This Err should",
            " have been caught by this function",
            " beforehand")),
            row,
        }))
    }
}

//
pub struct ElementResources {
    pub definition: Rc<ElementDefinition>,
    scope: Scope<'static>
}

impl ElementResources {
    //
    fn new(definition: Rc<ElementDefinition>) -> Self {
        //
        Self { definition, scope: Scope::new() }
    }
    //
    fn recycle(&mut self, definition: Rc<ElementDefinition>) {
        //
        self.definition = definition;
        self.scope.clear();
    }
    //
    pub fn reload(&mut self, engine: &Engine) -> Result<(), String> {
        //
        if let Some(err) = engine.run_ast_with_scope
        (&mut self.scope, &self.definition.script).err() {
            //
            return Err(self.definition.row.to_err_string(&err.to_string()));
        }
        //
        Ok(())
    }
    //
    pub fn call_fn(&mut self, engine: &Engine, name: &str, args: impl rhai::FuncArgs) -> Result<(), String> {
        //
        if !self.definition.script.iter_functions()
        .any(|func| { func.name == name}) {
            return Ok(());
        }
        //
        if let Some(err) = engine.call_fn::<()>
        (&mut self.scope, &self.definition.script, name, args).err() {
            //
            return Err(self.definition.row.to_err_string(&err.to_string()));
        }
        //
        Ok(())
    }
}

//
pub struct ElementHandler {
    pub properties: Rc<RefCell<Dynamic>>,
    pub resources: Rc<RefCell<ElementResources>>,
}

impl ElementHandler {
    pub fn new(def: &Rc<ElementDefinition>,
    object_info: Option<element::ObjectInitInfo>) -> Result<Self, String> {
        //
        let mut element_handler = Self {
            properties: Default::default(),
            resources: Rc::new(RefCell::new(
                ElementResources::new(Rc::clone(def))
            ))
        };
        //
        let row_copy = element_handler.resources.borrow().definition.row;
        //
        match row_copy {
            //
            TableRow::Metadata => {
                //
                let shared_map = Rc::new(RefCell::new(
                    Dynamic::from(element::Game::new(&element_handler.resources.borrow().definition.config))
                ));
                //
                element_handler.resources.borrow_mut().scope
                .push_dynamic("Game", Dynamic::from(Rc::clone(&shared_map)));
                element_handler.properties = shared_map;
                //
                Ok(element_handler)
            },
            //
            TableRow::Element(_, 2) => {
                //
                let shared_map = Rc::new(RefCell::new(
                    Dynamic::from(element::Scene::new(&element_handler.resources.borrow().definition.config))
                ));
                //
                element_handler.resources.borrow_mut().scope
                .push_dynamic("Scene", Dynamic::from(Rc::clone(&shared_map)));
                element_handler.properties = shared_map;
                //
                Ok(element_handler)
            },
            //
            TableRow::Element(rowid, 1) => {
                //
                if let Some(info) = object_info {
                    //
                    let shared_map = Rc::new(RefCell::new(
                        Dynamic::from(element::Object::new(&element_handler.resources.borrow().definition.config,info))
                    ));
                    //
                    element_handler.resources.borrow_mut().scope
                    .push_dynamic("Object", Dynamic::from(Rc::clone(&shared_map)));
                    element_handler.properties = shared_map;
                    //
                    Ok(element_handler)
                } else {
                    //
                    Err(format!("Tried to create object handler without 'object_info' parameter (name: '{}', id: {})", 
                    data::get_element_name(rowid), rowid))
                }
            },
            TableRow::Element(rowid, type_num) => {
                //
                Err(format!("In-valid element (on ElementHandler::new())(name: '{}', id: {}, type: {})",
                data::get_element_name(rowid), rowid, type_num))
            },
            TableRow::Asset(rowid, type_num) => {
                //
                Err(format!("Can't define an asset as an element (on ElementHandler::new())(name: '{}', id: {}, type: {})",
                data::get_asset_name(rowid), rowid, type_num))
            },
        }
    }

    pub fn recycle(&self, engine: &Engine, def: &Rc<ElementDefinition>,
    object_info: Option<element::ObjectInitInfo>) -> Result<(), String> {
        //
        self.resources.borrow_mut().recycle(Rc::clone(def));
        //
        let row_copy = self.resources.borrow().definition.row;
        //
        match row_copy {
            //
            TableRow::Element(_, 2) => {
                //
                self.properties.borrow_mut().write_lock::<element::Scene>()
                .expect("write_lock cast should succeed")
                .recycle(&self.resources.borrow().definition.config);
                //
                self.resources.borrow_mut().scope
                .push_dynamic("Scene", Dynamic::from(Rc::clone(&self.properties)));
                //
                self.resources.borrow_mut().reload(engine)?;
                //
                Ok(())
            },
            //
            TableRow::Element(rowid, 1) => {
                //
                if let Some(info) = object_info {
                    //
                    self.properties.borrow_mut().write_lock::<element::Object>()
                    .expect("write_lock cast should succeed")
                    .recycle(&self.resources.borrow().definition.config, info);
                    //
                    self.resources.borrow_mut().scope
                    .push_dynamic("Object", Dynamic::from(Rc::clone(&self.properties)));
                    //
                    self.resources.borrow_mut().reload(engine)?;
                    //
                    Ok(())
                } else {
                    //
                    Err(format!("Tried to recycle object handler without 'object_info' parameter (name: '{}', id: {})", 
                    data::get_element_name(rowid), rowid))
                }
            },
            //
            TableRow:: Metadata => {
                //
                Err("Tried to recycle State Manager.".into())
            },
            //
            TableRow::Element(rowid, type_num) => {
                //
                Err(format!("In-valid element (on ElementHandler::recycle())(name: '{}', id: {}, type: {})",
                data::get_element_name(rowid), rowid, type_num))
            },
            //
            TableRow::Asset(rowid, type_num) => {
                //
                Err(format!("Can't define an asset as an element (on ElementHandler::recycle())(name: '{}', id: {}, type: {})",
                data::get_asset_name(rowid), rowid, type_num))
            },
        }
    }
}

//
pub fn create_api(element_defs: &Rc<RefCell<HashMap<u32,Result<Rc<ElementDefinition>, String>>>>) -> Result<(Engine,
ElementHandler, ElementHandler, Rc<RefCell<Vec<ElementHandler>>>, Rc<RefCell<HashMap<String, KeyState>>>), String> {
    // Create an 'Engine'
    let mut engine = Engine::new_raw();

    // Register API features to the 'Engine'
    engine.on_print(|text| { log_1(&wasm_bindgen::JsValue::from_str(text)); })
          .register_type_with_name::<element::ElemPoint>("Point")
          .register_get_set("x", element::ElemPoint::get_x, element::ElemPoint::set_x)
          .register_get_set("y", element::ElemPoint::get_y, element::ElemPoint::set_y)
          .register_type_with_name::<element::ElemColor>("Color")
          .register_get_set("r", element::ElemColor::get_r, element::ElemColor::set_r)
          .register_get_set("g", element::ElemColor::get_g, element::ElemColor::set_g)
          .register_get_set("b", element::ElemColor::get_b, element::ElemColor::set_b)
          .register_get_set("a", element::ElemColor::get_a, element::ElemColor::set_a)
          .register_type_with_name::<asset::Sprite>("Sprite")
          .register_get("id", asset::Sprite::get_id_rhai)
          .register_get("cur_animation", asset::Sprite::get_cur_animation)
          .register_set("cur_animation", asset::Sprite::set_cur_animation)
          .register_get_set("cur_frame", asset::Sprite::get_cur_frame, asset::Sprite::set_cur_frame)
          .register_get("animation_time", asset::Sprite::get_animation_time)
          .register_get_set("repeat", asset::Sprite::get_repeat, asset::Sprite::set_repeat)
          .register_get("is_animation_finished", asset::Sprite::get_is_animation_finished)
          .register_fn("play_animation", asset::Sprite::play_animation)
          .register_fn("play_animation", asset::Sprite::play_animation_on_time)
          .register_type_with_name::<asset::AssetList<asset::Sprite>>("AssetList<Sprite>")
          .register_get("cur_asset", asset::AssetList::<asset::Sprite>::get_cur_asset)
          .register_set("cur_asset", asset::AssetList::<asset::Sprite>::set_cur_asset)
          .register_indexer_get(asset::AssetList::<asset::Sprite>::get_asset)
          .register_indexer_set(asset::AssetList::<asset::Sprite>::set_asset)
          .register_fn("len", asset::AssetList::<asset::Sprite>::len)
          .register_get("len", asset::AssetList::<asset::Sprite>::len)
          .register_fn("contains", asset::AssetList::<asset::Sprite>::contains)
          .register_type_with_name::<element::Object>("Object")
          .register_get_set("position", element::Object::get_position, element::Object::set_position)
          .register_get_set("scale", element::Object::get_scale, element::Object::set_scale)
          .register_get_set("color", element::Object::get_color, element::Object::set_color)
          .register_get_set("sprites", element::Object::get_sprites, element::Object::set_sprites)
          .register_get("index_in_stack", element::Object::get_index_in_stack)
          .register_type_with_name::<element::Camera>("Camera")
          .register_get_set("position", element::Camera::get_position, element::Camera::set_position)
          .register_get_set("zoom", element::Camera::get_zoom, element::Camera::set_zoom)
          .register_get_set("color", element::Camera::get_color, element::Camera::set_color)
          .register_type_with_name::<element::Layer>("Layer")
          .register_get("name", element::Layer::get_name)
          .register_get("instances", element::Layer::get_instances)
          .register_type_with_name::<element::Scene>("Scene")
          .register_get_set("camera", element::Scene::get_camera, element::Scene::set_camera)
          .register_get("objects_len", element::Scene::get_objects_len)
          .register_get("runtimes_len", element::Scene::get_runtimes_len)
          .register_get("runtime_vacants", element::Scene::get_runtime_vacants)
          .register_get("layers", element::Scene::get_layers)
          .register_fn("remove_instance", element::Scene::remove_instance)
          .register_fn("add_instance", element::Scene::add_instance)
          .register_type_with_name::<element::Game>("Game")
          .register_get_set("canvas_width", element::Game::get_canvas_width, element::Game::set_canvas_width)
          .register_get_set("canvas_height", element::Game::get_canvas_height, element::Game::set_canvas_height)
          .register_get_set("clear_red", element::Game::get_clear_red, element::Game::set_clear_red)
          .register_get_set("clear_green", element::Game::get_clear_green, element::Game::set_clear_green)
          .register_get_set("clear_blue", element::Game::get_clear_blue, element::Game::set_clear_blue)
          .register_get_set("fps", element::Game::get_fps, element::Game::set_fps)
          .register_get("cur_scene", element::Game::get_cur_scene)
          .register_set("cur scene", element::Game::set_cur_scene)
          .register_get("version", element::Game::get_version);

    // Register the standard packages
    let std_package = StandardPackage::new();
    // Load the standard packages into the 'Engine'
    std_package.register_into_engine(&mut engine);

    // Register a variable definition filter.
    engine.on_def_var(|is_runtime, info, _| {
        Ok((info.name != "Scene" && info.name != "Object" && info.name != "Game" && info.name != "State") || !is_runtime)
    });

    //
    element_defs.borrow_mut().insert(0,
        ElementDefinition::new(&engine,
        TableRow::Metadata
    ));

    // Create a new element instance for the state manager.
    // This instance will borrow its definition and contain
    // the element's 'Scope'.
    let state_manager = ElementHandler::new(
        element_defs.borrow().get(&0).unwrap().as_ref()?,
        None
    )?;

    // Receive the rowid of the initial scene from the the state manager's config
    let cur_scene_id = state_manager.properties.borrow()
    .read_lock::<element::Game>().expect("read_lock cast should succeed").cur_scene;
    //
    element_defs.borrow_mut().insert(cur_scene_id, 
        ElementDefinition::new(&engine,
        TableRow::Element(cur_scene_id, 2)
    ));
    //
    let cur_scene = ElementHandler::new(
        element_defs.borrow().get(&cur_scene_id).unwrap().as_ref()?,
        None
    )?;

    let state_table = Rc::new(RefCell::new(Dynamic::from_map(Map::default())));
    state_manager.resources.borrow_mut().scope
    .push_dynamic("State", Dynamic::from(Rc::clone(&state_table)));
    
    let api_scene_props = Rc::clone(&cur_scene.properties);
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
                    // of the value of the state table
                    Ok(Some(state_table.borrow().flatten_clone())) 
                }
            },
            "Scene" => {
                if context.scope().contains(name) {
                    // If the variable exists
                    // in the scope already
                    // (which means it's the
                    // current scene's scope)
                    Ok(None) 
                } else {
                    // Otherwise, return a clone
                    // of the value of the scene properties
                    Ok(Some(api_scene_props.borrow().flatten_clone()))
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
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("object_is_valid", move |idx: rhai::INT| -> bool {
        //
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        //
        idx < (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT && idx > -1
    });
    //
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("object_is_active", move |idx: rhai::INT| -> rhai::INT {
        //
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        //
        scene_props_borrow.layers[0..scene_props_borrow.layers_len]
        .iter().enumerate().flat_map(|(layer_idx, layer)| {
            layer.instances.iter().map(move |&index| {
                (index as rhai::INT, layer_idx as rhai::INT)
            })
        }).find(|&(index, _)| { index == idx }).unwrap_or((-1, -1)).1 as rhai::INT
    });

    //
    let state_manager_res = Rc::clone(&state_manager.resources);
    engine.register_fn("message_state_manager", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if let Ok(mut borrow) = state_manager_res.try_borrow_mut() {
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
    let cur_scene_res = Rc::clone(&cur_scene.resources);
    engine.register_fn("message_cur_scene", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if let Ok(mut borrow) = cur_scene_res.try_borrow_mut() {
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

    state_manager.resources.borrow_mut().reload(&engine)?;
    cur_scene.resources.borrow_mut().reload(&engine)?;

    //
    let object_stack: Rc<RefCell<Vec<ElementHandler>>> = Rc::new(RefCell::new(Vec::new()));

    //
    {
        //
        let instances = cur_scene.resources.borrow().definition
        .config["object-instances"].clone().into_typed_array::<Map>().expect(concat!("Every object's",
        " config should contain a 'object-instances' array, which should only have object-like members."));
        //
        let mut object_stack_borrow = object_stack.borrow_mut();
        //
        for (idx, map, rowid, layer) in instances
        .iter().enumerate().map(|(inst_index, inst)| {(
            //
            inst_index as u32, inst,
            dynamic_to_number(&inst["id"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an integer 'id' attribute.")) as u32,
            dynamic_to_number(&inst["layer"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an integer 'layer' attribute.")),
        )}) {
            //
            {
                //
                let mut scene_props_borrow = cur_scene.properties.borrow_mut();
                let mut scene_props_borrow = scene_props_borrow
                .write_lock::<element::Scene>().expect("write_lock cast should succeed");
                //
                scene_props_borrow.add_instance(idx as rhai::INT, layer as rhai::INT);
            } //
            //
            if !element_defs.borrow().contains_key(&rowid) {
                //
                element_defs.borrow_mut().insert(rowid,
                    ElementDefinition::new(&engine,
                    TableRow::Element(rowid, 1)
                ));
            }
            //
            object_stack_borrow.push(ElementHandler::new(
                element_defs.borrow().get(&rowid).unwrap().as_ref()?,
                Some(element::ObjectInitInfo::new(idx, map))
            )?);
            //
            object_stack_borrow.last().unwrap().resources.borrow_mut().reload(&engine)?;
        }
    }


    //
    let api_object_stack = Rc::clone(&object_stack);
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("get_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<element::Object, Box<EvalAltResult>> {
        //
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        //
        if idx >= (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(scene_props_borrow.objects_len+
            scene_props_borrow.runtimes_len, idx, context.position())));
        }
        //
        let object_stack_borrow;
        if let Ok(borrow) = api_object_stack.try_borrow() {
            object_stack_borrow = borrow;
        } else {
            return Err("Can't use the global function 'get_object' while the scene is being loaded".into());
        }
        //
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            //
            Ok(element.properties.borrow().clone_cast::<element::Object>())
        } else {
            //
            Err(Box::new(EvalAltResult::ErrorArrayBounds(scene_props_borrow.objects_len+
            scene_props_borrow.runtimes_len, idx, context.position())))
        }
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("message_object", move |context: rhai::NativeCallContext, idx: rhai::INT, 
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        let objects_len: usize;
        let runtimes_len: usize;
        //
        {
            //
            let scene_props_borrow = cur_scene_props.borrow();
            let scene_props_borrow = scene_props_borrow
            .read_lock::<element::Scene>().expect("read_lock cast should succeed");
            //
            objects_len = scene_props_borrow.objects_len;
            runtimes_len = scene_props_borrow.runtimes_len;
        }//

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
        let mut element_res_clone: Option<Rc<RefCell<ElementResources>>> = None;
        {
            //
            let object_stack_borrow;
            if let Ok(borrow) = api_object_stack.try_borrow() {
                object_stack_borrow = borrow;
            } else {
                return Err("Can't use the global function 'message_object' while the scene is being loaded".into());
            }
            //
            if let Some(element) = object_stack_borrow.get(idx as usize) {
                //
                element_res_clone = Some(Rc::clone(&element.resources));
            }
        }//

        //
        if let Some(element) = element_res_clone {
            //
            if let Ok(mut borrow) = element.try_borrow_mut() {
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
            Err(full_s.into())
        }
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    let api_element_defs = Rc::clone(element_defs);
    engine.register_fn("add_object_to_stack", move |context: rhai::NativeCallContext,
    id_source: rhai::INT, init_x: rhai::FLOAT, init_y: rhai::FLOAT| -> Result<rhai::INT, Box<EvalAltResult>> {
        //
        let mut scene_props_borrow = cur_scene_props.borrow_mut();
        let mut scene_props_borrow = scene_props_borrow
        .write_lock::<element::Scene>().expect("read_lock cast should succeed");
        //
        let mut object_stack_borrow;
        if let Ok(borrow) = api_object_stack.try_borrow_mut() {
            object_stack_borrow = borrow;
        } else {
            return Err("Can't use the global function 'add_object_to_stack' while the scene is being loaded".into());
        }
        //
        let def_rc_clone: Rc<ElementDefinition>;
        //
        if let Some(element_def) = api_element_defs.borrow().get(&(id_source as u32)) {
            //
            def_rc_clone = Rc::clone(element_def.as_ref()?);
            //
            match def_rc_clone.row {
                TableRow::Metadata => {
                    return Err("Tried to use 'add_object_to_stack' with the state manager's definition.".into())
                },
                TableRow::Asset(rowid, type_num) => {
                    return Err(format!("Tried to use 'add_object_to_stack' with a definition of an asset (name: '{}', id: {}, type: {})",
                    data::get_asset_name(rowid), rowid, type_num).into())
                },
                TableRow::Element(rowid, 2) => {
                    return Err(format!("Tried to use 'add_object_to_stack' with a definition of a scene (name: '{}', id: {})",
                    data::get_asset_name(rowid), rowid).into())
                },
                _ => ()
            }
        } else {
            //
            return Err("Tried to use 'add_object_to_stack' with a definition which doesn't exist.".into());
        }
        //
        if let Some(&vacant_index) = scene_props_borrow.runtime_vacants.iter().min() {
            //
            let object = object_stack_borrow
            .get_mut(vacant_index as usize).expect("Shouldn't be out of bounds.");
            //
            if let Err(err) = object.recycle(context.engine(),
            &def_rc_clone, Some(element::ObjectInitInfo {
                idx_in_stack: (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                init_x, init_y, init_scale_x: 1_f32, init_scale_y: 1_f32,
                init_color: String::from("#FFFFFF"), init_alpha: 255_u8
            })) {
                //
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            //
            return Ok(vacant_index as rhai::INT);
        }
        //
        if let Some(object_ref) = object_stack_borrow
        .get_mut(scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) {
            //
            if let Err(err) = object_ref.recycle(context.engine(),
            &def_rc_clone,Some(element::ObjectInitInfo {
                idx_in_stack: (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                init_x, init_y, init_scale_x: 1_f32, init_scale_y: 1_f32,
                init_color: String::from("#FFFFFF"), init_alpha: 255_u8
            })) {
                //
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            //
            let index = (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32;
            scene_props_borrow.runtimes_len += 1;
            scene_props_borrow.runtime_vacants.push(index);   
            //
            return Ok(index as rhai::INT);
        }
        //
        object_stack_borrow.push(
            {
                //
                let element = ElementHandler::new(
                &def_rc_clone, Some(element::ObjectInitInfo {
                    idx_in_stack: (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                    init_x, init_y, init_scale_x: 1_f32, init_scale_y: 1_f32,
                    init_color: String::from("#FFFFFF"), init_alpha: 255_u8
                }));
                //
                if element.is_err() {
                    //
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", element.err().unwrap()).into());
                }
                //
                let element = element.unwrap();
                if let Some(err_string) = element.resources.borrow_mut().reload(context.engine()).err() {
                    //
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err_string).into());
                }
                //
                element
            }
        );
        //
        let index = (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32;
        scene_props_borrow.runtimes_len += 1;
        scene_props_borrow.runtime_vacants.push(index);
        //
        Ok(index as rhai::INT)
    });

    // The API is done!
    Ok((engine, state_manager, cur_scene, object_stack, key_states))
}