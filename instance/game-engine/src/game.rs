
use crate::webapp;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use rhai::{Engine, Scope, AST, Map, packages::Package, packages::StandardPackage, EvalAltResult, Dynamic};

mod entity;
mod rhai_convert;

//
#[derive(Clone)]
pub enum EntityRow {
    Manager,
    Scene(u32),
    Object(u32),
}

//
struct EntityDefinition {
    reference: EntityRow,
    script: AST,
    config: Map,
}

//
impl EntityDefinition {
    //
    fn new(engine: &Engine, refer: EntityRow) -> Result<Self, (EntityRow, Box<EvalAltResult>)> {
        //
        let ast = engine.compile(&match refer {
            EntityRow::Manager => webapp::getGameScript(),
            EntityRow::Object(id) => webapp::getEntityScript(id),
            EntityRow::Scene(id) => webapp::getEntityScript(id),
        });
        //
        if ast.is_err() {
            //
            return Err((refer, ast.unwrap_err().into()));
        }
        //
        let json = engine.parse_json(&match refer {
            EntityRow::Manager => webapp::getGameConfig(),
            EntityRow::Object(id) => webapp::getEntityConfig(id),
            EntityRow::Scene(id) => webapp::getEntityConfig(id),
        }, false);
        //
        if json.is_err() {
            //
            return Err((refer, json.unwrap_err()));
        }
        //
        let def = Self {
            //
            reference: refer,
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
    fn new(engine: &Engine, def: Rc<EntityDefinition>) -> Result<Self, (EntityRow, Box<EvalAltResult>)> {
        //
        let mut inst = Self {
            //
            definition: def,
            //
            scope: Scope::new(),
        };
        //
        match inst.definition.reference {
            //
            EntityRow::Scene(_) => {
                //
                inst.scope.push("Scene", entity::create_scene(&engine, &inst.definition.config));
            },
            //
            _ => {},
        }
        //
        let err = engine.run_ast_with_scope(&mut inst.scope, &inst.definition.script).err();
        //
        if err.is_some() {
            //
            return Err((inst.definition.reference.to_owned(), err.unwrap()));
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
            return Err((self.definition.reference.to_owned(), err.unwrap()));
        }
        //
        Ok(())
    }
}

//
fn create_api(entity_defs: &mut HashMap<u32,Rc<EntityDefinition>>) -> Result<(Engine, EntityInstance, Rc<RefCell<EntityInstance>>, u32), (EntityRow, Box<EvalAltResult>)> {
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
          .register_type_with_name::<entity::ObjectInstanceInfo>("ObjectInstanceInfo")
          .register_get("index", entity::ObjectInstanceInfo::get_index)
          .register_get("init_x", entity::ObjectInstanceInfo::get_init_x)
          .register_get("init_y", entity::ObjectInstanceInfo::get_init_y)
          .register_fn("to_string", entity::ObjectInstanceInfo::to_string)
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
    let mut state_manager: EntityInstance = EntityInstance::new(&engine, 
        Rc::clone(
            entity_defs.get(&0)
            .expect("entity_defs.get(&0) should have had the state manager's definition by now")
        )
    )?;

    // Take the 'State' object map from 
    // the state manager and turn it into
    // a shared variable.
    let state_map = state_manager.scope.remove::<Dynamic>("State").unwrap_or(Dynamic::UNIT).into_shared();
    // If it's not a ()
    if !state_map.is_unit() {
        // Push it back into the state manager
        // as a shared object map variable.
        state_manager.scope.push("State", state_map.clone());
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
        engine.on_def_var(|_, info, context| {
            match info.name {
                // If the name of the
                // defined variable is 'State'
                "State" => {
                    if !context.scope().contains(info.name) {
                        // If the variable doesn't
                        // exist in the scope already
                        // (which means it's not the 
                        // state manager's scope)
                        return Err("Can't define State outside the State Manager script".into());
                    } else if && info.nesting_level > &&(0 as usize)  {
                        // If the variable is being
                        // defined outside the global scope
                        return Err("Can't define State outside the global scope".into());
                    } else {
                        return Ok(true);
                    } 
                },
                // Otherwise, continue with the normal variable definition process,
                // where script runtime definitions of 'Scene' or 'Object' are forbidden.
                _ => Ok((info.name != "Scene" && info.name != "Object") || !info.will_shadow)
            }
        });
    }

    // Receive the rowid of the initial scene from the the state manager's config
    let mut cur_scene_id = state_manager.definition.config["initial-scene"].as_int()
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
                )
            )?
        )
    );

    // Register API closures, which need to 
    // capture the game-loop's environment
    let api_cur_scene = Rc::clone(&cur_scene);
    engine.register_fn("get_cur_scene", move || -> Result<entity::Scene, Box<EvalAltResult>> {
        //
        let cur_scene_borrow = api_cur_scene.try_borrow();
        //
        if cur_scene_borrow.is_err() {
            //
            return Err("Can't use the 'get_cur_scene' function inside a scene script! Use the Scene object instead.".into());
        }
        //
        Ok(cur_scene_borrow.unwrap().scope.get_value::<entity::Scene>("Scene")
        .expect("The Scene object should have been created in this scene's scope by the time it was created"))
    });

    // The API is done!
    Ok((engine, state_manager, cur_scene, cur_scene_id))
}

//
pub fn run_game() -> Result<(), (EntityRow, Box<EvalAltResult>)>
{
    // Create the entity definitions hash map.
    let mut entity_defs: HashMap<u32,Rc<EntityDefinition>> = HashMap::new();
    // Create the API 'Engine', and the state manager instance.
    let (api_engine, mut state_manager, 
        cur_scene, mut cur_scene_id) = create_api(&mut entity_defs)?;

    // Call the 'create' function on the state manager instance.
    state_manager.call_fn(&api_engine, "create", ())?;
    //
    cur_scene.borrow_mut().call_fn(&api_engine, "create", ())?;

    // ..game loop..

    // Done!
    Ok(())
}