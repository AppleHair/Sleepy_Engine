
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use web_sys::console::log_1;
use rhai::{Engine, Scope, AST, Map, EvalAltResult, Dynamic,
    packages::{Package, StandardPackage}};

use crate::{data, game::{TableRow, dynamic_to_number}};

pub mod element;
pub mod asset;

pub type ElementDefinitions = HashMap<u32,Result<Rc<ElementDefinition>, String>>;
pub type KeyStates = HashMap<String, KeyState>;

/// A struct that will be
/// used to track the state\
/// of a key on the keyboard
/// in the key states table.
pub struct KeyState {
    pub is_held: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

/// A struct that will be used to
/// store all the data which is loaded\
/// for a single element defined in the
/// project file/game data file.
pub struct ElementDefinition {
    pub config: Map,
    pub script: AST,
    pub row: TableRow,
}

impl ElementDefinition {
    /// Using a rhai engine and row data,
    /// this function will load the element's\
    /// configuration and script and return
    /// a new element definition, or an error. 
    pub fn new(engine: &Engine, row: TableRow) -> Result<Rc<Self>, String> {
        // Load the element's script and compile
        // it into an AST (Abstract Syntax Tree).
        let ast = engine.compile(&match row {
            // The metadata script is the state manager's script.
            TableRow::Metadata => data::get_metadata_script(),
            // An element script is a script associated with
            // a specific element for the 'element' table.
            TableRow::Element(rowid, _) => data::get_element_script(rowid),
            // An asset doesn't have a script, because it's
            // not an element and it can't be used as one.
            TableRow::Asset(rowid, type_num) => { return Err(
            format!("Can't define an asset as an element (on ElementDefinition::new())(name: '{}', id: {}, type: {})",
            data::get_asset_name(rowid), rowid, type_num)); },
        });
        // Return a parse error if any occured
        // while compiling the script.
        if let Some(err) = ast.as_ref().err() {
            return Err(row.to_err_string(&err.to_string()));
        }
        // Load the element's configuration and
        // parse it into a rhai map (JSON object).
        let json = engine.parse_json(&match row {
            // The metadata config is the state manager's config.
            TableRow::Metadata => data::get_metadata_config(),
            // An element config is a config associated with
            // a specific element for the 'element' table.
            TableRow::Element(rowid, _) => data::get_element_config(rowid),
            // An asset does have a config, but it
            // still can't be used as an element.
            TableRow::Asset(rowid, type_num) => { return Err(
            format!("Can't define an asset as an element (on ElementDefinition::new())(name: '{}', id: {}, type: {})",
            data::get_asset_name(rowid), rowid, type_num)); },
        }, false);
        // Return an error if any occured
        // while parsing the config.
        if let Some(err) = json.as_ref().err() {
            return Err(row.to_err_string(&err.to_string()));
        }
        // Return a new element definition
        // wrapped in a counted reference
        // (if no error occured while loading
        // and parsing all this data, the
        // counted reference will be shared 
        // with element handlers, which will
        // use it to access the element's
        // configuration and script).
        Ok(Rc::new(Self {
            config: json.unwrap(), 
            script: ast.unwrap(),
            row,
        }))
    }
}

/// A struct that will store the
/// element's "resources", which\
/// include the element's definition,
/// and the scope of the\
/// element's script.
/// 
/// The main purpose of this
/// struct is to maintain the state\
/// of an element's script between
/// callbacks using the rhai scope,\
/// and keeping a counted reference
/// to the element's definition,\
/// which is needed for running the
/// element's script with the\
/// maintained scope among other things.
pub struct ElementResources {
    pub definition: Rc<ElementDefinition>,
    scope: Scope<'static>
}

impl ElementResources {
    /// Creates a new element resources\
    /// struct using a given element definition.
    fn new(definition: Rc<ElementDefinition>) -> Self {
        Self { definition, scope: Scope::new() }
    }
    /// Recycles an existing element resources\
    /// struct using a given element definition.
    fn recycle(&mut self, definition: Rc<ElementDefinition>) {
        self.definition = definition;
        self.scope.clear();
    }
    /// Runs the element's script
    /// with the current scope and\
    /// return an error if any occured.
    /// 
    /// This means the code written in 
    /// the global scope will be executed\
    /// and every variable defined in it will
    /// stay in the scope until it's cleared.
    pub fn run_script(&mut self, engine: &Engine) -> Result<(), String> {
        if let Some(err) = engine.run_ast_with_scope
        (&mut self.scope, &self.definition.script).err() {
            return Err(self.definition.row.to_err_string(&err.to_string()));
        }
        Ok(())
    }
    /// Runs a function defined in the
    /// element's script with the maintained
    /// scope and return an error if any occured.
    /// 
    /// If the function doesn't exist, the call
    /// will be ignored and no error will be raised.\
    /// Any new variable defined in the function
    /// will be cleared from the scope after the
    /// function returns.
    pub fn call_fn(&mut self, engine: &Engine, name: &str, args: impl rhai::FuncArgs) -> Result<(), String> {
        if !self.definition.script.iter_functions()
        .any(|func| { func.name == name}) {
            return Ok(());
        }
        if let Some(err) = engine.call_fn_with_options::<()>
        (rhai::CallFnOptions::new().eval_ast(false), &mut self.scope,
        &self.definition.script, name, args).err() {
            return Err(self.definition.row.to_err_string(&err.to_string()));
        }
        Ok(())
    }
}

/// A struct that will be used to
/// handle an instance of an element.
/// 
/// This struct will store the element's
/// properties and an element resources\
/// struct. The element's properties
/// will be shared with the scope of\
/// the element's script, which will
/// allow the script to access it's
/// local API.
pub struct ElementHandler {
    // Each attribute of this struct
    // is wrapped in an counted reference
    // (interior-mutated), which will allow
    // the rhai engine to access and modify
    // them in order to implement several
    // API features.
    pub properties: Rc<RefCell<Dynamic>>,
    pub resources: Rc<RefCell<ElementResources>>,
}

impl ElementHandler {
    /// Creates a new element handler.
    /// 
    /// The element resources will be
    /// created first using the given
    /// element definition, and then
    /// the element definition will be\
    /// used to create the element's
    /// properties, which will be shared
    /// with the element's script scope.
    pub fn new(def: &Rc<ElementDefinition>,
    object_info: Option<element::ObjectInitInfo>) -> Result<Self, String> {
        // The element handler first gets
        // created only with the element's
        // resources, and with the properties
        // set to defalt values.
        let mut element_handler = Self {
            properties: Default::default(),
            resources: Rc::new(RefCell::new(
                ElementResources::new(Rc::clone(def))
            ))
        };
        // Then, the element's properties
        // get created using the element's
        // definition. Different properties
        // will be created for different
        // types of elements.
        let row_copy = element_handler.resources.borrow().definition.row;
        match row_copy {
            // "Metadata" means we are creating
            // the state manager's element handler.
            TableRow::Metadata => {
                // The state manager's properties
                // use the "Game" API, which is
                // a local API defined in the
                // "element" module.
                let shared_map = Rc::new(RefCell::new(
                    Dynamic::from(element::Game::new(&element_handler.resources.borrow().definition.config))
                ));
                // Share a counted reference
                // (interior-mutated) to the
                // "Game" API with the state
                // manager's script scope and
                // with the state manager's
                // element handler.
                element_handler.resources.borrow_mut().scope
                .push_dynamic("Game", Dynamic::from(Rc::clone(&shared_map)));
                element_handler.properties = shared_map;
                // Return the element handler.
                Ok(element_handler)
            },
            // "Element" with a type 2
            // means we are creating a
            // scene's element handler.
            TableRow::Element(_, 2) => {
                // Scenes' properties use the
                // "Scene" API, which is a local API
                // defined in the "element" module.
                let shared_map = Rc::new(RefCell::new(
                    Dynamic::from(element::Scene::new(&element_handler.resources.borrow().definition.config))
                ));
                // Share a counted reference
                // (interior-mutated) to the
                // "Scene" API with the scene's 
                // script scope and with the
                // scene's element handler.
                element_handler.resources.borrow_mut().scope
                .push_dynamic("Scene", Dynamic::from(Rc::clone(&shared_map)));
                element_handler.properties = shared_map;
                // Return the element handler.
                Ok(element_handler)
            },
            // "Element" with a type 1
            // means we are creating an
            // object's element handler.
            TableRow::Element(rowid, 1) => {
                // Check if an object init info was provided
                if let Some(info) = object_info {
                    // Objects' properties use the
                    // "Object" API, which is a local API
                    // defined in the "element" module
                    // and it requires an object init info
                    // to be created.
                    let shared_map = Rc::new(RefCell::new(
                        Dynamic::from(element::Object::new(&element_handler.resources.borrow().definition.config,info))
                    ));
                    // Share a counted reference
                    // (interior-mutated) to the
                    // "Object" API with the object's
                    // script scope and with the
                    // object's element handler.
                    element_handler.resources.borrow_mut().scope
                    .push_dynamic("Object", Dynamic::from(Rc::clone(&shared_map)));
                    element_handler.properties = shared_map;
                    // Return the element handler.
                    Ok(element_handler)
                } else {
                    // If no object init info was
                    // provided, return an error.
                    Err(format!("Tried to create object handler without 'object_info' parameter (name: '{}', id: {})", 
                    data::get_element_name(rowid), rowid))
                }
            },
            TableRow::Element(rowid, type_num) => {
                // If the element's type isn't
                // 1 or 2, return an error.
                Err(format!("In-valid element (on ElementHandler::new())(name: '{}', id: {}, type: {})",
                data::get_element_name(rowid), rowid, type_num))
            },
            TableRow::Asset(rowid, type_num) => {
                // If an asset was provided,
                // return an error.
                Err(format!("Can't define an asset as an element (on ElementHandler::new())(name: '{}', id: {}, type: {})",
                data::get_asset_name(rowid), rowid, type_num))
            },
        }
    }
    /// Recycles an existing handler.
    /// 
    /// The element resources will be
    /// recycled first using the given
    /// element definition, and then
    /// the element definition will be\
    /// used to recycle the element's
    /// properties, which will be shared
    /// with the element's script scope.
    pub fn recycle(&self, def: &Rc<ElementDefinition>,
    object_info: Option<element::ObjectInitInfo>) -> Result<(), String> {
        // If the element handler is the
        // state manager, return an error.
        if let TableRow::Metadata = self.resources.borrow().definition.row {
            return Err("Tried to recycle State Manager.".into());
        }
        // The element's resources
        // get recycled first.
        self.resources.borrow_mut().recycle(Rc::clone(def));
        // Then, the element's properties
        // get recycled using the element's
        // definition. Different types of
        // elements will have different
        // types of properties to recycle.
        let row_copy = self.resources.borrow().definition.row;
        match row_copy {
            // "Element" with a type 2
            // means we are recycling a
            // scene's element handler.
            TableRow::Element(_, 2) => {
                // Scenes' properties use the
                // "Scene" API, which is a local API
                // defined in the "element" module.
                self.properties.borrow_mut().write_lock::<element::Scene>()
                .expect("write_lock cast should succeed")
                .recycle(&self.resources.borrow().definition.config);
                // Share a counted reference
                // (interior-mutated) to the
                // "Scne" API with the
                // scene's script scope
                self.resources.borrow_mut().scope
                .push_dynamic("Scene", Dynamic::from(Rc::clone(&self.properties)));

                Ok(())
            },
            // "Element" with a type 1
            // means we are recycling an
            // object's element handler.
            TableRow::Element(rowid, 1) => {
                // Check if an object init info was provided
                if let Some(info) = object_info {
                    // Objects' properties use the
                    // "Object" API, which is a local API
                    // defined in the "element" module
                    // and it requires an object init info
                    // to be recycled.
                    self.properties.borrow_mut().write_lock::<element::Object>()
                    .expect("write_lock cast should succeed")
                    .recycle(&self.resources.borrow().definition.config, info);
                    // Share a counted reference
                    // (interior-mutated) to the
                    // "Object" API with the
                    // object's script scope
                    self.resources.borrow_mut().scope
                    .push_dynamic("Object", Dynamic::from(Rc::clone(&self.properties)));
                    
                    Ok(())
                } else {
                    // If no object init info was
                    // provided, return an error.
                    Err(format!("Tried to recycle object handler without 'object_info' parameter (name: '{}', id: {})", 
                    data::get_element_name(rowid), rowid))
                }
            },
            TableRow:: Metadata => {
                // If the state manager definition
                // was provided, return an error.
                Err("Tried to recycle into State Manager.".into())
            },
            TableRow::Element(rowid, type_num) => {
                // If the element's type isn't
                // 1 or 2, return an error.
                Err(format!("In-valid element (on ElementHandler::recycle())(name: '{}', id: {}, type: {})",
                data::get_element_name(rowid), rowid, type_num))
            },
            TableRow::Asset(rowid, type_num) => {
                // If an asset was provided,
                // return an error.
                Err(format!("Can't define an asset as an element (on ElementHandler::recycle())(name: '{}', id: {}, type: {})",
                data::get_asset_name(rowid), rowid, type_num))
            },
        }
    }
}

/// Creates the API for the game engine,
/// and returns it's integrated components.
/// 
/// # Global APIs
/// 
/// global APIs are available to all
/// scripts, but not all of them can
/// be accessed from the global scope
/// or from non-API related functions.
/// While most of the global APIs are
/// available in the global scope, some
/// of them are only available in the
/// scope of a callback, like the 'update'
/// callback. This is because some of the
/// global APIs are related to the current
/// scene's objects, which need to be created
/// after the scene and the state manager's
/// scripts run, and before the 'init' callback,
/// which is called after all the object are created.
/// 
/// # Per-element Local APIs
/// 
/// per-element APIs will only be
/// available in the global scope
/// of the element's script, and in
/// any callback scope that is related
/// to the API (like the 'update' callback).
/// However, if you need to use your local
/// API in a non-API related function, which
/// is declared in the same script, you can
/// use the ! syntax to give the function
/// access to the caller's scope, which will
/// let you use the local API in the function
/// (also applys to other local variables).
/// 
/// for more information: https://rhai.rs/book/language/fn-parent-scope.html
/// 
/// ## Example
/// 
/// ```rhai
/// fn example() {
///     print(Object.position.x);
/// }
/// // example(); // will raise an error: "Object is not defined"
/// example!(); // will print the x position of the object using the local API
/// ```
/// 
/// The use of the ! syntax is not
/// recommended by the rhai book,
/// so you should use it with caution,
/// or try to find a better solution.
/// 
/// ## Example
/// 
/// ```rhai
/// fn better_example(obj) {
///     print(obj.position.x);
///     // do something with the clone of the object...
/// }
/// better_example(Object);
/// 
/// fn best_example(x) {
///     print(x);
///     // do something with the x position of the object...
/// }
/// best_example(Object.position.x);
/// ```
pub fn create_api(element_defs: &Rc<RefCell<ElementDefinitions>>) -> Result<(Engine,
ElementHandler, ElementHandler, Rc<RefCell<Vec<ElementHandler>>>, Rc<RefCell<KeyStates>>), String> {
    // Create a rhai engine, into which all
    // the API features will be registered.
    let mut engine = Engine::new_raw();

    // Register API types to the rhai
    // engine, which will mainly be
    // used for per-element local APIs.
    engine.register_type_with_name::<element::ElemPoint>("Point")
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
          .register_fn("contains", asset::AssetList::<asset::Sprite>::find)
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
          .register_set("cur_scene", element::Game::set_cur_scene)
          .register_get("version", element::Game::get_version);

    // Register a variable definition filter.
    // This will prevent scripts from shadowing
    // their own APIs by accident, and will raise
    // an error whenever they do.
    engine.on_def_var(|is_runtime, info, _| {
        Ok((info.name != "Scene" && info.name != "Object" && info.name != "Game" && info.name != "State") || !is_runtime)
    });

    // Load the state manager's definition,
    // which includes his configuration and script
    element_defs.borrow_mut().insert(0,
        ElementDefinition::new(&engine,
        TableRow::Metadata
    ));

    // Create a new element handler for the state manager.
    // This handler will take a counted reference to this
    // element's definition as long as no error occured
    // while loading the definition. If an error did occur,
    // it will be propagated back to the caller.
    let state_manager = ElementHandler::new(
        element_defs.borrow().get(&0).unwrap().as_ref()?,
        None
    )?;

    // Receive the rowid of the initial scene from the the state manager.
    let cur_scene_id = state_manager.properties.borrow()
    .read_lock::<element::Game>().expect("read_lock cast should succeed").cur_scene;
    // Load the initial scene's definition.
    element_defs.borrow_mut().insert(cur_scene_id, 
        ElementDefinition::new(&engine,
        TableRow::Element(cur_scene_id, 2)
    ));
    // Create a new element handler for the current
    // scene, or return an error if the definition
    // couldn't be loaded.
    let cur_scene = ElementHandler::new(
        element_defs.borrow().get(&cur_scene_id).unwrap().as_ref()?,
        None
    )?;
    
    // The following lines declare global
    // API functions, which will always be
    // available for scripts, even from
    // non-API related functions.

    // Register the standard packages,
    // which contain many common librarys
    // used by a programming language, like
    // math functions, casting functions,
    // string formating capabilitys and more.
    let std_package = StandardPackage::new();
    // Load the standard packages into the rhai engine
    std_package.register_into_engine(&mut engine);
    
    // Register a print function to the rhai engine,
    // which will be used by the scripts to print text
    // to the browser's console.
    engine.on_print(|text| { log_1(&wasm_bindgen::JsValue::from_str(text)); });
    // Create the state table, and share
    // a counted reference (interior-mutated)
    // to it with the state manager's script.
    // This table will be used to store
    // global variables, which will be
    // used for different purposes by
    // the game's scripts. Only the state
    // manager's script has write access
    // to this table, while all other scripts
    // only have read access to it.
    let state_table = Rc::new(RefCell::new(Dynamic::from_map(Map::default())));
    // Share a counted reference (interior-mutated)
    // to the state table with the state manager's
    // script. Because the state table counts as a
    // per-element API, it won't be naturally
    // available for write access in non-API
    // related functions, like any other
    // per-element API.
    state_manager.resources.borrow_mut().scope
    .push_dynamic("State", Dynamic::from(Rc::clone(&state_table)));
    // Share a counted reference to the
    // properties of the current scene,
    // for use in the variable resolver.
    let api_scene_props = Rc::clone(&cur_scene.properties);

    // Register a variable resolver.
    // This will allow the scripts to
    // read the state table and the
    // current scene's properties.
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

    // Create the key states table.
    // This table will be used to track
    // the key state of every key on the
    // keyboard, and will be used by the
    // scripts to check if a key is held,
    // pressed or released.
    let key_states: Rc<RefCell<KeyStates>> = Rc::new(RefCell::new(HashMap::new()));

    // Share a counted reference to the key
    // states table with the following API function.
    let api_key_states = Rc::clone(&key_states);
    engine.register_fn("key_is_held", move |key: &str| -> bool {
        // if the key exists in the table,
        if let Some(state) = api_key_states.borrow().get(key) {
            // return it's 'is_held' state
            state.is_held
        } else {
            // otherwise, return false
            false
        }
    });

    let api_key_states = Rc::clone(&key_states);
    engine.register_fn("key_just_pressed", move |key: &str| -> bool {
        // if the key exists in the table,
        if let Some(state) = api_key_states.borrow().get(key) {
            // return it's 'just_pressed' state
            state.just_pressed
        } else {
            // otherwise, return false
            false
        }
    });

    let api_key_states = Rc::clone(&key_states);
    engine.register_fn("key_just_released", move |key: &str| -> bool {
        // if the key exists in the table,
        if let Some(state) = api_key_states.borrow().get(key) {
            // return it's 'just_released' state
            state.just_released
        } else {
            // otherwise, return false
            false
        }
    });

    // Share a counted reference to the
    // current scene's properties, for
    // use in the following API function.
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("object_is_valid", move |idx: rhai::INT| -> bool {
        // borrow the current scene's properties for reading (immutable)
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        // return true if the index is in the range of the current scene's objects
        idx < (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT && idx > -1
    });

    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("object_is_active", move |idx: rhai::INT| -> rhai::INT {
        // borrow the current scene's properties for reading (immutable)
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        // if the index included in any layer in the current scene,
        // return the index of the layer it's included in, otherwise return -1
        scene_props_borrow.layers[0..scene_props_borrow.layers_len]
        .iter().enumerate().flat_map(|(layer_idx, layer)| {
            layer.instances.iter().map(move |&index| {
                (index as rhai::INT, layer_idx as rhai::INT)
            })
        }).find(|&(index, _)| { index == idx }).unwrap_or((-1, -1)).1 as rhai::INT
    });

    // Share a counted reference to 
    // the state manager's resources, 
    // for use in the following API function.
    let state_manager_res = Rc::clone(&state_manager.resources);
    engine.register_fn("message_state_manager", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        // if the state manager's resources are not borrowed,
        if let Ok(mut borrow) = state_manager_res.try_borrow_mut() {
            // call the function with the given name and
            // arguments, and if an error is raised, return it.
            if let Some(err) = borrow.call_fn(context.engine(),&format!("message_{}", name), args).err() {
                // Mention the use of the messaging API in the error message
                Err(format!("{}\nas a result of a call to 'message_state_manager'", err).into())
            } else { Ok(()) }
        } else {
            // otherwise, return an error
            Err(concat!("Can't use the 'message_state_manager' function while the state manager's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an element, which tried to message you back in the scope of that same message.").into())
        }
    });

    // Share a counted reference to 
    // the current scene's resources, 
    // for use in the following API function.
    let cur_scene_res = Rc::clone(&cur_scene.resources);
    engine.register_fn("message_cur_scene", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        // if the current scene's resources are not borrowed,
        if let Ok(mut borrow) = cur_scene_res.try_borrow_mut() {
            // call the function with the given name and
            // arguments, and if an error is raised, return it.
            if let Some(err) = borrow.call_fn(context.engine(),&format!("message_{}", name), args).err() {
                // Mention the use of the messaging API in the error message
                Err(format!("{}\nas a result of a call to 'message_cur_scene'", err).into())
            } else { Ok(()) }
        } else {
            // otherwise, return an error
            Err(concat!("Can't use the 'message_cur_scene' function while the current scene's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an element, which tried to message you back in the scope of that same message.").into())
        }
    });

    // Converts an element's name to it's id.
    // Returns an error if the name doesn't exist.
    engine.register_fn("element_name_to_id", |name: &str| -> Result<rhai::INT, Box<EvalAltResult>> {
        let res = data::get_element_id(name) as rhai::INT;
        if res == 0 {
            Err(format!("Tried to use 'element_name_to_id' with an element name that doesn't exist ('{}').", name).into())
        } else { Ok(res) }
    });

    // Converts an element's id to it's name.
    // Returns an error if the id doesn't exist.
    engine.register_fn("element_id_to_name", |id: rhai::INT| -> Result<String, Box<EvalAltResult>> {
        let res = data::get_element_name(id as u32);
        if res.is_empty() {
            Err(format!("Tried to use 'element_id_to_name' with an element id that doesn't exist ('{}').", id).into())
        } else { Ok(res) }
    });

    // Converts an asset's name to it's id.
    // Returns an error if the name doesn't exist.
    engine.register_fn("asset_name_to_id", |name: &str| -> Result<rhai::INT, Box<EvalAltResult>> {
        let res = data::get_asset_id(name) as rhai::INT;
        if res == 0 {
            Err(format!("Tried to use 'asset_name_to_id' with an asset name that doesn't exist ('{}').", name).into())
        } else { Ok(res) }
    });

    // Converts an asset's id to it's name.
    // Returns an error if the id doesn't exist.
    engine.register_fn("asset_id_to_name", |id: rhai::INT| -> Result<String, Box<EvalAltResult>> {
        let res = data::get_asset_name(id as u32);
        if res.is_empty() {
            Err(format!("Tried to use 'asset_id_to_name' with an asset id that doesn't exist ('{}').", id).into())
        } else { Ok(res) }
    });

    // Uses web-sys to get the client
    // width and height of the browser.
    engine.register_fn("get_client_width", || -> rhai::FLOAT {
        web_sys::window().expect("window cast should succeed")
        .document().expect("document cast should succeed")
        .document_element().expect("document_element cast should succeed")
        .client_width() as rhai::FLOAT
    });
    engine.register_fn("get_client_height", || -> rhai::FLOAT {
        web_sys::window().expect("window cast should succeed")
        .document().expect("document cast should succeed")
        .document_element().expect("document_element cast should succeed")
        .client_height() as rhai::FLOAT
    });

    // For some weird reason, the rhai standard
    // package doesn't include a 'min' and 'max'
    // function for two integers (int-int) or
    // two floats (float-float), but only for 
    // an integer and a float combined (int-float).
    // Because of this, I had to implement them myself.
    engine.register_fn("min",
    |value1: rhai::INT, value2: rhai::INT| -> rhai::INT {
        return value1.min(value2);
    });
    engine.register_fn("max",
    |value1: rhai::INT, value2: rhai::INT| -> rhai::INT {
        return value1.max(value2);
    });
    engine.register_fn("min",
    |value1: rhai::FLOAT, value2: rhai::FLOAT| -> rhai::FLOAT {
        return value1.min(value2);
    });
    engine.register_fn("max",
    |value1: rhai::FLOAT, value2: rhai::FLOAT| -> rhai::FLOAT {
        return value1.max(value2);
    });
    
    // Here the state manager and the current scene's
    // scripts get run for the first time, which
    // means they get to use all of the API features
    // declaired above at the global scope.
    // Every API feature that was declaired after this
    // point will only be usable in a callback scope.
    state_manager.resources.borrow_mut().run_script(&engine)?;
    cur_scene.resources.borrow_mut().run_script(&engine)?;

    // Create the "object stack",
    // which is a vector of element
    // handlers that will be used as
    // a dynamic pool of objects, meaning
    // that when the current scene will be
    // switched, the objects will be recycled,
    // and memory won't be reallocated.
    let object_stack: Rc<RefCell<Vec<ElementHandler>>> = Rc::new(RefCell::new(Vec::new()));

    // In order to fill the object stack,
    // we'll need to borrow it mutably.
    // Therefore, we'll need to open a
    // seperate block for the filling
    // process and borrow the object stack
    // in it, so that it will be dropped
    // as soon as the filling process is done.
    {
        let mut object_stack_borrow = object_stack.borrow_mut();
        // Get the "object-instances" list
        // from the current scene's configuration.
        let instances = cur_scene.resources.borrow().definition
        .config["object-instances"].clone().into_typed_array::<Map>().expect(concat!("Every object's",
        " config should contain a 'object-instances' array, which should only have object-like members."));
        // Itrate over the instances list.
        for (idx, map, rowid, layer) in instances
        .iter().enumerate().map(|(inst_index, inst)| {(
            // Devide the information into seperate variables:
            // idx - the index of the instance in the list.
            // map - the instance's map, which contains
            //       all the instance's attributes.
            // rowid - the instance's object rowid.
            // layer - the instance's layer.
            inst_index as u32, inst,
            dynamic_to_number(&inst["id"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an integer 'id' attribute.")) as u32,
            dynamic_to_number(&inst["layer"])
            .expect(concat!("Every instance in the 'object-instances' array",
            " of an object's config should contain an integer 'layer' attribute.")),
        )}) {
            // Borrow the current scene's properties in a seperate block.
            {
                let mut scene_props_borrow = cur_scene.properties.borrow_mut();
                let mut scene_props_borrow = scene_props_borrow
                .write_lock::<element::Scene>().expect("write_lock cast should succeed");
                // Add the instance to it's matching layer in the current scene.
                scene_props_borrow.add_instance(idx as rhai::INT, layer as rhai::INT);
            } // The borrow of the current scene's properties drops here.
            
            // If the instance's object definition
            // isn'y already loaded, load it.
            if !element_defs.borrow().contains_key(&rowid) {
                element_defs.borrow_mut().insert(rowid,
                    ElementDefinition::new(&engine,
                    TableRow::Element(rowid, 1)
                ));
            }
            // Create a new element handler for the
            // instance and push it to the object stack.
            object_stack_borrow.push(ElementHandler::new(
                element_defs.borrow().get(&rowid).unwrap().as_ref()?,
                Some(element::ObjectInitInfo::new(idx, map))
            )?);
            // Run the instance's script.
            object_stack_borrow.last().unwrap().resources.borrow_mut().run_script(&engine)?;
        }
    }

    // Share a counted reference to the
    // object stack and current scene's
    // properties with the following API function.
    let api_object_stack = Rc::clone(&object_stack);
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("get_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<element::Object, Box<EvalAltResult>> {
        // Borrow the current scene's properties for reading (immutable)
        let scene_props_borrow = cur_scene_props.borrow();
        let scene_props_borrow = scene_props_borrow
        .read_lock::<element::Scene>().expect("read_lock cast should succeed");
        // Check if the index is in the range of the current scene's objects
        if idx >= (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as rhai::INT {
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(scene_props_borrow.objects_len+
            scene_props_borrow.runtimes_len, idx, context.position())));
        }
        // Borrow the object stack for reading (immutable)
        let object_stack_borrow;
        if let Ok(borrow) = api_object_stack.try_borrow() {
            object_stack_borrow = borrow;
        } else {
            // The object stack is being borrowed for 
            // writing while the scene is being loaded,
            // because it needs to be filled with instances,
            // and therefore we won't be able to borrow
            // it if the scene is being loaded.
            return Err("Can't use the global function 'get_object' while the scene is being loaded".into());
        }
        // If we find the object in the object stack,
        if let Some(element) = object_stack_borrow.get(idx as usize) {
            // return a clone of it's properties
            Ok(element.properties.borrow().clone_cast::<element::Object>())
        } else {
            // otherwise, return an error
            Err(Box::new(EvalAltResult::ErrorArrayBounds(scene_props_borrow.objects_len+
            scene_props_borrow.runtimes_len, idx, context.position())))
        }
    });

    let api_object_stack = Rc::clone(&object_stack);
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    engine.register_fn("message_object", move |context: rhai::NativeCallContext, idx: rhai::INT, 
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        // Because we need to message an object,
        // we can't keep the current scene's properties
        // borrowed for reading, because the message might
        // lead to a change in the current scene's properties.
        // Therefore, we'll need to borrow the current scene's
        // properties inside a seperate block, so we could clone
        // the information we need from it, and then drop the borrow.
        let objects_len: usize;
        let runtimes_len: usize;
        {
            let scene_props_borrow = cur_scene_props.borrow();
            let scene_props_borrow = scene_props_borrow
            .read_lock::<element::Scene>().expect("read_lock cast should succeed");
            objects_len = scene_props_borrow.objects_len;
            runtimes_len = scene_props_borrow.runtimes_len;
        }// scene_props_borrow drops here.

        // If the index is out of
        // bounds, return an error.
        if idx >= (objects_len+runtimes_len) as rhai::INT {
            let info = "Argument 'idx' was out of bounds in call to 'message_object'.";
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx, objects_len+runtimes_len);
            return Err(full_s.into());
        }
        // What I said about the current scene's properties
        // also applies to the object stack, so we'll need
        // to borrow it inside a seperate block, and only
        // clone the information we need from it.
        let mut element_res_clone: Option<Rc<RefCell<ElementResources>>> = None;
        {
            let object_stack_borrow;
            if let Ok(borrow) = api_object_stack.try_borrow() {
                object_stack_borrow = borrow;
            } else {
                // The object stack is being borrowed for 
                // writing while the scene is being loaded,
                // because it needs to be filled with instances,
                // and therefore we won't be able to borrow
                // it if the scene is being loaded.
                return Err("Can't use the global function 'message_object' while the scene is being loaded".into());
            }
            // The information we need from the object stack
            // is a counted reference to the object's resources,
            // which will be needed in order to message the object.
            if let Some(element) = object_stack_borrow.get(idx as usize) {
                element_res_clone = Some(Rc::clone(&element.resources));
            }
        }// object_stack_borrow drops here.

        // If we found the object in the object stack
        // and we have a counted reference to it's resources,
        if let Some(element) = element_res_clone {
            // Try to borrow the object's resources mutably.
            if let Ok(mut borrow) = element.try_borrow_mut() {
                // call the function with the given name and
                // arguments, and if an error is raised, return it.
                if let Some(err) = borrow.call_fn(context.engine(),&format!("message_{}", name), args).err() {
                    // Mention the use of the messaging API in the error message
                    Err(format!("{}\nas a result of a call to 'message_object'", err).into())
                } else { Ok(()) }
            } else {
                // otherwise, return an error
                Err(concat!("Can't use the 'message_object' function while that object's script is running",
                " (is handling another callback). Note: This might have happened because you tried to message yourself,",
                " or messaged an element, which tried to message you back in the scope of that same message.").into())
            }
        } else {
            // otherwise, the object doesn't exist, so return an error
            let info = "Argument 'idx' was out of bounds in call to 'message_object'.";
            let full_s = &format!("{}.\nTried to find index {} on the object stack, when it only had {} elements.",
            info, idx, objects_len+runtimes_len);
            Err(full_s.into())
        }
    });

    // Share a counted reference to
    // the object stack, current scene's
    // properties and element definitions
    // table with the following API function.
    let api_object_stack = Rc::clone(&object_stack);
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    let api_element_defs = Rc::clone(element_defs);
    engine.register_fn("add_object_to_stack", move |context: rhai::NativeCallContext,
    id_source: rhai::INT, init_x: rhai::FLOAT, init_y: rhai::FLOAT| -> Result<rhai::INT, Box<EvalAltResult>> {
        // Borrow the object stack for writing (mutable)
        let mut object_stack_borrow;
        if let Ok(borrow) = api_object_stack.try_borrow_mut() {
            object_stack_borrow = borrow;
        } else {
            // The object stack is being borrowed for 
            // writing while the scene is being loaded,
            // because it needs to be filled with instances,
            // and therefore we won't be able to borrow
            // it if the scene is being loaded.
            return Err("Can't use the global function 'add_object_to_stack' while the scene is being loaded".into());
        }
        // Find the definition of the object we want to add.
        let def_rc_clone: Rc<ElementDefinition>;
        if let Some(element_def) = api_element_defs.borrow().get(&(id_source as u32)) {
            // Take a counted reference to this
            // object's definition as long as no error occured
            // while loading the definition. If an error did occur,
            // it will be propagated back to the caller.
            def_rc_clone = Rc::clone(element_def.as_ref()?);
            // Make sure the definition is of an object.
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
            // If the definition doesn't exist, return an error.
            return Err("Tried to use 'add_object_to_stack' with a definition which doesn't exist.".into());
        }
        // Borrow the current scene's properties for writing (mutable)
        let mut scene_props_borrow = cur_scene_props.borrow_mut();
        let mut scene_props_borrow = scene_props_borrow
        .write_lock::<element::Scene>().expect("read_lock cast should succeed");

        // Find the deapest vacant instance index in the object stack's runtime pool.
        if let Some(&vacant_index) = scene_props_borrow.runtime_vacants.iter().min() {
            // Recycle the object instance at the
            // vacant index for the new object instance.
            let object = object_stack_borrow
            .get_mut(vacant_index as usize).expect("Shouldn't be out of bounds.");

            if let Err(err) = object.recycle(&def_rc_clone,
                Some(element::ObjectInitInfo {
                idx_in_stack: (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                init_x, init_y, init_scale_x: 1_f32, init_scale_y: 1_f32,
                init_color: String::from("#FFFFFF"), init_alpha: 255_u8
            })) {
                // Mention the use of this function in the error message
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            // Run the recycled object instance's script.
            if let Err(err) = object.resources.borrow_mut().run_script(context.engine()) {
                // Mention the use of this function in the error message
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            // Return the index of the recycled object instance.
            return Ok(vacant_index as rhai::INT);
        }
        // If there are no vacant instance indices in the object stack's runtime pool,
        // try to extend the pool within the current bounds of the object stack.
        if let Some(object_ref) = object_stack_borrow
        .get_mut(scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) {
            // Recycle the revealed object instance from the
            // end of the extended pool for the new object instance.
            if let Err(err) = object_ref.recycle(&def_rc_clone,
                Some(element::ObjectInitInfo {
                idx_in_stack: (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                init_x, init_y, init_scale_x: 1_f32, init_scale_y: 1_f32,
                init_color: String::from("#FFFFFF"), init_alpha: 255_u8
            })) {
                // Mention the use of this function in the error message
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            // Run the recycled object instance's script.
            if let Err(err) = object_ref.resources.borrow_mut().run_script(context.engine()) {
                // Mention the use of this function in the error message
                return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err).into());
            }
            // Extend the object stack's runtime pool.
            let index = (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32;
            scene_props_borrow.runtimes_len += 1;
            scene_props_borrow.runtime_vacants.push(index);   
            // Return the index of the recycled object instance.
            return Ok(index as rhai::INT);
        }
        // If the object stack doesn't have enough
        // room to extend it's runtime pool, allocate
        // more memory for a new object instance to fit in.
        object_stack_borrow.push(
            {
                // Create a new element handler
                // for the new object instance.
                let element = ElementHandler::new(
                &def_rc_clone, Some(element::ObjectInitInfo {
                    idx_in_stack: (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32,
                    init_x, init_y, init_scale_x: 1_f32, init_scale_y: 1_f32,
                    init_color: String::from("#FFFFFF"), init_alpha: 255_u8
                }));
                if element.is_err() {
                    // Mention the use of this function in the error message
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", element.err().unwrap()).into());
                }
                // Run the new object instance's script.
                let element = element.unwrap();
                if let Some(err_string) = element.resources.borrow_mut().run_script(context.engine()).err() {
                    // Mention the use of this function in the error message
                    return Err(format!("{}\nas a result of a call to 'add_object_to_stack'", err_string).into());
                }
                // Push the new object instance
                // to the object stack.
                element
            }
        );
        // Extend the object stack's runtime pool.
        let index = (scene_props_borrow.objects_len+scene_props_borrow.runtimes_len) as u32;
        scene_props_borrow.runtimes_len += 1;
        scene_props_borrow.runtime_vacants.push(index);
        // Return the index of the new object instance.
        Ok(index as rhai::INT)
    });

    // The API is now complete!
    // Return all the components,
    // which are related to the API
    // and will need to be maintained
    // throughout the game's operation.
    Ok((engine, state_manager, cur_scene, object_stack, key_states))
}