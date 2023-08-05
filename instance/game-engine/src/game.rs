
use crate::webapp;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use rhai::{Engine, Scope, AST, Map, packages::Package, packages::StandardPackage, EvalAltResult, Dynamic};

mod scene;

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
                inst.scope.push("Scene", scene::create_scene(&engine, &inst.definition.config));
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
          .register_type_with_name::<scene::Camera>("Camera")
          .register_get_set("x", scene::Camera::get_x, scene::Camera::set_x)
          .register_set("x", scene::Camera::set_x_rhai_int)
          .register_set("x", scene::Camera::set_x_rhai_float)
          .register_get_set("y", scene::Camera::get_y, scene::Camera::set_y)
          .register_set("y", scene::Camera::set_y_rhai_int)
          .register_set("y", scene::Camera::set_y_rhai_float)
          .register_get_set("zoom", scene::Camera::get_zoom, scene::Camera::set_zoom)
          .register_set("zoom", scene::Camera::set_zoom_rhai_int)
          .register_set("zoom", scene::Camera::set_zoom_rhai_float)
          .register_fn("to_string", scene::Camera::to_string)
          .register_type_with_name::<scene::Layer>("Layer")
          .register_get("name", scene::Layer::get_name)
          .register_fn("to_string", scene::Layer::to_string)
          .register_type_with_name::<scene::Scene>("Scene")
          .register_get_set("width", scene::Scene::get_width, scene::Scene::set_width)
          .register_set("width", scene::Scene::set_width_rhai_int)
          .register_set("width", scene::Scene::set_width_rhai_float)
          .register_get_set("height", scene::Scene::get_height, scene::Scene::set_height)
          .register_set("height", scene::Scene::set_height_rhai_int)
          .register_set("height", scene::Scene::set_height_rhai_float)
          .register_get_set("in_color", scene::Scene::get_inside_color, scene::Scene::set_inside_color)
          .register_get_set("out_color", scene::Scene::get_outside_color, scene::Scene::set_outside_color)
          .register_get_set("camera", scene::Scene::get_camera, scene::Scene::set_camera)
          .register_get("layers", scene::Scene::get_layers)
          .register_fn("to_string", scene::Scene::to_string);

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
    engine.register_fn("get_cur_scene", move || -> Result<scene::Scene, Box<EvalAltResult>> {
        //
        let cur_scene_borrow = api_cur_scene.try_borrow();
        //
        if cur_scene_borrow.is_err() {
            //
            return Err("Can't use the 'get_cur_scene' function inside a scene script! Use the Scene object instead.".into());
        }
        //
        Ok(cur_scene_borrow.unwrap().scope.get_value::<scene::Scene>("Scene")
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