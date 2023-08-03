
use crate::webapp;

use std::collections::HashMap;

use rhai::{Engine, Scope, AST, Map, packages::Package, packages::StandardPackage, EvalAltResult, Dynamic};

mod scene;

//
#[derive(Clone, PartialEq)]
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
struct EntityInstance<'a> {
    definition: &'a EntityDefinition,
    scope: Scope<'static>,
}

//
impl<'a> EntityInstance<'a> {
    //
    fn new(engine: &Engine, def: &'a EntityDefinition) -> Result<Self, (EntityRow, Box<EvalAltResult>)> {
        //
        let mut inst = Self {
            //
            definition: def,
            //
            scope: Scope::new(),
        };
        //
        let err = engine.run_ast_with_scope(&mut inst.scope, &def.script).err();
        //
        if err.is_some() {
            //
            return Err((def.reference.to_owned(), err.unwrap()));
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
fn create_api(entity_defs: &mut HashMap<u32,EntityDefinition>) -> Result<(Engine, EntityInstance), (EntityRow, Box<EvalAltResult>)> {
    // Create an 'Engine'
    let mut engine = Engine::new_raw();

    // Register API features to the 'Engine'
    engine.on_print(|text| {
        webapp::log(text);
    });

    // Register the Standard Package
    let package = StandardPackage::new();
    // Load the package into the 'Engine'
    package.register_into_engine(&mut engine);

    // Insert a new entity definition into the hash map.
    // This definition will define the state manager, and
    // will store its script and config data.
    entity_defs.insert(0, EntityDefinition::new(&engine, EntityRow::Manager)?);
    // Create a new entity instance for the state manager.
    // This instance will borrow its definition and contain
    // the entity's 'Scope'.
    let mut state_manager = EntityInstance::new(&engine, entity_defs.get(&0)
        .expect("entity_defs.get(&0) should have had the state manager's definition by now"))?;

    // Take the 'State' object map from 
    // the state manager and turn it into
    // a shared variable.
    let state_map = state_manager.scope.remove::<Dynamic>("State").unwrap_or(Dynamic::UNIT).into_shared();
    // If it's not a ()
    if !state_map.is_unit() {
        // Push it back into the state manager as
        // as a shared object map variable.
        state_manager.scope.push("State", state_map.clone());
        // Register a variable resolver.
        engine.on_var(move |name, _, context| {
            match name {
                // Continue with the normal variable resolution process
                // if the "State" variable is defined inside the context's
                // scope (which should be the state manager's), but if "State"
                // isn't defined, return a flat copy of the shared state map.
                "State" => if context.scope().contains(name) { Ok(None) } else { Ok(Some(state_map.flatten_clone())) },
                // Continue with the normal variable resolution process.
                _ => Ok(None)
            }
        });
        // Register a variable definition filter.
        engine.on_def_var(|_, info, context| {
            match info.name {
                // Continue with the normal variable definition process
                // if the "State" variable is already defined inside the 
                // context's global scope (which should be the state manager's),
                // but if "State" isn't defined or defined not in the global 
                // scope, Prevent the variable definition process.
                "State" => if !context.scope().contains(info.name) { Err("Can't define State outside the State Manager script".into()) } 
                           else if && info.nesting_level > &&(0 as usize)  { Err("Can't define State outside the global scope".into()) } 
                           else { Ok(true) },
                // Continue with the normal variable definition process.
                _ => Ok(true)
            }
        });
    }

    // The API is Done!
    Ok((engine, state_manager))
}

//
pub fn run_game() -> Result<(), (EntityRow, Box<EvalAltResult>)>
{
    // Create the entity definitions hash map.
    let mut entity_defs: HashMap<u32,EntityDefinition> = HashMap::new();
    // Create the API 'Engine', and the state manager instance.
    let (api_engine, mut state_manager) = create_api(&mut entity_defs)?;

    // Receive the rowid of the initial scene from the the state manager's config
    let scene_id = state_manager.definition.config["initial-scene"].as_int()
        .expect("The value of 'initial-scene' in the state manager's config should be an integer") as u32;
    //
    if !entity_defs.contains_key(&scene_id) {
        //
        entity_defs.insert(scene_id, EntityDefinition::new(&api_engine, EntityRow::Scene(scene_id))?);
    }
    //
    let mut scene = EntityInstance::new(&api_engine, entity_defs.get(&scene_id)
        .expect("entity_defs.get(&scene_id) should have had the scene's definition by now"))?;

    // Call the 'create' function on the state manager instance.
    state_manager.call_fn(&api_engine, "create", ())?;
    //
    scene.call_fn(&api_engine, "create", ())?;

    // let mut objects: Vec<EntityInstance> = Vec::new();

    // Done!
    Ok(())
}