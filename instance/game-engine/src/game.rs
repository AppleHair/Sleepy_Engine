
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use rhai::Engine;

use crate::data;

mod engine_api;
mod renderer;

//
#[wasm_bindgen(catch)]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout_with_callback_and_f64(
        handler: &::js_sys::Function,
        timeout: f64,
    ) -> i32;
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
    pub fn init(key_states: Rc<RefCell<engine_api::KeyStates>>) -> Result<Self, JsValue> {
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
fn call_fn_on_all(name: &str, args: impl rhai::FuncArgs + Clone, engine: &Engine,
state_manager: &Rc<RefCell<engine_api::ElementResources>>, scene: &engine_api::ElementHandler,
object_stack: &Rc<RefCell<Vec<engine_api::ElementHandler>>>) -> Result<(), String> {
    // Call the function on the state manager instance.
    state_manager.borrow_mut().call_fn(engine, name, args.clone())?;
    //
    scene.resources.borrow_mut().call_fn(engine, name, args.clone())?;
    //
    let mut i = 0_usize;
    loop {
        //
        {
            //
            let scene_map_borrow = scene.properties.borrow();
            let scene_map_borrow = scene_map_borrow
            .read_lock::<engine_api::element::Scene>().expect("read_lock cast should succeed");
            //
            if i >= scene_map_borrow.objects_len+scene_map_borrow.runtimes_len {
                break;
            }
            //
            if ! scene_map_borrow.layers[0..scene_map_borrow.layers_len]
            .iter().flat_map(|layer| { layer.instances.iter()})
            .any(|&index| { index == i as u32 }) {
                //
                i += 1;
                continue;
            }
        }//

        //
        let mut element_res_clone: Option<Rc<RefCell<engine_api::ElementResources>>> = None;
        {
            //
            let object_stack_borrow = object_stack.borrow();
            //
            if let Some(element) = object_stack_borrow.get(i) {
                //
                element_res_clone = Some(Rc::clone(&element.resources));
            }
        }//

        //
        if let Some(element) = element_res_clone {
            //
            if let Ok(mut borrow) = element.try_borrow_mut() {
                //
                borrow.call_fn(engine, name, args.clone())?;
            }
        }
        //
        i += 1;
    }
    //
    Ok(())
}

//
fn switch_scene(scene_id: u32, engine: &Engine, scene: &engine_api::ElementHandler,
object_stack: &Rc<RefCell<Vec<engine_api::ElementHandler>>>,
element_defs: &engine_api::ElementDefinitions) -> Result<(), String> {
    //
    scene.recycle(
        element_defs.get(&scene_id).unwrap().as_ref()?,
        None
    )?;
    //
    scene.resources.borrow_mut().run_script(&engine)?;
    //
    let instances = scene.resources.borrow().definition
    .config["object-instances"].clone().into_typed_array::<rhai::Map>().expect(concat!("Every object's",
    " config should contain a 'object-instances' array, which should only have object-like members."));
    //
    let mut object_stack_borrow = object_stack.borrow_mut();
    //
    let scene_props_borrow = Rc::clone(&scene.properties);
    //
    for (idx, map, rowid, layer) in instances
    .iter().enumerate().map(|(inst_index, inst)| {(
        //
        inst_index, inst,
        dynamic_to_number(&inst["id"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an object's config should contain an integer 'id' attribute.")) as u32,
        dynamic_to_number(&inst["layer"])
        .expect(concat!("Every instance in the 'object-instances' array",
        " of an object's config should contain an integer 'layer' attribute.")),
    )}) {
        //
        {
            let mut scene_props_borrow = scene_props_borrow.borrow_mut();
            let mut scene_props_borrow = scene_props_borrow
            .write_lock::<engine_api::element::Scene>().expect("write_lock cast should succeed");
            //
            scene_props_borrow.add_instance(idx as rhai::INT, layer as rhai::INT);
        } //
        //
        if idx < object_stack_borrow.len() {
            //
            object_stack_borrow[idx].recycle(
            element_defs.get(&rowid).unwrap().as_ref()?,
            Some(engine_api::element::ObjectInitInfo::new(idx as u32, map)))?;
            //
            object_stack_borrow[idx].resources.borrow_mut().run_script(&engine)?;
        }
        //
        object_stack_borrow.push(engine_api::ElementHandler::new(
            element_defs.get(&rowid).unwrap().as_ref()?,
            Some(engine_api::element::ObjectInitInfo::new(idx as u32, map))
        )?);
        //
        object_stack_borrow.last().unwrap().resources.borrow_mut().run_script(&engine)?;
    }
    //
    Ok(())
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
pub fn run_game() -> Result<(), JsValue>
{
    // Create the element definitions hash map.
    let element_defs: Rc<RefCell<engine_api::ElementDefinitions>>
         = Rc::new(RefCell::new(HashMap::new()));
    // Create the asset definitions hash map.
    let mut asset_defs: renderer::AssetDefinitions = HashMap::new();

    // Create the API 'Engine', and the state manager instance.
    let (engine, state_manager, 
    cur_scene, object_stack,
    key_states) = engine_api::create_api(&element_defs)?;
    //
    load_elements(&engine, &mut element_defs.borrow_mut(), true);

    //
    let key_tracker = KeyStateTracker::init(key_states)?;

    //
    call_fn_on_all("init", (), &engine,
    &state_manager.resources, &cur_scene, &object_stack)?;

    //
    let (gl_context, gl_program,
    program_data,
    vertex_buffer, index_buffer) = 
        renderer::create_rendering_components(&state_manager.properties.borrow()
        .read_lock::<engine_api::element::Game>()
        .expect("read_lock cast should succeed"))?;

    //
    load_assets(&engine, &mut asset_defs, &gl_context);
    
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
    let cur_scene_props = Rc::clone(&cur_scene.properties);
    //
    let state_manager_props = Rc::clone(&state_manager.properties);
    //
    let renderer_object_stack = Rc::clone(&object_stack);
    //
    let json_engine = Engine::new_raw();
    //
    *draw_init.borrow_mut() = Some(Closure::<dyn FnMut(f64) -> Result<(), JsValue>>::new(
    move |draw_time: f64| -> Result<(), JsValue> {
        //
        let elapsed = draw_time - last_draw;
        last_draw = draw_time;

        //
        load_assets(&json_engine,
        &mut asset_defs, &gl_context);

        //
        renderer::render_scene(
            &gl_context, &gl_program, &program_data, &vertex_buffer,
            &index_buffer, &state_manager_props.borrow()
            .read_lock::<engine_api::element::Game>()
            .expect("read_lock cast should succeed"),
            &cur_scene_props.borrow()
            .read_lock::<engine_api::element::Scene>()
            .expect("read_lock cast should succeed"),
            &renderer_object_stack.borrow(),
            &asset_defs, elapsed
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
    let update_loop = Rc::new(RefCell::new(
    None::<Closure::<dyn FnMut() -> Result<(), JsValue>>>));
    //
    let update_init = Rc::clone(&update_loop);
    //
    let frame_rate_init = state_manager.properties.borrow()
    .read_lock::<engine_api::element::Game>()
    .expect("read_lock cast should succeed").fps as u32;
    //
    let mut last_update = window().unwrap().performance().unwrap().now();
    //
    *update_init.borrow_mut() = Some(Closure::<dyn FnMut() -> Result<(), JsValue>>::new(
    move || -> Result<(), JsValue> {
        //
        let update_time = window().unwrap().performance().unwrap().now();
        let elapsed = update_time - last_update;
        last_update = update_time;
        //
        call_fn_on_all("update", (elapsed as rhai::FLOAT, ), &engine,
        &state_manager.resources, &cur_scene, &object_stack)?;
        //
        key_tracker.calibrate();
        //
        let row_copy = cur_scene.resources.borrow().definition.row;
        //
        if let TableRow::Element(id, 2) = row_copy {
            //
            let mut cur_scene_id = state_manager.properties.borrow()
            .read_lock::<engine_api::element::Game>()
            .expect("read_lock cast should succeed").cur_scene;
            //
            let mut prv_scene_id = id;
            //
            while cur_scene_id != prv_scene_id {
                //
                switch_scene(cur_scene_id, &engine,
                &cur_scene, &object_stack, &element_defs.borrow())?;
                //
                call_fn_on_all("init", (), &engine,
                &state_manager.resources, &cur_scene, &object_stack)?;
                //
                prv_scene_id = cur_scene_id;
                cur_scene_id = state_manager.properties.borrow()
                .read_lock::<engine_api::element::Game>()
                .expect("read_lock cast should succeed").cur_scene;
            }
        }

        //
        load_elements(&engine, &mut element_defs.borrow_mut(), false);

        //
        set_timeout_with_callback_and_f64(
            update_loop
                .borrow().as_ref().unwrap()
                .as_ref().unchecked_ref(),
            1000_f64 / (state_manager.properties.borrow()
            .read_lock::<engine_api::element::Game>()
            .expect("read_lock cast should succeed").fps as f64)
        );
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
    set_timeout_with_callback_and_f64(
        update_init
            .borrow().as_ref().unwrap()
            .as_ref().unchecked_ref(),
        1000_f64 / (frame_rate_init as f64)
    );

    // Done!
    Ok(())
}