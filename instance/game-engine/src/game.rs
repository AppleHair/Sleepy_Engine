
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use rhai::Engine;

use crate::data;

mod engine_api;
mod renderer;

//
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout_with_callback_and_f64(
        handler: &::js_sys::Function,
        timeout: f64,
    ) -> i32;
}

// Rhai dynamic values are evaluated
// as integers or floats separately
// and they don't do any automatic casting
// between the two types of numbers.
// This behavior is not very convenient,
// because it requires you to check if 
// the value is a float or an integer 
// every time you want to extract a number
// from a script or an evaluated JSON file.
// Therefore, I created this function, which 
// will convert a dynamic value to a f32 if it's
// eather an integer(i32) or a float(f32), and 
// will return an error if it's not eather of
// the two. After that, you can cast the result
// into any other number type you want, by using
// the 'as' keyword, like this: 
// 'let x: u8 = dynamic_to_number(&dynamic)? as u8;'.
pub fn dynamic_to_number(dynam: &rhai::Dynamic) -> Result<f32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f32);
    }
    Ok(dynam.as_float()? as f32)
}

//
fn load_assets(engine: &Engine, asset_defs: &mut renderer::AssetDefinitions,
gl_context: &web_sys::WebGlRenderingContext) {
    for asset in data::assets_to_load().iter() {
        //
        let int_id = js_sys::Reflect::get_u32(&asset, 0)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u32;
        //
        let int_type = js_sys::Reflect::get_u32(&asset, 1)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u8;
        //
        asset_defs.insert(int_id, renderer::AssetDefinition::new(&engine,
        TableRow::Asset(int_id, int_type), gl_context));
    }
}

//
fn load_elements(engine: &Engine, element_defs: &mut engine_api::ElementDefinitions, init: bool) {
    for element in data::elements_to_load().iter() {
        //
        let int_id = js_sys::Reflect::get_u32(&element, 0)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u32;
        //
        let int_type = js_sys::Reflect::get_u32(&element, 1)
        .expect("The returned JSValue should be a array with two numbers.").as_f64()
        .expect("The returned JSValue should be a array with two numbers.") as u8;
        //
        if element_defs.contains_key(&int_id) && init { continue; }
        //
        element_defs.insert(int_id, engine_api::ElementDefinition::new(&engine,
        TableRow::Element(int_id, int_type)));
    }
}

//
#[derive(Clone, Copy)]
pub enum TableRow {
    // Because theres only one state manager in the game,
    // it's data isn't stored in the 'element' table,
    // but in unique rows in the 'blobs' table, refered
    // to in this code base as 'metadata'.
    Metadata,
    Element(u32, u8),
    Asset(u32, u8),
}

impl TableRow {
    pub fn to_err_string(&self, err: &str) -> String {
        let self_str = match self.clone() {
            //
            Self::Metadata => String::from("\non 'State Manager'."),
            //
            Self::Element(id, kind) => format!("\non the '{}' {kind_str}.", data::get_element_name(id.clone()),
            kind_str = match kind { 1 => "object", 2 => "scene", _ => "element" }),
            //
            Self::Asset(id, kind) => format!("\non the '{}' {kind_str}.", data::get_element_name(id.clone()),
            kind_str = match kind { 1 => "sprite", 2 => "audio", 3 => "font", _ => "asset" }),
        };
        format!("{}{}", err, self_str)
    }
}

//
pub struct KeyStateTracker {
    pub key_states: Rc<RefCell<engine_api::KeyStates>>,
    pub keys_just_changed: Rc<RefCell<Vec<String>>>,
    _keydown: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
    _keyup: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
}

impl KeyStateTracker {
    pub fn new(key_states: Rc<RefCell<engine_api::KeyStates>>) -> Result<Self, JsValue> {
        //
        let keys_just_changed = Rc::new(RefCell::new(Vec::new()));
        //
        let event_key_states = Rc::clone(&key_states);
        let event_keys_just_changed = Rc::clone(&keys_just_changed);
        let onkeydown = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            //
            if event.repeat() {
                return;
            }
            //
            let mut key_states_borrow = event_key_states.borrow_mut();
            let mut keys_just_changed_borrow = event_keys_just_changed.borrow_mut();
            //
            key_states_borrow.insert(event.code(), engine_api::KeyState {
                is_held: true, just_pressed: true, just_released: false
            });
            //
            keys_just_changed_borrow.push(event.code());
        });
        //
        window()
        .unwrap().document()
        .unwrap().add_event_listener_with_callback("keydown", onkeydown.as_ref().unchecked_ref())?;

        //
        let event_key_states = Rc::clone(&key_states);
        let event_keys_just_changed = Rc::clone(&keys_just_changed);
        let onkeyup = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
        move |event: web_sys::KeyboardEvent| {
            //
            if event.repeat() {
                return;
            }
            //
            let mut key_states_borrow = event_key_states.borrow_mut();
            let mut keys_just_changed_borrow = event_keys_just_changed.borrow_mut();
            //
            key_states_borrow.insert(event.code(), engine_api::KeyState {
                is_held: false, just_pressed: false, just_released: true
            });
            //
            keys_just_changed_borrow.push(event.code());
        });
        //
        window()
        .unwrap().document()
        .unwrap().add_event_listener_with_callback("keyup", onkeyup.as_ref().unchecked_ref())?;

        //
        Ok(Self {
            key_states,
            keys_just_changed,
            _keydown: onkeydown,
            _keyup: onkeyup
        })
    }

    pub fn calibrate(&self) {
        //
        let mut key_states_borrow = self.key_states.borrow_mut();
        let mut keys_just_changed_borrow = self.keys_just_changed.borrow_mut();
        //
        for key in keys_just_changed_borrow.clone() {
            //
            key_states_borrow.get_mut(&key)
            .expect("key should exist if it's inside the keys_just_changed vector")
            .just_pressed = false;
            key_states_borrow.get_mut(&key)
            .expect("key should exist if it's inside the keys_just_changed vector")
            .just_released = false;
        }
        //
        keys_just_changed_borrow.clear();
    }
}

//
pub struct Game {
    pub engine_api: Rc<rhai::Engine>,
    pub game_elements: Rc<engine_api::GameElementSet>,
    pub key_tracker: Rc<KeyStateTracker>,
    webgl_renderer: Rc<RefCell<renderer::WebGlRenderer>>,
    element_defs: Rc<RefCell<engine_api::ElementDefinitions>>,
    asset_defs: Rc<RefCell<renderer::AssetDefinitions>>,
    main_loop_started: bool,
    draw_loop_started: bool,
}

impl Game {
    //
    pub fn new() -> Result<Self, JsValue> {
        // Create the element definitions hash map.
        let element_defs: Rc<RefCell<engine_api::ElementDefinitions>>
            = Rc::new(RefCell::new(HashMap::new()));
        // Create the asset definitions hash map.
        let asset_defs: Rc<RefCell<renderer::AssetDefinitions>>
            = Rc::new(RefCell::new(HashMap::new()));

        // Create the API 'Engine', and the state manager instance.
        let (engine, game_elements,
        key_states) = engine_api::create_api(&element_defs)?;
        //
        load_elements(&engine, &mut element_defs.borrow_mut(), true);
        //
        let key_tracker = Rc::new(KeyStateTracker::new(key_states)?);
        //
        game_elements.call_fn_on_all("init", (), &engine)?;
        //
        let webgl_renderer = Rc::new(RefCell::new(renderer::WebGlRenderer::new(
        &game_elements.state_manager.properties.borrow()
        .read_lock::<engine_api::element::Game>()
        .expect("read_lock cast should succeed")
        )?));
        //
        load_assets(&engine, &mut asset_defs.borrow_mut(), &webgl_renderer.borrow().gl_context);
        //
        Ok(Self {
            engine_api: engine,
            game_elements,
            key_tracker,
            webgl_renderer,
            element_defs,
            asset_defs,
            main_loop_started: false,
            draw_loop_started: false,
        })
    }

    //
    pub fn start_draw_loop(&mut self) -> Result<(), JsValue> {
        //
        if self.draw_loop_started { return Ok(()); }
        //
        let draw_loop = Rc::new(RefCell::new(
        None::<Closure::<dyn FnMut(f64) -> Result<(), JsValue>>>));
        //
        let draw_init = Rc::clone(&draw_loop);
        //
        let mut last_draw = window()
        .unwrap().performance()
        .unwrap().now();
        //
        let game_elements = Rc::clone(&self.game_elements);
        let engine_api = Rc::clone(&self.engine_api);
        let webgl_renderer = Rc::clone(&self.webgl_renderer);
        let asset_defs = Rc::clone(&self.asset_defs);
        //
        *draw_init.borrow_mut() = Some(Closure::<dyn FnMut(f64) -> Result<(), JsValue>>::new(
        move |draw_time: f64| -> Result<(), JsValue> {
            //
            let elapsed = draw_time - last_draw;
            last_draw = draw_time;

            //
            load_assets(&engine_api,
            &mut asset_defs.borrow_mut(), &webgl_renderer.borrow().gl_context);

            //
            webgl_renderer.borrow_mut().render_scene(
                &game_elements.state_manager.properties
                .borrow().read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed"),
                &game_elements.cur_scene.properties
                .borrow().read_lock::<engine_api::element::Scene>()
                .expect("read_lock cast should succeed"),
                &game_elements.object_stack.borrow(),
                &asset_defs.borrow(), elapsed
            )?;
            //
            window().unwrap().request_animation_frame(
                draw_loop
                    .borrow().as_ref().unwrap()
                    .as_ref().unchecked_ref()
            )?;
            //
            Ok(())
        }));
        //
        window().unwrap().request_animation_frame(
            draw_init
                .borrow().as_ref().unwrap()
                .as_ref().unchecked_ref()
        )?;
        //
        self.draw_loop_started = true;
        //
        Ok(())
    }

    //
    pub fn start_main_loop(&mut self) -> Result<(), JsValue> {
        //
        if self.main_loop_started { return Ok(()); }
        //
        let update_loop = Rc::new(RefCell::new(
            None::<Closure::<dyn FnMut() -> Result<(), JsValue>>>
        ));
        //
        let update_init = Rc::clone(&update_loop);
        //
        let mut last_update = window().unwrap().performance().unwrap().now();
        //
        let game_elements = Rc::clone(&self.game_elements);
        let engine_api = Rc::clone(&self.engine_api);
        let element_defs = Rc::clone(&self.element_defs);
        let key_tracker = Rc::clone(&self.key_tracker);
        //
        *update_init.borrow_mut() = Some(Closure::<dyn FnMut() -> Result<(), JsValue>>::new(
        move || -> Result<(), JsValue> {
            //
            let update_time = window().unwrap().performance().unwrap().now();
            let elapsed = update_time - last_update;
            last_update = update_time;
            //
            game_elements.call_fn_on_all("update", (elapsed as rhai::FLOAT, ), &engine_api)?;
            //
            key_tracker.calibrate();
            //
            let row_copy = game_elements.cur_scene.resources.borrow().definition.row;
            //
            if let TableRow::Element(id, 2) = row_copy {
                //
                let mut cur_scene_id = game_elements.state_manager.properties.borrow()
                .read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed").cur_scene;
                //
                let mut prv_scene_id = id;
                //
                while cur_scene_id != prv_scene_id {
                    //
                    game_elements.switch_scene(cur_scene_id,
                    &engine_api, &element_defs.borrow())?;
                    //
                    game_elements.call_fn_on_all("init", (), &engine_api)?;
                    //
                    prv_scene_id = cur_scene_id;
                    cur_scene_id = game_elements.state_manager.properties
                    .borrow().read_lock::<engine_api::element::Game>()
                    .expect("read_lock cast should succeed").cur_scene;
                }
            }
    
            //
            load_elements(&engine_api, &mut element_defs.borrow_mut(), false);
    
            //
            set_timeout_with_callback_and_f64(
                update_loop
                    .borrow().as_ref().unwrap()
                    .as_ref().unchecked_ref(),
                1000_f64 / (game_elements.state_manager
                .properties.borrow().read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed").fps as f64)
            );
            //
            Ok(())
        }));
        //
        set_timeout_with_callback_and_f64(
            update_init
                .borrow().as_ref().unwrap()
                .as_ref().unchecked_ref(),
            1000_f64 / (self.game_elements.state_manager
            .properties.borrow().read_lock::<engine_api::element::Game>()
            .expect("read_lock cast should succeed").fps as f64)
        );
        //
        self.main_loop_started = true;
        //
        Ok(())
    }
}