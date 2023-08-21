
use crate::data;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use rhai::Engine;

mod entity;
mod rhai_api;
mod renderer;



#[wasm_bindgen]
pub struct ClosuresHandle {
    interval_id: i32,
    _interval: Closure::<dyn Fn() -> Result<(), JsValue>>,
    _keydown: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
    _keyup: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
}

impl Drop for ClosuresHandle {
    fn drop(&mut self) {
        window().unwrap().clear_interval_with_handle(self.interval_id);
    }
}



//
pub fn run_game() -> Result<ClosuresHandle, JsValue>
{
    // Create the object definitions hash map.
    let mut object_defs: HashMap<u32,Rc<rhai_api::EntityDefinition>> = HashMap::new();

    //
    let keys_just_changed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    // Create the API 'Engine', and the state manager instance.
    let (api_engine, state_manager, 
        cur_scene, mut cur_scene_id, 
        object_stack,
        key_states) = rhai_api::create_api(&mut object_defs)?;
    
    //
    let event_key_states = Rc::clone(&key_states);
    //
    let event_keys_just_changed = Rc::clone(&keys_just_changed);
    //
    let onkeydown = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
    move |event: web_sys::KeyboardEvent| {
        //
        let mut key_states_borrow = event_key_states.borrow_mut();
        //
        let mut keys_just_changed_borrow = event_keys_just_changed.borrow_mut();
        //
        key_states_borrow.insert(event.key(), rhai_api::KeyState { is_held: true, just_pressed: true, just_released: false });
        //
        keys_just_changed_borrow.push(event.key());
    });

    window().unwrap().document().unwrap().add_event_listener_with_callback("keydown", onkeydown.as_ref().unchecked_ref())?;

    //
    let event_key_states = Rc::clone(&key_states);
    //
    let event_keys_just_changed = Rc::clone(&keys_just_changed);
    //
    let onkeyup = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(
    move |event: web_sys::KeyboardEvent| {
        //
        let mut key_states_borrow = event_key_states.borrow_mut();
        //
        let mut keys_just_changed_borrow = event_keys_just_changed.borrow_mut();
        //
        key_states_borrow.insert(event.key(), rhai_api::KeyState { is_held: false, just_pressed: false, just_released: true });
        //
        keys_just_changed_borrow.push(event.key());
    });

    window().unwrap().document().unwrap().add_event_listener_with_callback("keyup", onkeyup.as_ref().unchecked_ref())?;
    

    //
    call_fn_on_all("create", (), &api_engine, &state_manager.script, 
    &cur_scene.script, &object_stack, &key_states, &keys_just_changed)?;
    
    //
    let project_config = api_engine.parse_json(
    &data::get_metadata_config(1),false).unwrap();
    //
    let canvas_width = rhai_api::dynamic_to_number(&project_config["canvas-width"]).unwrap() as i32;
    //
    let canvas_height = rhai_api::dynamic_to_number(&project_config["canvas-height"]).unwrap() as i32;

    //
    let (gl, gl_program,
        program_data,
        buffer) = create_rendering_components(canvas_width, canvas_height)?;
    //
    let cur_scene_map = Rc::clone(&cur_scene.map.0);
    //
    let draw_loop = Rc::new(RefCell::new(
    None::<Closure::<dyn FnMut(f64) -> Result<(), JsValue>>>));
    //
    let draw_init = Rc::clone(&draw_loop);
    //
    let mut last_draw = window().unwrap().performance().unwrap().now();
    //
    *draw_init.borrow_mut() = Some(Closure::<dyn FnMut(f64) -> Result<(), JsValue>>::new(
    move |draw_time: f64| -> Result<(), JsValue> {
        //
        let elapsed = draw_time - last_draw;
        last_draw = draw_time;

        //
        renderer::render_scene(
            &gl, &gl_program, &program_data, &buffer,
            &cur_scene_map.borrow()
                .read_lock::<entity::Scene>()
                .expect("read_lock cast should succeed")
        )?;
        //
        window().unwrap().request_animation_frame(draw_loop
            .borrow()
            .as_ref()
            .unwrap()
            .as_ref()
            .unchecked_ref()
        ).or_else(|js| {
            //
            return Err(js
                .as_string()
                .unwrap_or(String::from("Unknown error occurred while rendering the game."))
            );
        })?;
        //
        Ok(())
    }));

    //
    window().unwrap().request_animation_frame(draw_init
        .borrow()
        .as_ref()
        .unwrap()
        .as_ref()
        .unchecked_ref()
    ).or_else(|js| {
        return Err(js
            .as_string()
            .unwrap_or(String::from("Unknown error occurred while rendering the game."))
        );
    })?;

    //
    let frame_rate = rhai_api::dynamic_to_number(&project_config["fps"]).unwrap() as i32;
    //
    let frame_time = 1000 / frame_rate;
    //
    let update_loop = Closure::<dyn Fn() -> Result<(), JsValue>>::new(
    move || -> Result<(), JsValue> {
        //
        call_fn_on_all("update", (frame_time as rhai::FLOAT, ), &api_engine,
        &state_manager.script, &cur_scene.script, &object_stack, &key_states, &keys_just_changed)?;
        //
        Ok(())
    });
    //
    let inter_id = window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(
        update_loop.as_ref().unchecked_ref(),
        frame_time
    ).or_else(|js| {
        return Err(js
            .as_string()
            .unwrap_or(String::from("Unknown error occurred while running the game."))
        );
    })?;

    // Done!
    Ok(ClosuresHandle { 
        interval_id: inter_id,
        _interval: update_loop,
        _keydown: onkeydown,
        _keyup: onkeyup
    })
}



//
fn call_fn_on_all(name: &str, args: impl rhai::FuncArgs + Clone, engine: &Engine,
manager: &Rc<RefCell<rhai_api::EntityScript>>, scene: &Rc<RefCell<rhai_api::EntityScript>>,
object_stack: &Rc<RefCell<Vec<rhai_api::Entity<rhai_api::Object>>>>,
key_states: &Rc<RefCell<HashMap<String, rhai_api::KeyState>>>,
keys_just_changed: &Rc<RefCell<Vec<String>>>) -> Result<(), String> {
    // Call the function on the state manager instance.
    manager.borrow_mut().call_fn(engine, name, args.clone())?;
    //
    scene.borrow_mut().call_fn(engine, name, args.clone())?;
    //
    let mut i = 0_usize;
    loop {
        //
        if i >= object_stack.borrow().len() {
            break;
        }
        //
        if object_stack.borrow().get(i).unwrap().map.0.borrow()
        .read_lock::<entity::Object>().expect("read_lock cast should succeed")
        .active == true {
            //
            let object = Rc::clone( 
                &object_stack.borrow().get(i).unwrap().script
            );
            //
            object.borrow_mut().call_fn(engine, name, args.clone())?;
        }
        //
        i += 1;
    }
    //
    let mut key_states_borrow = key_states.borrow_mut();
    //
    let mut keys_just_changed_borrow = keys_just_changed.borrow_mut();
    //
    for key in keys_just_changed_borrow.clone() {
        //
        key_states_borrow.get_mut(&key)
        .expect("key should exist if it's inside the keys_just_changed vector")
        .just_pressed = false;
        //
        key_states_borrow.get_mut(&key)
        .expect("key should exist if it's inside the keys_just_changed vector")
        .just_released = false;
    }
    //
    keys_just_changed_borrow.clear();
    //
    Ok(())
}



//
fn create_rendering_components(canvas_width: i32, canvas_height: i32)
 -> Result<(web_sys::WebGlRenderingContext, web_sys::WebGlProgram,
HashMap<String, renderer::ProgramDataLocation>, web_sys::WebGlBuffer), String> {
    //
    let gl = renderer::create_context(
        canvas_width as i32, 
        canvas_height as i32
    ).or_else(|js| {
        return Err(js
            .as_string()
            .unwrap_or(String::from("Rendering Error: Couldn't create WebGL context."))
        );
    })?;
    //
    let (gl_program, 
        program_data) = renderer::create_scene_rendering_program(&gl
    ).or_else(|js| {
        return Err(js
            .as_string()
            .unwrap_or(String::from("Rendering Error: Couldn't create the scene rendering shader program."))
        );
    })?;
    // 
    let buffer = gl
        .create_buffer()
        .ok_or(String::from("failed to create buffer"))?;
    //
    Ok((gl, gl_program, program_data, buffer))
}