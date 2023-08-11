
use crate::webapp;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use rhai::{Engine, Scope, AST, Map, packages::Package, packages::StandardPackage, EvalAltResult, Dynamic};

mod entity;
mod rhai_convert;

//
pub enum PostRuntimeTask {
    DeactivateObject(rhai::INT),
    ActivateObject(rhai::INT),
    InstanceSwitchLayer(rhai::INT, rhai::INT, rhai::INT, rhai::Position),
}

//
#[derive(Clone)]
pub enum EntityRow {
    Manager,
    Scene(u32),
    Object(u32),
}

//
struct EntityDefinition {
    row: EntityRow,
    script: AST,
    config: Map,
}

//
impl EntityDefinition {
    //
    fn new(engine: &Engine, row: EntityRow) -> Result<Self, (EntityRow, Box<EvalAltResult>)> {
        //
        let ast = engine.compile(&match row {
            EntityRow::Manager => webapp::getGameScript(),
            EntityRow::Object(id) => webapp::getEntityScript(id),
            EntityRow::Scene(id) => webapp::getEntityScript(id),
        });
        //
        if ast.is_err() {
            //
            return Err((row, ast.unwrap_err().into()));
        }
        //
        let json = engine.parse_json(&match row {
            EntityRow::Manager => webapp::getGameConfig(),
            EntityRow::Object(id) => webapp::getEntityConfig(id),
            EntityRow::Scene(id) => webapp::getEntityConfig(id),
        }, false);
        //
        if json.is_err() {
            //
            return Err((row, json.unwrap_err()));
        }
        //
        let def = Self {
            //
            row,
            //
            script: ast.expect("This Err should have been caught by this function beforehand"),
            //
            config: json.expect("This Err should have been caught by this function beforehand"),
        };
        //
        Ok(def)
    }
}

//
struct EntityInstance {
    definition: Rc<EntityDefinition>,
    scope: Scope<'static>,
}

//
impl EntityInstance {
    //
    fn new(engine: &Engine, def: Rc<EntityDefinition>, 
    obj_idx: Option<(usize, f64, f64)>) -> Result<Self, (EntityRow, Box<EvalAltResult>)> {
        //
        let mut inst = Self {
            //
            definition: def,
            //
            scope: Scope::new(),
        };
        //
        match (inst.definition.row.to_owned(), obj_idx) {
            //
            (EntityRow::Scene(_), None) => {
                //
                inst.scope.push("Scene", entity::create_scene(&inst.definition.config));
            },
            //
            (EntityRow::Object(_), Some((idx, init_x, init_y))) => {
                //
                inst.scope.push_constant("INDEX", idx as rhai::INT);
                //
                inst.scope.push_constant("ACTIVE", true);
                //
                inst.scope.push("Object", entity::create_object(&inst.definition.config, init_x, init_y));
            }
            //
            (EntityRow::Manager, None) => {},
            //
            _ => { return Err((inst.definition.row.to_owned(), "Couldn't find the right resources for the entity.".into()))},
        }
        //
        let err = engine.run_ast_with_scope(&mut inst.scope, &inst.definition.script).err();
        //
        if err.is_some() {
            //
            return Err((inst.definition.row.to_owned(), err.unwrap()));
        }
        //
        Ok(inst)
    }

    //
    fn call_fn(&mut self, engine: &Engine, name: &str, args: impl rhai::FuncArgs) -> Result<(), (EntityRow, Box<EvalAltResult>)> 
    {
        //
        let err = engine.call_fn::<()>(&mut self.scope, &self.definition.script, name, args).err();
        //
        if err.is_some() {
            //
            return Err((self.definition.row.to_owned(), err.unwrap()));
        }
        //
        Ok(())
    }
}

//
fn create_api(entity_defs: &mut HashMap<u32,Rc<EntityDefinition>>) -> Result<(Engine, Rc<RefCell<EntityInstance>>,
Rc<RefCell<EntityInstance>>, u32, Rc<RefCell<Vec<Rc<RefCell<EntityInstance>>>>>, Rc<RefCell<Vec<PostRuntimeTask>>>),
(EntityRow, Box<EvalAltResult>)> {
    // Create an 'Engine'
    let mut engine = Engine::new_raw();

    // Register API features to the 'Engine'
    engine.on_print(|text| { webapp::log(text); })
          .register_type_with_name::<entity::PositionPoint>("Position")
          .register_get_set("x", entity::PositionPoint::get_x, entity::PositionPoint::set_x)
          .register_set("x", entity::PositionPoint::set_x_rhai_int)
          .register_set("x", entity::PositionPoint::set_x_rhai_float)
          .register_get_set("y", entity::PositionPoint::get_y, entity::PositionPoint::set_y)
          .register_set("y", entity::PositionPoint::set_y_rhai_int)
          .register_set("y", entity::PositionPoint::set_y_rhai_float)
          .register_fn("to_string", entity::PositionPoint::to_string)
          .register_type_with_name::<entity::CollisionBox>("CollisionBox")
          .register_get("point1", entity::CollisionBox::get_point1)
          .register_get("point2", entity::CollisionBox::get_point2)
          .register_fn("to_string", entity::CollisionBox::to_string)
          .register_type_with_name::<entity::Object>("Object")
          .register_get_set("position", entity::Object::get_position, entity::Object::set_position)
          .register_get("origin_offset", entity::Object::get_origin_offset)
          .register_get("collision_boxes", entity::Object::get_collision_boxes)
          .register_fn("to_string", entity::Object::to_string)
          .register_type_with_name::<entity::Camera>("Camera")
          .register_get_set("position", entity::Camera::get_position, entity::Camera::set_position)
          .register_get_set("zoom", entity::Camera::get_zoom, entity::Camera::set_zoom)
          .register_set("zoom", entity::Camera::set_zoom_rhai_int)
          .register_set("zoom", entity::Camera::set_zoom_rhai_float)
          .register_fn("to_string", entity::Camera::to_string)
          .register_type_with_name::<entity::Layer>("Layer")
          .register_get("name", entity::Layer::get_name)
          .register_get("instances", entity::Layer::get_instances)
          .register_fn("to_string", entity::Layer::to_string)
          .register_type_with_name::<entity::Scene>("Scene")
          .register_get_set("width", entity::Scene::get_width, entity::Scene::set_width)
          .register_set("width", entity::Scene::set_width_rhai_int)
          .register_set("width", entity::Scene::set_width_rhai_float)
          .register_get_set("height", entity::Scene::get_height, entity::Scene::set_height)
          .register_set("height", entity::Scene::set_height_rhai_int)
          .register_set("height", entity::Scene::set_height_rhai_float)
          .register_get_set("in_color", entity::Scene::get_inside_color, entity::Scene::set_inside_color)
          .register_get_set("out_color", entity::Scene::get_outside_color, entity::Scene::set_outside_color)
          .register_get_set("camera", entity::Scene::get_camera, entity::Scene::set_camera)
          .register_get("stack_len", entity::Scene::get_stack_len)
          .register_get("layers", entity::Scene::get_layers)
          .register_fn("to_string", entity::Scene::to_string);

    // Register the Standard Package
    let package = StandardPackage::new();
    // Load the package into the 'Engine'
    package.register_into_engine(&mut engine);

    // Insert a new entity definition into the hash map.
    // This definition will define the state manager, and
    // will store its script and config data.
    entity_defs.insert(0, 
        Rc::new(
            EntityDefinition::new(&engine, 
                EntityRow::Manager
            )?
        )
    );
    // Create a new entity instance for the state manager.
    // This instance will borrow its definition and contain
    // the entity's 'Scope'.
    let state_manager: Rc<RefCell<EntityInstance>> = Rc::new(
        RefCell::new(
            EntityInstance::new(&engine, 
                Rc::clone(
                    entity_defs.get(&0)
                    .expect("entity_defs.get(&0) should have had the state manager's definition by now")
                ), None
            )?
        )
    );

    // Take the 'State' object map from 
    // the state manager and turn it into
    // a shared variable.
    let state_map = state_manager.borrow_mut().scope.remove::<Dynamic>("State").unwrap_or(Dynamic::UNIT).into_shared();
    // If it's not a ()
    if !state_map.is_unit() {
        // Push it back into the state manager
        // as a shared object map variable.
        state_manager.borrow_mut().scope.push("State", state_map.clone());
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
                        Ok(Some(state_map.flatten_clone())) 
                    }
                },
                // Otherwise, continue with the normal variable resolution process.
                _ => Ok(None)
            }
        });
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
    }

    // Receive the rowid of the initial scene from the the state manager's config
    let mut cur_scene_id = state_manager.borrow().definition.config["initial-scene"].as_int()
    .expect("The value of 'initial-scene' in the state manager's config should be an integer") as u32;
    //
    entity_defs.insert(cur_scene_id, 
        Rc::new(
            EntityDefinition::new(&engine, 
                EntityRow::Scene(cur_scene_id)
            )?
        )
    );
    //
    let cur_scene = Rc::new(
        RefCell::new(
            EntityInstance::new(&engine, 
                Rc::clone(
                    entity_defs.get(&cur_scene_id)
                    .expect("entity_defs.get(&scene_id) should have had the scene's definition by now")
                ), None
            )?
        )
    );
    //
    let object_stack: Rc<RefCell<Vec<Rc<RefCell<EntityInstance>>>>> = Rc::new(RefCell::new(Vec::new()));

    //
    let mut i = 0_usize;
    //
    for inst_info in cur_scene.borrow().definition.config["object-instances"].clone()
    .into_typed_array::<Map>().expect(concat!("Every object's config should contain a 'object-instances'",
    " array, which should only have object-like members.")) {
        //
        let inst_id = rhai_convert::dynamic_to_number(&inst_info["id"])
        .expect("Every instance in the 'object-instances' array of an object's config should contain an integer 'id' attribute.")
        as u32;
        let (init_x, init_y) = (
            rhai_convert::dynamic_to_number(&inst_info["x"])
            .expect("Every instance in the 'object-instances' array of an object's config should contain an float 'x' attribute."), 
            rhai_convert::dynamic_to_number(&inst_info["y"])
            .expect("Every instance in the 'object-instances' array of an object's config should contain an float 'y' attribute."),
        );
        //
        entity_defs.insert(inst_id, 
            Rc::new(
                EntityDefinition::new(&engine, 
                    EntityRow::Object(inst_id)
                )?
            )
        );
        //
        object_stack.borrow_mut().push(Rc::new( 
                RefCell::new(
                    EntityInstance::new(&engine,
                        Rc::clone(
                            entity_defs.get(&inst_id)
                            .expect("entity_defs.get(&inst_id_u32) should have had the object's definition by now")
                        ), Some((i, init_x, init_y))
                    )?
                )
            )
        );
        //
        i += 1;
    }

    //
    i = 0_usize;
    //
    for layer in cur_scene.borrow().scope.get_value::<entity::Scene>("Scene")
    .expect("The Scene map should have been created in this scene's scope by the time it was created")
    .layers {
        //
        let mut j: rhai::INT = 0;
        //
        for idx in layer.instances {
            //
            Rc::clone(object_stack.borrow().get(idx as usize).unwrap())
            .borrow_mut().scope.push_constant("LAYER", i as rhai::INT)
            .push_constant("LAYER_INDEX", j);
            //
            j += 1;
        }
        //
        i += 1;
    }
    
    //
    let post_runtime_tasks: Rc<RefCell<Vec<PostRuntimeTask>>> = Rc::new(RefCell::new(Vec::new()));

    // Register API closures, which need to 
    // capture the game-loop's environment
    let api_cur_scene = Rc::clone(&cur_scene);
    engine.register_fn("get_cur_scene", move || -> Result<entity::Scene, Box<EvalAltResult>> {
        //
        let cur_scene_borrow = api_cur_scene.try_borrow();
        //
        if cur_scene_borrow.is_err() {
            //
            return Err(concat!("Can't use the 'get_cur_scene' function inside a scene",
            " script! Note: Use the 'Scene' map instead.").into());
        }
        //
        Ok(cur_scene_borrow.unwrap().scope.get_value::<entity::Scene>("Scene")
        .expect("The 'Scene' map should have been created in this scene's scope by the time it was created"))
    });

    //
    let api_cur_scene = Rc::clone(&cur_scene);
    engine.register_fn("is_cur_scene", move |name: &str| -> Result<bool, Box<EvalAltResult>> {
        //
        let cur_scene_borrow = api_cur_scene.try_borrow();
        //
        if cur_scene_borrow.is_err() {
            //
            return Err(concat!("Can't use the 'is_cur_scene' function inside a scene",
            " script! Note: This script only runs when its scene is the current scene,",
            " so you don't need to use this function inside this script.").into());
        }
        //
        Ok(if let EntityRow::Scene(id) = cur_scene_borrow.unwrap().definition.row {
            webapp::getEntityName(id) == name
        } else { false })
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    engine.register_fn("get_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<entity::Object, Box<EvalAltResult>> {
        //
        if idx >= (api_object_stack.borrow().len() as rhai::INT) {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(api_object_stack.borrow().len(), idx, context.position())));
        }
        //
        let object_reference = Rc::clone( 
            api_object_stack.borrow().get(idx as usize).unwrap()
        );
        //
        if object_reference.try_borrow().is_err() {
            //
            return Err(concat!("Can't use the 'get_object' function to get values from the current script's",
            " object! Note: Use the 'Object' map instead.").into());
        }
        //
        let object = object_reference.borrow().scope.get_value::<entity::Object>("Object")
        .expect("The 'Object' map should have been created in this object's scope by the time it was created");
        //
        Ok(object)
    });
    //
    let api_object_stack = Rc::clone(&object_stack);
    engine.register_fn("object_is_valid", move |idx: rhai::INT| -> bool {
        idx < (api_object_stack.borrow().len() as rhai::INT) && idx > -1
    });
    //
    let api_object_stack = Rc::clone(&object_stack);
    engine.register_fn("object_is_active", move |context: rhai::NativeCallContext, 
    idx: rhai::INT| -> Result<bool, Box<EvalAltResult>> {
        //
        if idx >= (api_object_stack.borrow().len() as rhai::INT) {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(api_object_stack.borrow().len(), idx, context.position())));
        }
        //
        let object_reference = Rc::clone( 
            api_object_stack.borrow().get(idx as usize).unwrap()
        );
        //
        if object_reference.try_borrow().is_err() {
            //
            return Err(concat!("Can't use the 'object_is_active' function to get values", 
            " from the current script's object! Note: Use the 'ACTIVE' constant instead.").into());
        }
        //
        let active = object_reference.borrow().scope.get_value::<bool>("ACTIVE")
        .expect("The 'ACTIVE' constant should have been created in this object's scope by the time it was created");
        //
        Ok(active)
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_post_runtime_tasks = Rc::clone(&post_runtime_tasks);
    engine.register_fn("activate_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<bool, Box<EvalAltResult>> {
        //
        if idx >= (api_object_stack.borrow().len() as rhai::INT) {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(api_object_stack.borrow().len(), idx, context.position())));
        }
        //
        let object_reference = Rc::clone( 
            api_object_stack.borrow().get(idx as usize).unwrap()
        );
        //
        if object_reference.try_borrow_mut().is_err() {
            //
            api_post_runtime_tasks.borrow_mut().push(PostRuntimeTask::ActivateObject(idx));
            //
            return Ok(false)
        }
        //
        let mut object_borrow = object_reference.borrow_mut();
        //
        let _ = object_borrow.scope.remove::<bool>("ACTIVE");
        //
        object_borrow.scope.push_constant("ACTIVE", true);
        //
        Ok(true)
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    let api_post_runtime_tasks = Rc::clone(&post_runtime_tasks);
    engine.register_fn("deactivate_object", move |context: rhai::NativeCallContext,
    idx: rhai::INT| -> Result<bool, Box<EvalAltResult>> {
        //
        if idx >= (api_object_stack.borrow().len() as rhai::INT) {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(api_object_stack.borrow().len(), idx, context.position())));
        }
        //
        let object_reference = Rc::clone( 
            api_object_stack.borrow().get(idx as usize).unwrap()
        );
        //
        if object_reference.try_borrow_mut().is_err() {
            //
            api_post_runtime_tasks.borrow_mut().push(PostRuntimeTask::DeactivateObject(idx));
            //
            return Ok(false)
        }
        //
        let mut object_borrow = object_reference.borrow_mut();
        //
        let _ = object_borrow.scope.remove::<bool>("ACTIVE");
        //
        object_borrow.scope.push_constant("ACTIVE", false);
        //
        Ok(true)
    });

    //
    let api_state_manager = Rc::clone(&state_manager);
    engine.register_fn("message_state_manager", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if api_state_manager.try_borrow_mut().is_err() {
            //
            return Err(concat!("Can't use the 'message_state_manager' function while the state manager's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an entity, which tried to message you back in the scope of that same message.").into());
        }
        //
        let err = api_state_manager.borrow_mut().call_fn(context.engine(), 
        &format!("message_{}", name), args).err();
        //
        if let Some((EntityRow::Manager, evalres)) = err {
            return Err(format!("{}\non the script of 'State Manager',\nas a result of a call to 'message_state_manager'",
            evalres.to_string()).into());
        }
        //
        Ok(())
    });

    //
    let api_cur_scene = Rc::clone(&cur_scene);
    engine.register_fn("message_cur_scene", move |context: rhai::NativeCallContext,
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if api_cur_scene.try_borrow_mut().is_err() {
            //
            return Err(concat!("Can't use the 'message_cur_scene' function while the current scene's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an entity, which tried to message you back in the scope of that same message.").into());
        }
        //
        let err = api_cur_scene.borrow_mut().call_fn(context.engine(), 
        &format!("message_{}", name), args).err();
        //
        if let Some((EntityRow::Scene(id), evalres)) = err {
            return Err(format!("{}\non the script of the '{name}' scene,\nas a result of a call to 'message_cur_scene'",
            evalres.to_string(), name = webapp::getEntityName(id)).into());
        }
        //
        Ok(())
    });

    //
    let api_object_stack = Rc::clone(&object_stack);
    engine.register_fn("message_object", move |context: rhai::NativeCallContext, idx: rhai::INT, 
    name: &str, args: rhai::Array| -> Result<(), Box<EvalAltResult>> {
        //
        if idx >= (api_object_stack.borrow().len() as rhai::INT) {
            //
            return Err(Box::new(EvalAltResult::ErrorArrayBounds(api_object_stack.borrow().len(), idx, context.position())));
        }
        //
        let object_reference = Rc::clone( 
            api_object_stack.borrow().get(idx as usize).unwrap()
        );
        //
        if object_reference.try_borrow_mut().is_err() {
            //
            return Err(concat!("Can't use the 'message_object' function while that object's script is running",
            " (is handling another callback). Note: This might have happened because you tried to message yourself,",
            " or messaged an entity, which tried to message you back in the scope of that same message.").into());
        }
        //
        let err = object_reference.borrow_mut().call_fn(context.engine(), 
        &format!("message_{}", name), args).err();
        //
        if let Some((EntityRow::Object(id), evalres)) = err {
            return Err(format!("{}\non the script of the '{name}' object,\nas a result of a call to 'message_object'",
            evalres.to_string(), name = webapp::getEntityName(id)).into());
        }
        //
        Ok(())
    });

    //
    let api_post_runtime_tasks = Rc::clone(&post_runtime_tasks);
    engine.register_fn("instance_switch_layer", move |context: rhai::NativeCallContext,
    layer_from: rhai::INT, layer_idx_from: rhai::INT, layer_to: rhai::INT| {
        //
        api_post_runtime_tasks.borrow_mut().push(
            PostRuntimeTask::InstanceSwitchLayer(layer_from, layer_idx_from, layer_to, context.position())
        );
    });

    // //
    // let api_object_stack = Rc::clone(&object_stack);
    // engine.register_fn("add_object_to_stack", move |context: rhai::NativeCallContext,
    // idx: rhai::INT| -> rhai::INT {});

    // The API is done!
    Ok((engine, state_manager, cur_scene, cur_scene_id, object_stack, post_runtime_tasks))
}

//
fn handle_post_runtime_tasks(post_runtime_tasks: Rc<RefCell<Vec<PostRuntimeTask>>>,
object_stack: Rc<RefCell<Vec<Rc<RefCell<EntityInstance>>>>>, cur_scene: Rc<RefCell<EntityInstance>>,
fn_name: &str, ent_row: EntityRow) -> Result<(), (EntityRow, Box<EvalAltResult>)> {
    //
    post_runtime_tasks.borrow_mut().reverse();
    //
    loop {
        //
        match post_runtime_tasks.borrow_mut().pop() {
            //
            None => { break Ok(()); },
            //
            Some(PostRuntimeTask::ActivateObject(idx)) => {
                //
                let object_reference = Rc::clone( 
                    object_stack.borrow().get(idx as usize).unwrap()
                );
                //
                let mut object_borrow = object_reference.borrow_mut();
                //
                let _ = object_borrow.scope.remove::<bool>("ACTIVE");
                //
                object_borrow.scope.push_constant("ACTIVE", true);
            },
            //
            Some(PostRuntimeTask::DeactivateObject(idx)) => {
                //
                let object_reference = Rc::clone( 
                    object_stack.borrow().get(idx as usize).unwrap()
                );
                //
                let mut object_borrow = object_reference.borrow_mut();
                //
                let _ = object_borrow.scope.remove::<bool>("ACTIVE");
                //
                object_borrow.scope.push_constant("ACTIVE", false);
            },
            //
            Some(PostRuntimeTask::InstanceSwitchLayer(layer_from, layer_idx_from, layer_to, position)) => {
                //
                let mut cur_scene_borrow = cur_scene.borrow_mut();
                //
                let mut cur_scene_map = cur_scene_borrow.scope.get_value::<entity::Scene>("Scene")
                .expect("The 'Scene' map should have been created in this scene's scope by the time it was created");
                //
                let moving_stack_index: usize;
                //
                let to_update_stack_index: usize;
                //
                let new_layer_idx: rhai::INT;
                //
                if let Some(layer) = cur_scene_map.layers.get_mut(layer_from as usize) {
                    if let Some(_) = layer.instances.get(layer_idx_from as usize) {
                        //
                        to_update_stack_index = layer.instances.last().unwrap().clone() as usize;
                        //
                        moving_stack_index = layer.instances.swap_remove(layer_idx_from as usize) as usize;
                    } else {
                        //
                        break Err(
                            (ent_row, Box::new(EvalAltResult::ErrorInFunctionCall(String::from(fn_name), String::new(),
                            format!("Argument 'layer_idx_from' was out of bounds in call to 'instance_switch_layer'.")
                            .into(), position)))
                        );
                    }
                } else {
                    //
                    break Err(
                        (ent_row, Box::new(EvalAltResult::ErrorInFunctionCall(String::from(fn_name), String::new(),
                        format!("Argument 'layer_from' was out of bounds in call to 'instance_switch_layer'.")
                        .into(), position)))
                    );
                }
                //
                if let Some(layer) = cur_scene_map.layers.get_mut(layer_to as usize) {
                    //
                    layer.instances.push(moving_stack_index as u32);
                    //
                    new_layer_idx = layer.instances.len() as rhai::INT;
                } else {
                    //
                    break Err(
                        (ent_row, Box::new(EvalAltResult::ErrorInFunctionCall(String::from(fn_name), String::new(),
                        format!("Argument 'layer_to' was out of bounds in call to 'instance_switch_layer'.")
                        .into(), position)))
                    );
                }

                //
                let moving_object_reference = Rc::clone( 
                    object_stack.borrow().get(moving_stack_index)
                    .expect("The indexes specified in every element of every layer's instances array should be correct.")
                );
                //
                if moving_stack_index != to_update_stack_index {
                    //
                    let to_update_object_reference = Rc::clone( 
                        object_stack.borrow().get(to_update_stack_index)
                        .expect("The indexes specified in every element of every layer's instances array should be correct.")
                    );
                    //
                    let mut to_update_object_borrow = to_update_object_reference.borrow_mut(); 
                    //
                    let _ = to_update_object_borrow.scope.remove::<rhai::INT>("LAYER_INDEX");
                    //
                    to_update_object_borrow.scope.push_constant("LAYER_INDEX", layer_idx_from);
                }
                //
                let mut moving_object_borrow = moving_object_reference.borrow_mut(); 
                //
                let _ = moving_object_borrow.scope.remove::<rhai::INT>("LAYER");
                //
                let _ = moving_object_borrow.scope.push_constant("LAYER", layer_to)
                .remove::<rhai::INT>("LAYER_INDEX");
                //
                moving_object_borrow.scope.push_constant("LAYER_INDEX", new_layer_idx);
                //
                cur_scene_borrow.scope.set_or_push("Scene", cur_scene_map);
            }
        }
    }
}

//
fn call_fn_on_all(name: &str, args: impl rhai::FuncArgs + Clone, engine: &Engine, manager: &Rc<RefCell<EntityInstance>>, 
scene: &Rc<RefCell<EntityInstance>>, object_stack: &Rc<RefCell<Vec<Rc<RefCell<EntityInstance>>>>>,
post_runtime_tasks: &Rc<RefCell<Vec<PostRuntimeTask>>>) -> Result<(), (EntityRow, Box<EvalAltResult>)> {
    //
    let mut cur_row: EntityRow;
    // Call the function on the state manager instance.
    manager.borrow_mut().call_fn(engine, name, args.clone())?;
    //
    cur_row = manager.borrow().definition.row.clone();
    //
    handle_post_runtime_tasks(Rc::clone(&post_runtime_tasks),
    Rc::clone(&object_stack), Rc::clone(&scene),
    &name, cur_row)?;
    //
    scene.borrow_mut().call_fn(engine, name, args.clone())?;
    //
    cur_row = scene.borrow().definition.row.clone();
    //
    handle_post_runtime_tasks(Rc::clone(&post_runtime_tasks),
    Rc::clone(&object_stack), Rc::clone(&scene),
    &name, cur_row)?;
    //
    let mut i = 0_usize;
    loop {
        //
        if i >= object_stack.borrow().len() {
            break;
        }
        
        //
        let object = Rc::clone( object_stack.borrow().get(i).unwrap() );
        //
        if object.borrow().scope.get_value::<bool>("ACTIVE")
        .expect("The 'ACTIVE' constant should have been created in this object's scope by the time it was created") {
            //
            object.borrow_mut().call_fn(engine, name, args.clone())?;
            //
            cur_row = object.borrow().definition.row.clone();
            //
            handle_post_runtime_tasks(Rc::clone(&post_runtime_tasks),
            Rc::clone(&object_stack), Rc::clone(&scene),
            &name, cur_row)?;
        }
        //
        i += 1;
    }

    //
    Ok(())
}

//
pub fn run_game() -> Result<(), (EntityRow, Box<EvalAltResult>)>
{
    // Create the entity definitions hash map.
    let mut entity_defs: HashMap<u32,Rc<EntityDefinition>> = HashMap::new();
    // Create the API 'Engine', and the state manager instance.
    let (api_engine, state_manager, 
        cur_scene, mut cur_scene_id, 
        object_stack,
        post_runtime_tasks) = create_api(&mut entity_defs)?;
    
    //
    call_fn_on_all("create", (), &api_engine, &state_manager, &cur_scene, &object_stack, &post_runtime_tasks)?;

    // ..game loop..

    // Done!
    Ok(())
}