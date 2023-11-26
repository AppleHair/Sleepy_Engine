
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use rhai::Engine;

use crate::data;

/// Defines the game engine's API,
/// And all of it's associated types.
mod engine_api;
/// Defines the game's renderer.
mod renderer;

#[wasm_bindgen]
extern "C" {
    /// I had to make my own setTimeout
    /// import instead of using the one\
    /// from the `web_sys` crate, because 
    /// the one from `web_sys` uses i32\
    /// instead of f64 for the timeout 
    /// parameter,and I need to be able to\
    /// use floats for the timeout parameter,
    /// because it needs to be as\
    /// precise as possible.
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout_with_callback_and_f64(
        handler: &::js_sys::Function,
        timeout: f64,
    ) -> i32;
}

/// This enum is used to identify
/// a row in one of the neccessary\
/// tables in the database\game data
/// file by it's table(enum variant),\
/// rowid and type (except the `Metadata`
/// variant).
/// 
/// Because theres only one state
/// manager in the game, it's data isn't\
/// stored in the 'element' table, but in
/// unique rows in the 'blobs' table,\
/// refered to in this code base as `Metadata`.
#[derive(Clone, Copy)]
pub enum TableRow {
    Metadata,
    Element(u32, u8),
    Asset(u32, u8),
}

impl TableRow {
    /// When an error occurs in one of an
    /// elements' scripts or elsewhere in the\
    /// game, it's very useful to know what
    /// element or asset caused the error.
    /// 
    /// Therefore, I created this function,
    /// which will use an available `TableRow`\
    /// object to wrap an error message in a
    /// string, which will contain the name\
    /// of the element or asset, which caused
    /// the error.
    pub fn to_err_string(&self, err: &str) -> String {
        let self_str = match self.clone() {
            Self::Metadata => String::from("\non 'State Manager'."),
            Self::Element(id, kind) => format!("\non the '{}' {kind_str}.", data::get_element_name(id.clone()),
            kind_str = match kind { 1 => "object", 2 => "scene", _ => "element" }),
            Self::Asset(id, kind) => format!("\non the '{}' {kind_str}.", data::get_element_name(id.clone()),
            kind_str = match kind { 1 => "sprite", 2 => "audio", 3 => "font", _ => "asset" }),
        };
        format!("{}{}", err, self_str)
    }
}

/// Rhai dynamic values are evaluated
/// as integers or floats separately\
/// and they don't do any automatic
/// casting between the two types of\
/// numbers. This behavior is not very
/// convenient, because it requires\
/// you to check if the value is a float
/// or an integer every time you want to\
/// extract a number from a script or an
/// evaluated JSON file. Therefore, I\
/// created this function, which will
/// convert a dynamic value to a f32 if it's\
/// eather an integer(i32) or a float(f32),
/// and will return an error if it's not\
/// eather of the two. After that, you can
/// cast the result into any other number\
/// type you want, by using the 'as' keyword.
/// 
/// # Examples
/// 
/// ```rust
/// let dynamic = rhai::Dynamic::from(1);
/// let i: u8 = dynamic_to_number(&dynamic)? as u8;
/// assert_eq!(i, 1_u8);
/// 
/// let f: f32 = dynamic_to_number(&dynamic)?;
/// assert_eq!(f, 1_f32);
/// ```
pub fn dynamic_to_number(dynam: &rhai::Dynamic) -> Result<f32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f32);
    }
    Ok(dynam.as_float()? as f32)
}

/// This function will load all the assets,
/// which were not already loaded, into the\
/// received `AssetDefinitions` table.
/// 
/// It might also update the assets, which
/// were already loaded, if they were changed.
fn load_assets(engine: &Engine, asset_defs: &mut renderer::AssetDefinitions,
gl_context: &web_sys::WebGlRenderingContext) {
    // Gets the array of assets to load
    // from the data module and iterates over it.
    for asset in data::assets_to_load().iter() {
        // Gets the asset's id and type from the array.
        let int_id = js_sys::Reflect::get_u32(&asset, 0)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u32;
        let int_type = js_sys::Reflect::get_u32(&asset, 1)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u8;
        // Creates and inserts the asset into the table.
        // If it already there, it will be overwritten.
        asset_defs.insert(int_id, renderer::AssetDefinition::new(&engine,
        TableRow::Asset(int_id, int_type), gl_context));
    }
}

/// This function will load all the elements,
/// which were not already loaded, into the\
/// received `ElementDefinitions` table.
/// 
/// It might also update the elements, which
/// were already loaded, if they were changed.\
/// However, if the `init` parameter is set to
/// `true`, it will only load the elements, which\
/// were not already loaded.
fn load_elements(engine: &Engine, element_defs: &mut engine_api::ElementDefinitions, init: bool) {
    // Gets the array of elements to load
    // from the data module and iterates over it.
    for element in data::elements_to_load().iter() {
        // Gets the element's id and type from the array.
        let int_id = js_sys::Reflect::get_u32(&element, 0)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u32;
        let int_type = js_sys::Reflect::get_u32(&element, 1)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u8;
        // If the element is already loaded, and the init
        // parameter is set to true, it will skip the element.
        if element_defs.contains_key(&int_id) && init { continue; }
        // Creates and inserts the element into the table.
        // If it already there, it will be overwritten.
        element_defs.insert(int_id, engine_api::ElementDefinition::new(&engine,
        TableRow::Element(int_id, int_type)));
    }
}

/// This struct is used to track
/// the state of the keyboard keys.
pub struct KeyStateTracker {
    pub key_states: Rc<RefCell<engine_api::KeyStates>>,
    pub keys_just_changed: Rc<RefCell<Vec<String>>>,
    _keydown: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
    _keyup: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
}

impl KeyStateTracker {
    /// Takes a reference (interior mutated)
    /// to a `KeyStates` table, and uses it\
    /// to create a new `KeyStateTracker` instance.
    pub fn new(key_states: Rc<RefCell<engine_api::KeyStates>>) -> Result<Self, JsValue> {
        // Creates a new vector, which will be
        // used to track key state changes.
        let keys_just_changed = Rc::new(RefCell::new(Vec::new()));
        // Creates the keydown and keyup
        // closures, which will be used to
        // update the key states table.
        let event_key_states = Rc::clone(&key_states);
        let event_keys_just_changed = Rc::clone(&keys_just_changed);
        let onkeydown = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            // Ignores key repeats.
            if event.repeat() { return; }
            // Updates the key state table,
            // and adds the key to the vector
            // of keys whose state just changed.
            event_key_states.borrow_mut().insert(event.code(),
            engine_api::KeyState { is_held: true, just_pressed: true, just_released: false });
            event_keys_just_changed.borrow_mut().push(event.code());
        });
        let event_key_states = Rc::clone(&key_states);
        let event_keys_just_changed = Rc::clone(&keys_just_changed);
        let onkeyup = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            // Ignores key repeats.
            if event.repeat() { return; }
            // Updates the key state table,
            // and adds the key to the vector
            // of keys whose state just changed.
            event_key_states.borrow_mut().insert(event.code(),
            engine_api::KeyState { is_held: false, just_pressed: false, just_released: true });
            event_keys_just_changed.borrow_mut().push(event.code());
        });
        // Adds the keydown and keyup closures
        // to the document's appropriate
        // event listeners.
        window().unwrap().document().unwrap()
        .add_event_listener_with_callback("keydown", onkeydown.as_ref().unchecked_ref())?;
        window().unwrap().document().unwrap()
        .add_event_listener_with_callback("keyup", onkeyup.as_ref().unchecked_ref())?;

        // Returns the new `KeyStateTracker` instance.
        Ok(Self {
            key_states,
            keys_just_changed,
            _keydown: onkeydown,
            _keyup: onkeyup
        })
    }

    /// This function should be called
    /// at the end of every frame, to\
    /// make sure that the key states
    /// are updated correctly.
    /// 
    /// It will set the `just_pressed`
    /// and `just_released` fields of\
    /// every key, which was pressed or
    /// released during the frame, to\
    /// false. It will also clear the
    /// vector of keys, which were pressed\
    /// or released during the frame.
    pub fn calibrate(&self) {
        // Gets a mutable reference to the
        // key states table, and iterates
        // over the vector of keys, which
        // were pressed or released during
        // the frame.
        let mut key_states_borrow = self.key_states.borrow_mut();
        for key in self.keys_just_changed.borrow().iter() {
            // Sets the `just_pressed` and
            // `just_released` fields of
            // the key's state to false.
            key_states_borrow.get_mut(key)
            .expect("key should exist if it's inside the keys_just_changed vector")
            .just_pressed = false;
            key_states_borrow.get_mut(key)
            .expect("key should exist if it's inside the keys_just_changed vector")
            .just_released = false;
        }
        // Clears the vector of keys,
        // which were pressed or released
        // during the frame.
        self.keys_just_changed.borrow_mut().clear();
    }
}

/// This struct handles all
/// the game engine's core 
/// functionality according to
/// the functions called on it.
pub struct Game {
    engine_api: Rc<rhai::Engine>,
    game_elements: Rc<engine_api::GameElementSet>,
    key_tracker: Option<KeyStateTracker>,
    webgl_renderer: Option<renderer::WebGlRenderer>,
    element_defs: Rc<RefCell<engine_api::ElementDefinitions>>,
    asset_defs: Option<renderer::AssetDefinitions>,
}

impl Game {
    /// Creates a new `Game`.\
    /// 
    /// Initializes the game's components,\
    /// Loads the game's assets and elements,\
    /// and defines the game's initial state.
    pub fn new() -> Result<Self, JsValue> {
        // Create the element definitions table.
        let element_defs: Rc<RefCell<engine_api::ElementDefinitions>> 
            = Rc::new(RefCell::new(HashMap::new()));
        // Create the asset definitions table.
        let mut asset_defs: renderer::AssetDefinitions = HashMap::new();
        // Create the game engine's API in
        // a new rhai `Engine`, and get all
        // the components which are integrated
        // with the API.
        let (engine_api, game_elements,
        key_states) = engine_api::create_api(&element_defs)?;
        // Create the key state tracker
        // using the key states table,
        // which is already integrated
        // with the API.
        let key_tracker = KeyStateTracker::new(key_states)?;
        // Load all the elements which
        // were not already loaded.
        load_elements(&engine_api, &mut element_defs.borrow_mut(), true);
        // Create the WebGL renderer
        // the game will use to render
        // it's graphics.
        let webgl_renderer = renderer::WebGlRenderer::new(
        &game_elements.state_manager.properties.borrow()
        .read_lock::<engine_api::element::Game>()
        .expect("read_lock cast should succeed"))?;
        // Load all the assets
        load_assets(&engine_api, &mut asset_defs, &webgl_renderer.gl_context);
        // Call the `init` function on all the elements.
        game_elements.call_fn_on_all("init", (), &engine_api)?;
        // Return the new `Game`.
        Ok(Self {
            engine_api,
            game_elements,
            key_tracker: Some(key_tracker),
            webgl_renderer: Some(webgl_renderer),
            element_defs,
            asset_defs: Some(asset_defs),
        })
    }

    /// This function will start the game's draw\
    /// loop, which will render the game's graphics.
    /// 
    /// After the draw loop is started,\
    /// it will be impossible to start it again.
    pub fn start_draw_loop(&mut self) -> Result<(), JsValue> {
        // Takes the WebGL renderer
        // and asset definitions out
        // of the `Game` struct.
        let mut webgl_renderer = self.webgl_renderer.take()
        .ok_or(JsValue::from_str("Tried to start the draw loop a second time."))?;
        let mut asset_defs = self.asset_defs.take().unwrap();
        // Take a reference to the
        // game elements and the
        // rhai API engine.
        let game_elements = Rc::clone(&self.game_elements);
        let engine_api = Rc::clone(&self.engine_api);

        // Set up the draw loop:

        // The closure will be stored in
        // an `Option`, which will start
        // out as `None`, wraped in a
        // counted reference (interior mutated)
        let draw_loop = Rc::new(RefCell::new(
        None::<Closure::<dyn FnMut(f64) -> Result<(), JsValue>>>));
        // We will need to clone the
        // closure's reference to be
        // able to use the closure
        // inside itself.
        let draw_init = Rc::clone(&draw_loop);
        // We will also need to store
        // the time of the last frame
        // in a mutable variable, so
        // we can calculate the time
        // elapsed between frames.
        let mut last_draw = window().unwrap().performance().unwrap().now();
        // Here we create the closure
        // itself using the `draw_init`
        // reference, which will also
        // be used to request the first
        // frame, and the next frame will
        // be requested with the `draw_loop`
        // reference from inside the closure.
        *draw_init.borrow_mut() = Some(Closure::<dyn FnMut(f64) -> Result<(), JsValue>>::new(
        move |draw_time: f64| -> Result<(), JsValue> {
            // Calculate the time elapsed
            let elapsed = draw_time - last_draw;
            last_draw = draw_time;
            // Load all the assets which
            // were not already loaded, and
            // update the ones which were.
            load_assets(&engine_api,
            &mut asset_defs, &webgl_renderer.gl_context);
            // Render the game's graphics.
            webgl_renderer.render_scene(
                &game_elements.state_manager.properties
                .borrow().read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed"),
                &game_elements.cur_scene.properties
                .borrow().read_lock::<engine_api::element::Scene>()
                .expect("read_lock cast should succeed"),
                &game_elements.object_stack.borrow(),
                &asset_defs, elapsed
            )?;
            // Request the next frame.
            window().unwrap().request_animation_frame(
                draw_loop
                    .borrow().as_ref().unwrap()
                    .as_ref().unchecked_ref()
            )?;
            
            Ok(())
        }));
        // Start the draw loop.
        window().unwrap().request_animation_frame(
            draw_init
                .borrow().as_ref().unwrap()
                .as_ref().unchecked_ref()
        )?;
        
        Ok(())
    }

    /// This function will start the game's main\
    /// update loop, which will change the game's state.
    /// 
    /// After the update loop is started,\
    /// it will be impossible to start it again.
    pub fn start_main_loop(&mut self) -> Result<(), JsValue> {
        // Takes the key state tracker
        // out of the `Game` struct.
        let key_tracker = self.key_tracker.take()
        .ok_or(JsValue::from_str("Tried to start the main loop a second time."))?;
        // Take a reference to the
        // game elements, rhai API
        // engine, and element definitions.
        let game_elements = Rc::clone(&self.game_elements);
        let engine_api = Rc::clone(&self.engine_api);
        let element_defs = Rc::clone(&self.element_defs);
        
        // Set up the update loop:

        // The closure will be stored in
        // an `Option`, which will start
        // out as `None`, wraped in a
        // counted reference (interior mutated)
        let update_loop = Rc::new(RefCell::new(
            None::<Closure::<dyn FnMut() -> Result<(), JsValue>>>
        ));
        // We will need to clone the
        // closure's reference to be
        // able to use the closure
        // inside itself.
        let update_init = Rc::clone(&update_loop);
        // We will also need to store
        // the time of the last frame
        // in a mutable variable, so
        // we can calculate the time
        // elapsed between frames.
        let mut last_update = window().unwrap().performance().unwrap().now();
        // Here we create the closure
        // itself using the `update_init`
        // reference, which will also
        // be used to request the first
        // frame, and the next frame will
        // be requested with the `update_loop`
        // reference from inside the closure.
        *update_init.borrow_mut() = Some(Closure::<dyn FnMut() -> Result<(), JsValue>>::new(
        move || -> Result<(), JsValue> {
            // Calculate the time elapsed
            let update_time = window().unwrap().performance().unwrap().now();
            let elapsed = update_time - last_update;
            last_update = update_time;
            // Call the `update` function on all the elements.
            game_elements.call_fn_on_all("update", (elapsed as rhai::FLOAT, ), &engine_api)?;
            // Calibrate the key states.
            key_tracker.calibrate();
            // Get the current scene's id.
            let row_copy = game_elements.cur_scene.resources.borrow().definition.row;
            if let TableRow::Element(id, 2) = row_copy {
                // Make a mutable copy of the current scene's id.
                let mut prv_scene_id = id;
                // Get the state manager's `cur_scene` property.
                let mut cur_scene_id = game_elements.state_manager.properties.borrow()
                .read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed").cur_scene;
                // If the current scene's id is different from
                // what the state manager's `cur_scene` property
                // implies, switch to the current scene.
                while cur_scene_id != prv_scene_id {
                    game_elements.switch_scene(cur_scene_id,
                    &engine_api, &element_defs.borrow())?;
                    // Call the `init` function on all the elements.
                    game_elements.call_fn_on_all("init", (), &engine_api)?;
                    // Update the previous scene id to
                    // the scene id we just switched to,
                    // and get the `cur_scene` property again.
                    prv_scene_id = cur_scene_id;
                    cur_scene_id = game_elements.state_manager.properties
                    .borrow().read_lock::<engine_api::element::Game>()
                    .expect("read_lock cast should succeed").cur_scene;
                // If the state manager switched the scene again in the init function,
                // keep switching the scene until the two values are equal.
                }
            }
            // Load all the elements which
            // were not already loaded, and
            // update the ones which were.
            load_elements(&engine_api, &mut element_defs.borrow_mut(), false);
            // Request the next frame.
            set_timeout_with_callback_and_f64(
                update_loop
                    .borrow().as_ref().unwrap()
                    .as_ref().unchecked_ref(),
                1000_f64 / (game_elements.state_manager
                .properties.borrow().read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed").fps as f64)
            );

            Ok(())
        }));
        // Start the update loop.
        set_timeout_with_callback_and_f64(
            update_init
                .borrow().as_ref().unwrap()
                .as_ref().unchecked_ref(),
            1000_f64 / (self.game_elements.state_manager
            .properties.borrow().read_lock::<engine_api::element::Game>()
            .expect("read_lock cast should succeed").fps as f64)
        );
        
        Ok(())
    }
}