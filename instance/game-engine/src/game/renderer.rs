
use std::{rc::Rc, collections::HashMap};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlBuffer, WebGlTexture, WebGlContextAttributes};

use crate::{data, game::TableRow};

use super::engine_api::{element, self};

const MAX_QUAD_COUNT: i32 = 1000;
const INDCIES_PER_QUAD: i32 = 6;
const VERTICES_PER_QUAD: i32 = 4;
const FLOATS_PER_VERTEX: i32 = 13;

pub enum ProgramDataLocation {
    Attribute(u32),
    Uniform(web_sys::WebGlUniformLocation),
}

//
pub enum AssetData {
    ImageData{width: i32, height: i32, texture: WebGlTexture},
}

//
impl AssetData {
    pub fn new_image_data(id: u32, gl_context: &WebGlRenderingContext) -> Result<AssetData, JsValue> {
        // Use the id to get the image png data.
        let image_data = data::get_asset_data(id);

        // Extract the image's width and height from the png data.
        let width = i32::from_be_bytes([image_data[16], image_data[17], image_data[18], image_data[19]]);
        let height = i32::from_be_bytes([image_data[20], image_data[21], image_data[22], image_data[23]]);
        // Create webgl texture object
        let texture = gl_context.create_texture().unwrap();
        // Activate texture unit 1 with this texture,
        // because unit 0 was reserved for the white texture,
        // and we override the binding to unit 1 in our rendering
        // loop regardless.
        gl_context.active_texture(WebGlRenderingContext::TEXTURE1);
        // bind the texture to TEXTURE_2D
        gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&texture));
        // Set Parameters
        gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::LINEAR as i32);
        gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::LINEAR as i32);
        gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_S,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32);
        gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_T,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32);
        // Extract the image's pixel data from the png
        // data using the 'image' crate and upload it into the texture.
        gl_context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGlRenderingContext::TEXTURE_2D, 0,
            WebGlRenderingContext::RGBA as i32, width, height,
            0, WebGlRenderingContext::RGBA,
            WebGlRenderingContext::UNSIGNED_BYTE,
            Some(image::load_from_memory_with_format(&image_data,
            image::ImageFormat::Png).expect("Couldn't load PNG file.")
            .to_rgba8().pixels().map(|pixel| { pixel.0 }).flatten()
            .collect::<Vec<u8>>().as_slice())
        )?;
        // Return the image data with the webgl texture object.
        Ok(AssetData::ImageData{width, height, texture})
    }
}

//
pub struct AssetDefinition {
    pub row: TableRow,
    pub asset_data: AssetData,
    pub config: rhai::Map,
}

//
impl AssetDefinition {
    //
    pub fn new(engine: &rhai::Engine, row: TableRow, gl_context: &WebGlRenderingContext) -> Result<Self, JsValue> {
        //
        let asset_data = match row {
            TableRow::Asset(id, 1) => AssetData::new_image_data(id, gl_context)?,
            TableRow::Asset(_, _) => { return Err(concat!("Audio / Font asset definitions are not implemented in",
                " this version of the engine. Please remove any use of them from your project.").into()); },
            _ => { return Err("Can't define an element as an asset.".into()); },
        };
        //
        let json = engine.parse_json(&match row {
            TableRow::Asset(id, _) => data::get_asset_config(id),
            _ => { return Err("Can't define an element as an asset.".into()); },
        }, false);
        //
        if let Some(err) = json.as_ref().err() {
            //
            return Err(JsValue::from_str(&row.to_err_string(&err.to_string())));
        }
        //
        Ok(Self{row,asset_data,
        config: json.expect(
            concat!("This Err should",
            " have been caught by this",
            " function beforehand")
        )})
    }
}

//
pub fn create_rendering_components(canvas_width: i32, canvas_height: i32)
 -> Result<(WebGlRenderingContext, WebGlProgram,
HashMap<String, ProgramDataLocation>, WebGlBuffer, WebGlBuffer), JsValue> {
    //
    let gl_context = create_context(
    canvas_width, canvas_height)?;
    //
    let (gl_program, 
    data_locations) = 
        create_scene_rendering_program(&gl_context)?;
    // 
    let vertex_buffer = gl_context
        .create_buffer()
        .ok_or("failed to create buffer")?;
    //
    gl_context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    // Use the scene rendering shader program.
    gl_context.use_program(Some(&gl_program));

    //
    if let Some(ProgramDataLocation::Attribute(location)) = data_locations.get("a_position") {
        // Enable the attribute-reading method
        gl_context.enable_vertex_attrib_array(*location);
        // Tell the GPU how to read the vertex buffer by attributes
        gl_context.vertex_attrib_pointer_with_i32(*location, 
        2, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 0);

    } else { return Err("Couldn't find attribute 'a_position'".into()); }

    //
    if let Some(ProgramDataLocation::Attribute(location)) = data_locations.get("a_color") {
        // Enable the attribute-reading method
        gl_context.enable_vertex_attrib_array(*location);
        // Tell the GPU how to read the vertex buffer by attributes
        gl_context.vertex_attrib_pointer_with_i32(*location, 
        4, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 8);

    } else { return Err("Couldn't find attribute 'a_color'".into()); }

    //
    if let Some(ProgramDataLocation::Attribute(location)) = data_locations.get("a_texcoord") {
        // Enable the attribute-reading method
        gl_context.enable_vertex_attrib_array(*location);
        // Tell the GPU how to read the vertex buffer by attributes
        gl_context.vertex_attrib_pointer_with_i32(*location, 
        2, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 24);

    } else { return Err("Couldn't find attribute 'a_texcoord'".into()); }

    //
    if let Some(ProgramDataLocation::Attribute(location)) = data_locations.get("a_texsize") {
        // Enable the attribute-reading method
        gl_context.enable_vertex_attrib_array(*location);
        // Tell the GPU how to read the vertex buffer by attributes
        gl_context.vertex_attrib_pointer_with_i32(*location, 
        2, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 32);

    } else { return Err("Couldn't find attribute 'a_texsize'".into()); }

    //
    if let Some(ProgramDataLocation::Attribute(location)) = data_locations.get("a_scale") {
        // Enable the attribute-reading method
        gl_context.enable_vertex_attrib_array(*location);
        // Tell the GPU how to read the vertex buffer by attributes
        gl_context.vertex_attrib_pointer_with_i32(*location, 
        2, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 40);

    } else { return Err("Couldn't find attribute 'a_scale'".into()); }

    //
    if let Some(ProgramDataLocation::Attribute(location)) = data_locations.get("a_texindex") {
        // Enable the attribute-reading method
        gl_context.enable_vertex_attrib_array(*location);
        // Tell the GPU how to read the vertex buffer by attributes
        gl_context.vertex_attrib_pointer_with_i32(*location, 
        1, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 48);

    } else { return Err("Couldn't find attribute 'a_texindex'".into()); }

    //
    if let Some(ProgramDataLocation::Uniform(location)) = data_locations.get("u_resolution") {
        //
        gl_context.uniform2f(Some(location), canvas_width as f32, canvas_height as f32);

    } else { return Err("Couldn't find uniform 'u_resolution'".into()); }

    // Get the maximun texture units we can use on the fragment shader
    let max_texture_units = gl_context
    .get_parameter(WebGlRenderingContext::MAX_TEXTURE_IMAGE_UNITS)
    .unwrap().as_f64().unwrap() as i32;

    //
    if let Some(ProgramDataLocation::Uniform(location)) = data_locations.get("u_textures") {
        //
        gl_context.uniform1iv_with_i32_array(Some(location),
        (0..max_texture_units).collect::<Vec<i32>>().as_slice());

    } else { return Err("Couldn't find uniform 'u_textures'".into()); }
    
    //
    gl_context.buffer_data_with_i32(
        WebGlRenderingContext::ARRAY_BUFFER,
        MAX_QUAD_COUNT * VERTICES_PER_QUAD * FLOATS_PER_VERTEX * 4,
        WebGlRenderingContext::DYNAMIC_DRAW,
    );

    //
    let index_buffer = gl_context
        .create_buffer()
        .ok_or("failed to create buffer")?;
    //
    gl_context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

    //
    let mut indcies: Vec<u16> = Vec::new();
    for i in 0..MAX_QUAD_COUNT {
        // Copy data into the Vec<u16> 
        // from the slice which includes
        // the indcies which represent 
        // the order of the vertices' rendering.
        indcies.extend_from_slice(&[
            (0 + VERTICES_PER_QUAD * i) as u16,
            (1 + VERTICES_PER_QUAD * i) as u16,
            (2 + VERTICES_PER_QUAD * i) as u16,
            (2 + VERTICES_PER_QUAD * i) as u16,
            (1 + VERTICES_PER_QUAD * i) as u16,
            (3 + VERTICES_PER_QUAD * i) as u16,
        ]);
    }
    // Note that `Uint16Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Uint16Array` to be invalid.
    //
    // As a result, after `Uint16Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        //
        let indcies_array = js_sys::Uint16Array::view(indcies.as_slice());
        //
        gl_context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &indcies_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
    // Create webgl white texture object
    let white_texture = gl_context.create_texture().unwrap();
    // Activate texture unit 0 with this white texture
    gl_context.active_texture(WebGlRenderingContext::TEXTURE0);
    // bind the white texture to the webgl context
    gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(&white_texture));
    // Set Parameters
    gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
        WebGlRenderingContext::TEXTURE_MIN_FILTER,
        WebGlRenderingContext::LINEAR as i32);
    gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
        WebGlRenderingContext::TEXTURE_MAG_FILTER,
        WebGlRenderingContext::LINEAR as i32);
    gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
        WebGlRenderingContext::TEXTURE_WRAP_S,
        WebGlRenderingContext::CLAMP_TO_EDGE as i32);
    gl_context.tex_parameteri(WebGlRenderingContext::TEXTURE_2D,
        WebGlRenderingContext::TEXTURE_WRAP_T,
        WebGlRenderingContext::CLAMP_TO_EDGE as i32);
    // Create the image data of the texture (1x1 white pixel)
    gl_context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGlRenderingContext::TEXTURE_2D, 0,
        WebGlRenderingContext::RGBA as i32, 1, 1,
        0, WebGlRenderingContext::RGBA,
        WebGlRenderingContext::UNSIGNED_BYTE,
        Some(&[255,255,255,255])
    )?;
    //
    Ok((gl_context, gl_program, data_locations, vertex_buffer, index_buffer))
}

//
pub fn render_scene(gl_context: &WebGlRenderingContext, gl_program: &WebGlProgram,
data_locations: &HashMap<String,ProgramDataLocation>, vertex_buffer: &web_sys::WebGlBuffer,
index_buffer: &web_sys::WebGlBuffer, scene: &element::Scene, asset_defs: &HashMap<u32,Result<AssetDefinition, JsValue>>,
object_stack: &Vec<engine_api::Element<engine_api::Object>>, elapsed: f64) -> Result<(), JsValue> {
    // Use the scene rendering shader program.
    gl_context.use_program(Some(&gl_program));

    {
        // Get the outside-color of the stage.
        let outcolor = hex_color_to_rgba(&scene.out_color);
        // Set the clear color to the outside color.
        gl_context.clear_color(outcolor[0], outcolor[1], outcolor[2], outcolor[3]);
    }// 'outcolor' drops here

    // Clear the canvas
    gl_context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    // Set the blending method the alpha of the images will control
    gl_context.enable(WebGlRenderingContext::BLEND);
    gl_context.blend_func(WebGlRenderingContext::ONE, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);

    //
    if let Some(ProgramDataLocation::Uniform(location)) = data_locations.get("u_camera") {
        //
        gl_context.uniform2f(Some(location), scene.camera.position.x.floor(), scene.camera.position.y.floor());

    } else { return Err("Couldn't find uniform 'u_camera'".into()); }

    //
    if let Some(ProgramDataLocation::Uniform(location)) = data_locations.get("u_zoom") {
        //
        gl_context.uniform1f(Some(location), scene.camera.zoom);

    } else { return Err("Couldn't find uniform 'u_zoom'".into()); }
    
    //
    let mut vertices: Vec<f32> = Vec::new();
    // Add a quad for the scene.
    vertices.extend_from_slice(&generate_colored_quad(0.0, 0.0,
    scene.width.floor(), scene.height.floor(),
    hex_color_to_rgba(&scene.in_color)));

    // Get the maximun texture units we can use on the fragment shader
    let max_texture_units = gl_context
    .get_parameter(WebGlRenderingContext::MAX_TEXTURE_IMAGE_UNITS)
    .unwrap().as_f64().unwrap() as i32;

    // 
    let mut texture_slots: Vec<u32> = vec![0];
    texture_slots.reserve_exact((max_texture_units-1) as usize);

    //
    let mut unit_id: f32;
    let mut quad_width: f32;
    let mut quad_height: f32;
    let mut tex_width: f32;
    let mut tex_height: f32;
    let mut texcoord_1: [f32; 2] = [0.0, 0.0];
    let mut texcoord_2: [f32; 2] = [0.0, 0.0];
    let mut origin_minus_offset: [f32; 2] = [0.0, 0.0];
    //
    for &index in scene.layers[0..scene.layers_len].iter()
    .flat_map(|layer| { layer.instances.iter() }) {
        //
        if (vertices.len() as i32) / (FLOATS_PER_VERTEX * VERTICES_PER_QUAD * MAX_QUAD_COUNT) >= 1 {
            //
            flush(&gl_context, &mut vertices, &vertex_buffer, &index_buffer);
            texture_slots.truncate(1);
        }
        //
        if let Some(object) = object_stack.get(index as usize) {
            // Get a mutable borrow of the object,
            // which will later switch to the sprite of the object.
            let object_or_sprite = Rc::clone(&object.map);
            let mut object_or_sprite = object_or_sprite.borrow_mut();
            let mut object_or_sprite = object_or_sprite
                .write_lock::<element::Object>()
                .expect("write lock should succeed.");
            //
            if object_or_sprite.active == false {
                continue;
            }

            // Here the object will switch to a sprite.
            // This needs to be done because the sprite is
            // owned by the object, and borrowing it at the
            // same time with the mutable borrow of the object
            // will violate the borrowing rules.
            {
                // Get the object's current sprite asset's id.
                let cur_sprite_asset_id = object_or_sprite.sprites.cur_asset;
                // Receive a mutable borrow of the active sprite.
                let object_or_sprite = object_or_sprite.sprites.members
                .get_mut(cur_sprite_asset_id)
                .expect("the cur_asset index should exist in the members array of an AssetList.");
                //
                let texture_asset = asset_defs.get(&object_or_sprite.id)
                .expect("the id included in a sprites array of an object should be correct").as_ref()?;
                //
                let gl_texture: &WebGlTexture;
                //
                match &texture_asset.asset_data {
                    AssetData::ImageData { width, height, texture } => {
                        quad_width = *width as f32;
                        quad_height = *height as f32;
                        tex_width = *width as f32;
                        tex_height = *height as f32;
                        gl_texture = texture;
                    }
                    //_ => { return Err("The asset data of an image asset wasn't image data.".into()); }
                }

                //
                let fps = engine_api::dynamic_to_number(&texture_asset.config["fps"])
                .expect("Every sprite's config should have a 'fps' property with an integer number.") as i32;
                
                //
                let mut found_anim = false;
                //
                for anim in texture_asset.config["animations"]
                .clone().into_typed_array::<rhai::Map>().expect(concat!("Every sprite's config",
                " should have an 'animations' array, which contains object-like members.")) {
                    //
                    if anim["name"].clone().into_string().expect(concat!("Every",
                    " member of the 'animations' array in a sprite's config should",
                    " have a 'name' property with a string.")) != object_or_sprite.cur_animation {
                        continue;
                    }
                    //
                    found_anim = true;
                    //
                    let frames = anim["frames"]
                    .clone().into_typed_array::<rhai::Map>().expect(concat!("Every",
                    " member of the 'animations' array in a sprite's config should",
                    " have a 'frames' array, which contains object-like members."));
                    //
                    if !object_or_sprite.is_animation_finished {
                        //
                        object_or_sprite.animation_time += elapsed;
                        object_or_sprite.cur_frame = ((fps as f64) * object_or_sprite.animation_time * 0.001).floor() as u32;
                        //
                        if object_or_sprite.cur_frame >= frames.len() as u32 {
                            //
                            object_or_sprite.cur_frame = (frames.len() - 1) as u32;
                            //
                            if !object_or_sprite.repeat {
                                object_or_sprite.is_animation_finished = true;
                            } else {
                                object_or_sprite.cur_frame = 0;
                                object_or_sprite.animation_time = 0.0;
                            }
                        }
                    }
                    //
                    let area = frames[object_or_sprite.cur_frame as usize]["area"]
                    .clone().try_cast::<rhai::Map>().expect("Every frame should have a 'area' object-like property");
                    //
                    let offset = frames[object_or_sprite.cur_frame as usize]["offset"]
                    .clone().try_cast::<rhai::Map>().expect("Every frame should have a 'offset' object-like property");
                    //
                    texcoord_1 = [
                        engine_api::dynamic_to_number(&area["x1"])
                        .expect("x1 should be a number.") / quad_width, 
                        engine_api::dynamic_to_number(&area["y1"])
                        .expect("y1 should be a number.") / quad_height
                    ];
                    //
                    texcoord_2 = [
                        1.0 - (engine_api::dynamic_to_number(&area["x2"])
                        .expect("x2 should be a number.") / quad_width),
                        1.0 - (engine_api::dynamic_to_number(&area["y2"])
                        .expect("y2 should be a number.") / quad_height)
                    ];
                    //
                    quad_width = quad_width - (
                        engine_api::dynamic_to_number(&area["x1"])
                        .expect("x1 should be a number.") +
                        engine_api::dynamic_to_number(&area["x2"])
                        .expect("x2 should be a number.")
                    );
                    //
                    quad_height = quad_height - (
                        engine_api::dynamic_to_number(&area["y1"])
                        .expect("y1 should be a number.") +
                        engine_api::dynamic_to_number(&area["y2"])
                        .expect("y2 should be a number.")
                    );
                    //
                    let origin = texture_asset.config["origin"]
                    .clone().try_cast::<rhai::Map>().expect(concat!("Every sprite's config should",
                    " have a 'origin' object-like property"));
                    //
                    origin_minus_offset = [
                        engine_api::dynamic_to_number(&origin["x"])
                        .expect("origin.x should be a number") -
                        engine_api::dynamic_to_number(&offset["x"])
                        .expect("offset.x should be a number"),
                        engine_api::dynamic_to_number(&origin["y"])
                        .expect("origin.y should be a number") -
                        engine_api::dynamic_to_number(&offset["y"])
                        .expect("offset.y should be a number")
                    ];
                    //
                    break;
                }

                //
                if !found_anim {
                    //
                    continue;
                }

                //
                if let Some((idx, _)) = texture_slots.iter()
                .enumerate().find(|&slot| { object_or_sprite.id == *slot.1 }) {
                    //
                    unit_id = idx as f32;
                } else if texture_slots.len() < (max_texture_units as usize) {
                    //
                    gl_context.active_texture(WebGlRenderingContext::TEXTURE0 + (texture_slots.len() as u32));
                    //
                    gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(gl_texture));
                    //
                    texture_slots.push(object_or_sprite.id);
                    //
                    unit_id = (texture_slots.len() as f32)-1_f32;
                } else {
                    //
                    flush(&gl_context, &mut vertices, &vertex_buffer, &index_buffer);
                    texture_slots.truncate(1);
                    //
                    gl_context.active_texture(WebGlRenderingContext::TEXTURE0 + (texture_slots.len() as u32));
                    //
                    gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(gl_texture));
                    //
                    texture_slots.push(object_or_sprite.id);
                    //
                    unit_id = (texture_slots.len() as f32)-1_f32;
                }
            }// Here the sprite switches back to being an object.

            //
            vertices.extend_from_slice(&generate_textured_quad(object_or_sprite.position.x.floor() - 
            (origin_minus_offset[0] * object_or_sprite.scale.x), object_or_sprite.position.y.floor() - 
            (origin_minus_offset[1] * object_or_sprite.scale.y), quad_width * object_or_sprite.scale.x,
            quad_height * object_or_sprite.scale.y, texcoord_1, texcoord_2,
            [tex_width, tex_height], [object_or_sprite.scale.x, object_or_sprite.scale.y],
            unit_id));
        }
    }
    //
    flush(&gl_context, &mut vertices, &vertex_buffer, &index_buffer);

    // The Rendering is Done!
    Ok(())
}

//
fn flush(gl_context: &WebGlRenderingContext, vertices: &mut Vec<f32>, vertex_buffer: &web_sys::WebGlBuffer,
index_buffer: &web_sys::WebGlBuffer) {
    // Bind the vertex buffer
    // to the WebGL context.
    gl_context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(vertex_buffer));
    // Bind the index buffer
    // to the WebGL context.
    gl_context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(index_buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.

    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        //
        let vertices_array = js_sys::Float32Array::view(vertices.as_slice());
        //
        gl_context.buffer_sub_data_with_f64_and_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            0.0,
            &vertices_array,
        );
    }
    // Draw the scene rectangle on the canvas
    gl_context.draw_elements_with_i32(
        WebGlRenderingContext::TRIANGLES,
        (vertices.len() as i32 * INDCIES_PER_QUAD) / (VERTICES_PER_QUAD * FLOATS_PER_VERTEX),
        WebGlRenderingContext::UNSIGNED_SHORT,
        0,
    );
    //
    vertices.clear();
}

fn create_context(width: i32, height: i32)  -> Result<WebGlRenderingContext, JsValue> {
    // Get the page's document.
    let document = web_sys::window().unwrap().document().unwrap();
    // Get to canvas from the document.
    let canvas = document.get_element_by_id("canvas").unwrap();
    // Convert the canvas element into an HTMLCanvasElement object.
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    // Set the desired width and height of the canvas,
    // which is the size the canvas will be rendered
    // at regardless of how CSS displays it.
    canvas.set_attribute("width", &format!("{width}"))?;
    canvas.set_attribute("height", &format!("{height}"))?;

    // Get the WebGL context
    // from the canvas.
    // (String::from("premultipliedAlpha"), false)
    let context = canvas
    .get_context_with_context_options("webgl",
        WebGlContextAttributes::new()
        .alpha(false)
        .premultiplied_alpha(true)
        .dyn_ref::<JsValue>().unwrap()
    )?
    .unwrap()
    .dyn_into::<WebGlRenderingContext>()?;
    //
    context.pixel_storei(WebGlRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);

    //
    Ok(context)
}

fn create_scene_rendering_program(gl_context: &WebGlRenderingContext)
 -> Result<(WebGlProgram,HashMap<String,ProgramDataLocation>), JsValue> {
    // Create the vertex shader
    let vert_shader = compile_shader(
        &gl_context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec2 a_position;
        attribute vec4 a_color;
        attribute vec2 a_texcoord;
        attribute vec2 a_texsize;
        attribute vec2 a_scale;
        attribute float a_texindex;

        uniform vec2 u_resolution;
        uniform vec2 u_camera;
        uniform float u_zoom;

        varying vec4 v_color;
        varying vec2 v_texcoord;
        varying vec2 v_texsize;
        varying vec2 v_scale;
        varying float v_texindex;

        void main() {

            v_color = a_color;
            v_texcoord = a_texcoord * a_texsize;
            v_texsize = a_texsize;
            v_scale = a_scale;
            v_texindex = a_texindex;

            vec2 camRelative = a_position - u_camera;
            vec2 clipSpace = camRelative * abs(u_zoom) * 2.0 / u_resolution;
            gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
        }
    "#,
    )?;

    // Create the fragment shader
    let frag_shader = compile_shader(
        &gl_context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        precision highp float;

        varying vec4 v_color;
        varying vec2 v_texcoord;
        varying vec2 v_texsize;
        varying vec2 v_scale;
        varying float v_texindex;

        uniform float u_zoom;
        uniform sampler2D u_textures[gl_MaxTextureImageUnits];

        void main() {

            vec2 texcoord = v_texcoord;

            vec2 pixPerTex = abs(v_scale) * abs(u_zoom);

            vec2 pixoffset = clamp(fract(v_texcoord) * pixPerTex, 0.0, 0.5) - clamp((1.0 - fract(v_texcoord)) * pixPerTex, 0.0, 0.5);

            if (v_texsize != vec2(0.0,0.0)) {
                
                texcoord = (floor(v_texcoord) + 0.5 + pixoffset) / v_texsize;
            }

            int index = int(v_texindex);

            // gl_FragColor = texture2D(u_textures[index], texcoord) * v_color;
            // ERROR: '[]' : Index expression must be constant

            for (int i=0; i<gl_MaxTextureImageUnits; i++) {
                if (index == i) {
                    gl_FragColor = texture2D(u_textures[i], texcoord) * v_color;
                }
            }
        }
    "#,
    )?;

    // Create the shader program using
    // the vertex and fragment shaders.
    let gl_program = link_program(&gl_context, &vert_shader, &frag_shader)?;

    //
    let mut data_locations: HashMap<String,ProgramDataLocation> = HashMap::new();

    // Look up attribute locations.
    data_locations.insert(String::from("a_position"), ProgramDataLocation::Attribute(gl_context
        .get_attrib_location(&gl_program, "a_position") as u32
    ));
    data_locations.insert(String::from("a_color"), ProgramDataLocation::Attribute(gl_context
        .get_attrib_location(&gl_program, "a_color") as u32
    ));
    data_locations.insert(String::from("a_texcoord"), ProgramDataLocation::Attribute(gl_context
        .get_attrib_location(&gl_program, "a_texcoord") as u32
    ));
    data_locations.insert(String::from("a_texsize"), ProgramDataLocation::Attribute(gl_context
        .get_attrib_location(&gl_program, "a_texsize") as u32
    ));
    data_locations.insert(String::from("a_scale"), ProgramDataLocation::Attribute(gl_context
        .get_attrib_location(&gl_program, "a_scale") as u32
    ));
    data_locations.insert(String::from("a_texindex"), ProgramDataLocation::Attribute(gl_context
        .get_attrib_location(&gl_program, "a_texindex") as u32
    ));

    // Look up uniform locations.
    data_locations.insert(String::from("u_resolution"), ProgramDataLocation::Uniform(gl_context
        .get_uniform_location(&gl_program, "u_resolution")
        .ok_or("Unable to get uniform location (u_resolution)")?
    ));
    data_locations.insert(String::from("u_camera"), ProgramDataLocation::Uniform(gl_context
        .get_uniform_location(&gl_program, "u_camera")
        .ok_or("Unable to get uniform location (u_camera)")?
    ));
    data_locations.insert(String::from("u_zoom"), ProgramDataLocation::Uniform(gl_context
        .get_uniform_location(&gl_program, "u_zoom")
        .ok_or("Unable to get uniform location (u_zoom)")?
    ));
    data_locations.insert(String::from("u_textures"), ProgramDataLocation::Uniform(gl_context
        .get_uniform_location(&gl_program, "u_textures")
        .ok_or("Unable to get uniform location (u_textures)")?
    ));

    Ok((gl_program, data_locations))
}

// Creates an array of f32 floats with
// the vertices which should represent a 
// desired colored rectangle, while only using
// 5 arguments: x, y, width, height and color.
fn generate_colored_quad(x: f32, y: f32,
width: f32, height: f32, color: [f32; 4]) -> [f32; (VERTICES_PER_QUAD * FLOATS_PER_VERTEX) as usize] {
    let x1 = x;
    let x2 = x + width;
    let y1 = y;
    let y2 = y + height;
    //  x, y, red, green, blue, alpha, texture_x(0-1), texture_y(0-1), tex_width, tex_height, scale_x, scale_y, texture_unit_id
    [
        x1, y1, color[0], color[1], color[2], color[3], 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
        x2, y1, color[0], color[1], color[2], color[3], 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0,
        x1, y2, color[0], color[1], color[2], color[3], 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
        x2, y2, color[0], color[1], color[2], color[3], 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
    ]
}

// Creates an array of f32 floats with
// the vertices which should represent a 
// desired textured rectangle, while only using
// 7 arguments: x, y, width, height, texpoint_1,
// texpoint_2 and texture unit id.
fn generate_textured_quad(x: f32, y: f32,
width: f32, height: f32, texpoint_1: [f32; 2],
texpoint_2: [f32; 2], tex_size: [f32; 2],
scale: [f32; 2], texunit_id: f32) -> [f32; (VERTICES_PER_QUAD * FLOATS_PER_VERTEX) as usize] {
    let x1 = x;
    let x2 = x + width;
    let y1 = y;
    let y2 = y + height;
    //  x, y, red, green, blue, alpha, texture_x(0-1), texture_y(0-1), tex_width, tex_height, scale_x, scale_y, texture_unit_id
    [
        x1, y1, 1.0, 1.0, 1.0, 1.0, texpoint_1[0], texpoint_1[1], tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
        x2, y1, 1.0, 1.0, 1.0, 1.0, texpoint_2[0], texpoint_1[1], tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
        x1, y2, 1.0, 1.0, 1.0, 1.0, texpoint_1[0], texpoint_2[1], tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
        x2, y2, 1.0, 1.0, 1.0, 1.0, texpoint_2[0], texpoint_2[1], tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
    ]
}

// Receives a string borrow with a
// hex color code (#RRGGBBAA / #RRGGBB),
// and converts it into a slice of floats,
// for use with the WebGL context.
fn hex_color_to_rgba(hex: &str) -> [f32; 4] {
    // Result slice with
    // place-holder values
    let mut result: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    // Result slice index counter
    let mut i = 0;
    // Hex color string
    // index counter
    let mut si = 1;

    while si < hex.len() {
        // Convert the hex string into an
        // integer, cast it to f32 and divide
        // by 255, to make it range from 0 to 1.
        result[i] = i64::from_str_radix(&hex[si..si+2], 16)
        .expect("hex to i64 parse should succeed") as f32 / 255_f32;
        // Increase hex color string
        // index counter by 2.
        si += 2;
        // Increase result slice
        // index counter by 1.
        i += 1;
    }
    // Return the 
    // result slice
    result
}

// Compiles a shader and
// returns a WebGlShader
// object if the compilation
// is successful. otherwise,
// the error log is returned.
fn compile_shader(
    gl_context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    // Create a shader with the
    // provided WebGL context.
    let shader = gl_context
        .create_shader(shader_type)
        .ok_or(String::from("Unable to create shader"))?;
    // Attach the provided GLSL
    // source to the shader.
    gl_context.shader_source(&shader, source);
    // Compile the GLSL source
    // attached to the shader.
    gl_context.compile_shader(&shader);

    // Check the shader's compile status
    if gl_context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // Return the shader if it's compiled.
        Ok(shader)
    } else {
        // Return the compilation error log
        // if the shader isn't compiled.
        Err(gl_context
            .get_shader_info_log(&shader)
            .unwrap_or(String::from("Unknown error occurred while creating shader")))
    }
}

// Links a new program
// to a WebGl context and
// returns a WebGlProgram
// object if the linking
// is successful. otherwise,
// the error log is returned.
fn link_program(
    gl_context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    // Create a program with the
    // provided WebGL context.
    let program = gl_context
        .create_program()
        .ok_or(String::from("Unable to create program"))?;
    // Attach the provided vertex
    // and fragment shaders to the program.
    gl_context.attach_shader(&program, vert_shader);
    gl_context.attach_shader(&program, frag_shader);
    // Link the program to the
    // provided WebGL context.
    gl_context.link_program(&program);

    // Check the program's link status
    if gl_context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // Return the program if it's linked.
        Ok(program)
    } else {
        // Return the linking error log
        // if the program isn't linked.
        Err(gl_context
            .get_program_info_log(&program)
            .unwrap_or(String::from("Unknown error occurred while creating shader program")))
    }
}