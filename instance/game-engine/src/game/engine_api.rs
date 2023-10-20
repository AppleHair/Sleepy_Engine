
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
    fn new(engine: &Engine, mut scope: Scope<'static>, definition: Rc<ElementDefinition>) -> Result<Self, String> {
        //
        if let Some(err) = engine.run_ast_with_scope
        (&mut scope, &definition.script).err() {
            //
            return Err(definition.row.to_err_string(&err.to_string()));
        }
        //
        Ok(Self { definition, scope })
    }
    //
    fn recycle(&mut self, engine: &Engine, definition: Rc<ElementDefinition>) -> Result<(), String> {
        //
        self.definition = definition;
        self.scope.clear();
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
    pub fn new(engine: &Engine, def: &Rc<ElementDefinition>,
    object_info: Option<(u32, f32, f32)>) -> Result<Self, String> {
        //
        let mut element_handler = Self {
            properties: Default::default(),
            resources: Rc::new(RefCell::new(
                ElementResources::new(&engine, 
                Scope::new(),Rc::clone(def))?
            ))
        };
        //
        let row_copy = element_handler.resources.borrow().definition.row;
        //
        match row_copy {
            //
            TableRow::Metadata => {
                //
                let shared_map = Rc::new(RefCell::new(Dynamic::from_map(Map::default())));
                //
                if let Some(map) = element_handler.resources
                .borrow_mut().scope.remove::<Map>("State") {
                    let _ = shared_map.replace(Dynamic::from_map(map));
                }
                //
                element_handler.resources.borrow_mut().scope
                .push_dynamic("State", Dynamic::from(Rc::clone(&shared_map)));
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
                        Dynamic::from(element::Object::new(&element_handler.resources.borrow().definition.config,
                            info.0, info.1,
                            info.2))
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
    object_info: Option<(f32, f32)>) -> Result<(), String> {
        //
        self.resources.borrow_mut().recycle(&engine, Rc::clone(def))?;
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
                Ok(())
            },
            //
            TableRow::Element(rowid, 1) => {
                //
                if let Some(info) = object_info {
                    //
                    self.properties.borrow_mut().write_lock::<element::Object>()
                    .expect("write_lock cast should succeed")
                    .recycle(&self.resources.borrow().definition.config, info.0, info.1);
                    //
                    self.resources.borrow_mut().scope
                    .push_dynamic("Object", Dynamic::from(Rc::clone(&self.properties)));
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
pub fn create_api(element_defs: &mut HashMap<u32,Result<Rc<ElementDefinition>, String>>) -> Result<(Engine, ElementHandler, ElementHandler,
Rc<RefCell<u32>>, Rc<RefCell<u32>>, Rc<RefCell<Vec<ElementHandler>>>, Rc<RefCell<HashMap<String, KeyState>>>), String> {
    // Create an 'Engine'
    let mut engine = Engine::new_raw();

    // Register API features to the 'Engine'
    engine.on_print(|text| { log_1(&wasm_bindgen::JsValue::from_str(text)); })
          .register_type_with_name::<element::ElemPoint>("Position")
          .register_get_set("x", element::ElemPoint::get_x, element::ElemPoint::set_x)
          .register_get_set("y", element::ElemPoint::get_y, element::ElemPoint::set_y)
          .register_fn("to_string", element::ElemPoint::to_string)
          .register_type_with_name::<element::CollisionBox>("CollisionBox")
          .register_get("point1", element::CollisionBox::get_point1)
          .register_get("point2", element::CollisionBox::get_point2)
          .register_fn("to_string", element::CollisionBox::to_string)
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
          .register_fn("to_string", asset::Sprite::to_string)
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
          .register_get_set("sprites", element::Object::get_sprites, element::Object::set_sprites)
          .register_get("collision_boxes", element::Object::get_collision_boxes)
          .register_get("index_in_stack", element::Object::get_index_in_stack)
          .register_fn("to_string", element::Object::to_string)
          .register_type_with_name::<element::Camera>("Camera")
          .register_get_set("position", element::Camera::get_position, element::Camera::set_position)
          .register_get_set("zoom", element::Camera::get_zoom, element::Camera::set_zoom)
          .register_fn("to_string", element::Camera::to_string)
          .register_type_with_name::<element::Layer>("Layer")
          .register_get("name", element::Layer::get_name)
          .register_get("instances", element::Layer::get_instances)
          .register_fn("to_string", element::Layer::to_string)
          .register_type_with_name::<element::Scene>("Scene")
          .register_get_set("width", element::Scene::get_width, element::Scene::set_width)
          .register_get_set("height", element::Scene::get_height, element::Scene::set_height)
          .register_get("in_color", element::Scene::get_inside_color)
          .register_set("in_color", element::Scene::set_inside_color)
          .register_get("out_color", element::Scene::get_outside_color)
          .register_set("out_color", element::Scene::set_outside_color)
          .register_get_set("camera", element::Scene::get_camera, element::Scene::set_camera)
          .register_get("objects_len", element::Scene::get_objects_len)
          .register_get("runtimes_len", element::Scene::get_runtimes_len)
          .register_get("runtime_vacants", element::Scene::get_runtime_vacants)
          .register_get("layers", element::Scene::get_layers)
          .register_fn("remove_instance", element::Scene::remove_instance)
          .register_fn("add_instance", element::Scene::add_instance)
          .register_fn("to_string", element::Scene::to_string);

    // Register the standard packages
    let std_package = StandardPackage::new();
    // Load the standard packages into the 'Engine'
    std_package.register_into_engine(&mut engine);

    //
    element_defs.insert(0,
        ElementDefinition::new(&engine,
        TableRow::Metadata
    ));

    // Create a new element instance for the state manager.
    // This instance will borrow its definition and contain
    // the element's 'Scope'.
    let state_manager = ElementHandler::new(&engine,
        element_defs.get(&0).unwrap().as_ref()?,
        None
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
        dynamic_to_number(&element_defs.get(&0).unwrap().as_ref()?.config["initial-scene"])
        .expect("The value of 'initial-scene' in the state manager's config should be an integer") as u32
    ));
    //
    let prv_scene_id = Rc::new(RefCell::new(0_u32));
    //
    element_defs.insert(cur_scene_id.borrow().clone(), 
        ElementDefinition::new(&engine,
        TableRow::Element(cur_scene_id.borrow().clone(), 2)
    ));
    //
    let cur_scene = ElementHandler::new(&engine,
        element_defs.get(&cur_scene_id.borrow()).unwrap().as_ref()?,
        None
    )?;
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
        for (idx, init_x, init_y, rowid) in instances.iter().enumerate()
        .map(|(map_index, map)| {(
            //
            map_index as u32,
            //
            dynamic_to_number(&map["x"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an float 'x' attribute.")),
            //
            dynamic_to_number(&map["y"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an float 'y' attribute.")),
            //
            dynamic_to_number(&map["id"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an integer 'id' attribute.")) as u32,
        )}) {
            //
            if !element_defs.contains_key(&rowid) {
                //
                element_defs.insert(rowid,
                    ElementDefinition::new(&engine,
                    TableRow::Element(rowid, 1)
                ));
            }
            //
            object_stack_borrow.push(ElementHandler::new(&engine,
                element_defs.get(&rowid).unwrap().as_ref()?,
                Some((idx, init_x, init_y))
            )?);
        }
    }

    let api_state_map = Rc::clone(&state_manager.properties);
    let api_scene_map = Rc::clone(&cur_scene.properties);
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
    let cur_scene_res = Rc::clone(&cur_scene.resources);
    engine.register_fn("is_cur_scene", move |name: &str| -> Result<bool, Box<EvalAltResult>> {
        //
        if let Ok(borrow) = cur_scene_res.try_borrow() {
            //
            Ok(if let TableRow::Element(id, 2) = borrow.definition.row {
                //
                data::get_element_name(id) == name
            } else { false })
        } else {
            //
            Err(concat!("Can't use the 'is_cur_scene' function inside a scene",
            " script! Note: A scene script will only run when its scene will be",
            " the current scene, so you don't need to use this function inside a scene script.").into())
        }
    });
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
        let object_stack_borrow = api_object_stack.borrow();
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
            let object_stack_borrow = api_object_stack.borrow();
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
    engine.register_fn("add_object_to_stack", move |context: rhai::NativeCallContext,
    idx_source: rhai::INT, init_x: rhai::FLOAT, init_y: rhai::FLOAT| -> Result<rhai::INT, Box<EvalAltResult>> {
        //
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        //
        if idx_source >= (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT {
            //
            let info = "Argument 'idx_source' was out of bounds in call to 'add_object_to_stack'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx_source, scene_props_borrow.objects_len+scene_props_borrow.runtimes_len);
            //
            return Err(full_s.into());
        }
        //
        let mut object_stack_borrow = api_object_stack.borrow_mut();
        //
        let def_rc_clone: Rc<ElementDefinition>;
        //
        if let Some(element) = object_stack_borrow.get(idx_source as usize) {
            //
            if let Ok(borrow) = element.resources.try_borrow() {
                //
                def_rc_clone = Rc::clone(&borrow.definition);
            } else {
                //
                return Err(concat!("Can't create a new object using a reference to an object",
                " whose script is running (is handling another callback). Note: It's recommended",
                " to not add objects which are used for making other objects to a layer, and not use",
                " 'message_' callbacks with this function, which might need to refer to objects",
                " involved in the current callback to create new objects.").into());
            }
        } else {
            //
            let info = "Argument 'idx_source' was out of bounds in call to 'add_object_to_stack'.";
            //
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx_source, scene_props_borrow.objects_len+scene_props_borrow.runtimes_len);
            //
            return Err(full_s.into());
        }
        //
        if let Some(&vacant_index) = scene_props_borrow.runtime_vacants.iter().min() {
            //
            let object = object_stack_borrow
            .get_mut(vacant_index as usize).expect("Shouldn't be out of bounds.");
            //
            if let Err(err) = object.recycle(context.engine(),
            &def_rc_clone, Some((init_x, init_y))) {
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
            &def_rc_clone,Some((init_x, init_y))) {
                //
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            //
            return Ok((scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT);
        }
        //
        object_stack_borrow.push(
            {
                //
                let element = ElementHandler::new(context.engine(),
                &def_rc_clone, Some(((scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                init_x as f32, init_y as f32)));
                //
                if element.is_err() {
                    //
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", element.err().unwrap()).into());
                }
                //
                element.unwrap()
            }
        );
        //
        Ok((scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT)
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