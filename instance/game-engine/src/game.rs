
use crate::data;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::window;
use rhai::Engine;

mod entity;
mod engine_api;
mod renderer;



#[wasm_bindgen]
pub struct ClosuresHandle {
    interval_id: i32,
    _interval: Closure::<dyn FnMut() -> Result<(), JsValue>>,
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
    let mut object_defs: HashMap<u32,Rc<engine_api::EntityDefinition>> = HashMap::new();

    //
    let keys_just_changed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    // Create the API 'Engine', and the state manager instance.
    let (api_engine, state_manager, 
        cur_scene, cur_scene_id,
        prv_scene_id, object_stack,
        key_states) = engine_api::create_api(&mut object_defs)?;
    
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
        key_states_borrow.insert(event.key(), engine_api::KeyState { is_held: true, just_pressed: true, just_released: false });
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
        key_states_borrow.insert(event.key(), engine_api::KeyState { is_held: false, just_pressed: false, just_released: true });
        //
        keys_just_changed_borrow.push(event.key());
    });

    window().unwrap().document().unwrap().add_event_listener_with_callback("keyup", onkeyup.as_ref().unchecked_ref())?;
    

    //
    call_fn_on_all("create", (), &api_engine, &state_manager.script, 
    &cur_scene.script, &object_stack)?;

    //
    let project_config = api_engine.parse_json(
    &data::get_metadata_config(1),false).unwrap();
    //
    let canvas_width = engine_api::dynamic_to_number(&project_config["canvas-width"]).unwrap() as i32;
    //
    let canvas_height = engine_api::dynamic_to_number(&project_config["canvas-height"]).unwrap() as i32;

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
    let frame_rate = engine_api::dynamic_to_number(&project_config["fps"]).unwrap() as i32;
    //
    let mut last_update = window().unwrap().performance().unwrap().now();
    //
    *prv_scene_id.borrow_mut() = cur_scene_id.borrow().clone();
    //
    let update_loop = Closure::<dyn FnMut() -> Result<(), JsValue>>::new(
    move || -> Result<(), JsValue> {
        //
        let update_time = window().unwrap().performance().unwrap().now();
        let elapsed = update_time - last_update;
        last_update = update_time;
        //
        call_fn_on_all("update", (elapsed as rhai::FLOAT, ), &api_engine,
        &state_manager.script, &cur_scene.script, &object_stack)?;
        //
        {
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
        }        
        //
        if *prv_scene_id.borrow() != *cur_scene_id.borrow() {
            //
            switch_scene(cur_scene_id.borrow().clone(), &api_engine,
            &cur_scene, &object_stack, &mut object_defs)?;
            //
            call_fn_on_all("create", (), &api_engine,
            &state_manager.script, &cur_scene.script, &object_stack)?;
            //
            *prv_scene_id.borrow_mut() = cur_scene_id.borrow().clone();
        }
        //
        Ok(())
    });
    //
    let inter_id = window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(
        update_loop.as_ref().unchecked_ref(),
        1000 / frame_rate
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
manager: &Rc<RefCell<engine_api::EntityScript>>, scene: &Rc<RefCell<engine_api::EntityScript>>,
object_stack: &Rc<RefCell<Vec<engine_api::Entity<engine_api::Object>>>>) -> Result<(), String> {
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
    Ok(())
}

//
fn switch_scene(scene_id: u32, engine: &Engine, scene: &engine_api::Entity<engine_api::Scene>,
object_stack: &Rc<RefCell<Vec<engine_api::Entity<engine_api::Object>>>>,
object_defs: &mut HashMap<u32,Rc<engine_api::EntityDefinition>>) -> Result<(), String> {
    //
    object_defs.clear();
    //
    scene.recycle_scene(&engine, 
        Rc::new(
            engine_api::EntityDefinition::new(&engine, 
                engine_api::TableRow::Entity(scene_id, 2)
            )?
        )
    )?;
    //
    let instances = scene.script.borrow().definition.config["object-instances"].clone()
    .into_typed_array::<rhai::Map>().expect(concat!("Every object's config should contain a 'object-instances'",
    " array, which should only have object-like members."));
    //
    let mut object_stack_borrow = object_stack.borrow_mut();
    //
    if instances.len() > object_stack_borrow.len() {
        //
        object_stack_borrow.resize_with(instances.len(), || { engine_api::Entity { 
            map: engine_api::Object(Rc::new(RefCell::new(rhai::Dynamic::UNIT))), script: Default::default() } 
        });
    }
    //
    if instances.len() < object_stack_borrow.len() {
        //
        for idx in instances.len()..object_stack_borrow.len() {
            //
            let object_borrow = object_stack_borrow.get(idx).expect("Range was wrong.");
            //
            object_borrow.script.borrow_mut().definition = Default::default();
            //
            object_borrow.map.0.borrow_mut().write_lock::<entity::Object>()
            .expect("write_lock cast should succeed").active = false;
        }
    }

    //
    let mut i = 0_usize;
    //
    let layers_len = scene.map.0.borrow().read_lock::<entity::Scene>()
    .expect("read_lock cast should succeed").layers_len;
    //
    for layer in scene.map.0.borrow().read_lock::<entity::Scene>()
    .expect("read_lock cast should succeed").layers.clone() {
        //
        if i >= layers_len {
            //
            break;
        }
        //
        let mut j = 0_usize;
        //
        for idx in layer.instances {
            //
            let inst_info = instances.get(idx as usize)
            .expect("The indexes specified in every element of every layer's instances array should be correct.");
            //
            let ent_id = engine_api::dynamic_to_number(&inst_info["id"])
            .expect(concat!("Every instance in the 'object-instances' array of an object's",
            " config should contain an integer 'id' attribute.")) as u32;
            let (init_x, init_y) = (
                engine_api::dynamic_to_number(&inst_info["x"])
                .expect(concat!("Every instance in the 'object-instances' array of an object's",
                " config should contain an float 'x' attribute.")), 
                engine_api::dynamic_to_number(&inst_info["y"])
                .expect(concat!("Every instance in the 'object-instances' array of an object's",
                " config should contain an float 'y' attribute.")),
            );
            //
            if !object_defs.contains_key(&ent_id) {
                //
                object_defs.insert(ent_id, 
                    Rc::new(
                        engine_api::EntityDefinition::new(&engine, 
                            engine_api::TableRow::Entity(ent_id, 1)
                        )?
                    )
                );
            }
            //
            let object_borrow = object_stack_borrow.get_mut(idx as usize)
            .expect("The indexes specified in every element of every layer's instances array should be correct.");
            //
            if object_borrow.map.0.borrow().is_unit() {
                //
                *object_borrow = engine_api::Entity::new_object(&engine,
                    Rc::clone(
                        object_defs.get(&ent_id)
                        .expect("object_defs.get(&inst_id_u32) should have had the object's definition by now")
                    ), (idx, i, j, init_x, init_y)
                )?;
                //
                j += 1;
                continue;
            }
            //
            object_borrow.recycle_object(&engine,
                Rc::clone(
                    object_defs.get(&ent_id)
                    .expect("object_defs.get(&inst_id_u32) should have had the object's definition by now")
                ), (i, j, init_x, init_y)
            )?;
            //
            j += 1;
        }
        //
        i += 1;
    }
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