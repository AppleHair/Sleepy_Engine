
use std::{collections::HashMap, cell::RefCell, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::window;
use rhai::Engine;

use crate::data;

mod engine_api;
mod renderer;

//
#[wasm_bindgen]
pub struct ClosuresHandle {
    _keydown: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
    _keyup: Closure::<dyn Fn(web_sys::KeyboardEvent)>,
}

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

impl Default for TableRow {
    fn default() -> Self { Self::Element(Default::default(), Default::default()) }
}

//
pub fn run_game() -> Result<ClosuresHandle, JsValue>
{
    // Create the element definitions hash map.
    let mut element_defs: HashMap<u32,Result<Rc<engine_api::ElementDefinition>, String>> = HashMap::new();
    //  Create the asset definitions hash map.
    let asset_defs: Rc<RefCell<HashMap<u32,Result<renderer::AssetDefinition, JsValue>>>> = Rc::new(RefCell::new(HashMap::new()));

    // Create the API 'Engine', and the state manager instance.
    let (engine, state_manager, 
    cur_scene, cur_scene_id,
    prv_scene_id, object_stack,
    key_states) = engine_api::create_api(&mut element_defs)?;
    //
    load_elements(&engine, &mut element_defs, true);

    //
    let keys_just_changed: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    //
    let (onkeydown,
    onkeyup) = 
        create_keyboard_listeners(&key_states, &keys_just_changed)?;

    //
    call_fn_on_all("create", (), &engine, &state_manager.handler, 
    &cur_scene.handler, &object_stack)?;

    //
    let canvas_width = engine_api::dynamic_to_number(&state_manager.handler
        .borrow().definition.config["canvas-width"]).unwrap() as i32;
    //
    let canvas_height = engine_api::dynamic_to_number(&state_manager.handler
        .borrow().definition.config["canvas-height"]).unwrap() as i32;
    //
    let (gl_context, gl_program,
    program_data,
    vertex_buffer, index_buffer) = 
        renderer::create_rendering_components(canvas_width, canvas_height)?;

    //
    load_assets(&engine, &mut asset_defs.borrow_mut(), &gl_context);
    
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
    let cur_scene_map = Rc::clone(&cur_scene.map);
    //
    let renderer_asset_defs = Rc::clone(&asset_defs);
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
        &mut renderer_asset_defs.borrow_mut(), &gl_context);

        //
        renderer::render_scene(
            &gl_context, &gl_program, &program_data, &vertex_buffer,
            &index_buffer, &cur_scene_map.borrow()
                .read_lock::<engine_api::element::Scene>()
                .expect("read_lock cast should succeed"),
            &renderer_asset_defs.borrow(),
            &renderer_object_stack.borrow(),
            elapsed
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
    let frame_rate = engine_api::dynamic_to_number(&state_manager.handler
        .borrow().definition.config["fps"]).unwrap() as u32;
    //
    let frame_rate_init = frame_rate.clone();
    //
    let mut last_update = window().unwrap().performance().unwrap().now();
    //
    *prv_scene_id.borrow_mut() = cur_scene_id.borrow().clone();
    //
    *update_init.borrow_mut() = Some(Closure::<dyn FnMut() -> Result<(), JsValue>>::new(
    move || -> Result<(), JsValue> {
        //
        let update_time = window().unwrap().performance().unwrap().now();
        let elapsed = update_time - last_update;
        last_update = update_time;
        //
        call_fn_on_all("update", (elapsed as rhai::FLOAT, ), &engine,
        &state_manager.handler, &cur_scene.handler, &object_stack)?;
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
            switch_scene(cur_scene_id.borrow().clone(), &engine,
            &cur_scene, &object_stack, &element_defs)?;
            //
            call_fn_on_all("create", (), &engine,
            &state_manager.handler, &cur_scene.handler, &object_stack)?;
            //
            *prv_scene_id.borrow_mut() = cur_scene_id.borrow().clone();
        }

        //
        load_elements(&engine, &mut element_defs, false);

        //
        set_timeout_with_callback_and_f64(
            update_loop
                .borrow().as_ref().unwrap()
                .as_ref().unchecked_ref(),
            1000_f64 / (frame_rate as f64)
        );
        //
        Ok(())
    }));

    //
    set_timeout_with_callback_and_f64(
        update_init
            .borrow().as_ref().unwrap()
            .as_ref().unchecked_ref(),
        1000_f64 / (frame_rate_init as f64)
    );

    //
    window().unwrap().request_animation_frame(
        draw_init
            .borrow().as_ref().unwrap()
            .as_ref().unchecked_ref()
    )?;

    // Done!
    Ok(ClosuresHandle {
        _keydown: onkeydown,
        _keyup: onkeyup
    })
}

//
fn create_keyboard_listeners(key_states: &Rc<RefCell<HashMap<String, engine_api::KeyState>>>, 
keys_just_changed: &Rc<RefCell<Vec<String>>>) -> Result<(Closure<dyn Fn(web_sys::KeyboardEvent)>,
Closure<dyn Fn(web_sys::KeyboardEvent)>), JsValue> {
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
        key_states_borrow.insert(event.code(), engine_api::KeyState {
            is_held: true, just_pressed: true, just_released: false
        });
        //
        keys_just_changed_borrow.push(event.code());
    });

    window()
    .unwrap().document()
    .unwrap().add_event_listener_with_callback("keydown", onkeydown.as_ref().unchecked_ref())?;

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
    Ok((onkeydown, onkeyup))
}

//
fn call_fn_on_all(name: &str, args: impl rhai::FuncArgs + Clone, engine: &Engine,
manager: &Rc<RefCell<engine_api::ElementHandler>>, scene: &Rc<RefCell<engine_api::ElementHandler>>,
object_stack: &Rc<RefCell<Vec<engine_api::Element<engine_api::Object>>>>) -> Result<(), String> {
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
        if object_stack.borrow().get(i)
        .unwrap().map.borrow().read_lock::<engine_api::element::Object>()
        .expect("read_lock cast should succeed").active == true {
            //
            let object = Rc::clone( 
                &object_stack
                .borrow().get(i)
                .unwrap().handler
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
fn load_assets(engine: &Engine, asset_defs: &mut HashMap<u32,Result<renderer::AssetDefinition, JsValue>>,
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
fn load_elements(engine: &Engine, element_defs: &mut HashMap<u32,Result<Rc<engine_api::ElementDefinition>, String>>
, init: bool) {
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
fn switch_scene(scene_id: u32, engine: &Engine, scene: &engine_api::Element<engine_api::Scene>,
object_stack: &Rc<RefCell<Vec<engine_api::Element<engine_api::Object>>>>,
element_defs: &HashMap<u32,Result<Rc<engine_api::ElementDefinition>, String>>) -> Result<(), String> {
    //
    scene.recycle_scene(&engine, 
        Rc::clone(
            element_defs.get(&scene_id)
            .expect("element_defs.get(&scene_id) should have had the scene's definition by now").as_ref()?
        )
    )?;
    //
    let instances = scene.handler.borrow().definition.config["object-instances"].clone()
    .into_typed_array::<rhai::Map>().expect(concat!("Every object's config should contain a 'object-instances'",
    " array, which should only have object-like members."));
    //
    let mut object_stack_borrow = object_stack.borrow_mut();
    //
    if instances.len() > object_stack_borrow.len() {
        //
        object_stack_borrow.resize_with(instances.len(), || { Default::default() });
    }
    //
    if instances.len() < object_stack_borrow.len() {
        //
        for idx in instances.len()..object_stack_borrow.len() {
            //
            let object_borrow = object_stack_borrow.get(idx)
            .expect("range should be correct.");
            //
            object_borrow.handler.borrow_mut().definition = Default::default();
            //
            object_borrow.map.borrow_mut().write_lock::<engine_api::element::Object>()
            .expect("write_lock cast should succeed").active = false;
        }
    }

    //
    let mut i = 0_usize;
    //
    let layers_len = scene.map.borrow().read_lock::<engine_api::element::Scene>()
    .expect("read_lock cast should succeed").layers_len;
    //
    for layer in scene.map.borrow().read_lock::<engine_api::element::Scene>()
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
            let object_borrow = object_stack_borrow.get_mut(idx as usize)
            .expect("The indexes specified in every element of every layer's instances array should be correct.");
            //
            if object_borrow.map.borrow().is_unit() {
                //
                *object_borrow = engine_api::Element::new_object(&engine,
                    Rc::clone(
                        element_defs.get(&ent_id)
                        .expect("element_defs.get(&ent_id) should have had the object's definition by now").as_ref()?
                    ), (idx, i, j, init_x, init_y)
                )?;
                //
                j += 1;
                continue;
            }
            //
            object_borrow.recycle_object(&engine,
                Rc::clone(
                    element_defs.get(&ent_id)
                    .expect("element_defs.get(&ent_id) should have had the object's definition by now").as_ref()?
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