
use std::{rc::Rc, collections::HashMap};

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlBuffer, WebGlTexture, WebGlContextAttributes};

use crate::{data, game::{TableRow, dynamic_to_number}};

use super::engine_api::{element, self};

pub type AssetDefinitions = HashMap<u32,Result<AssetDefinition, JsValue>>;

/// The html element id
/// of the canvas element.
static mut CANVAS_ID: &str = "canvas";

const MAX_QUAD_COUNT: i32 = 1000;
const INDCIES_PER_QUAD: i32 = 6;
const VERTICES_PER_QUAD: i32 = 4;
const FLOATS_PER_VERTEX: i32 = 13;

const VERTEX_SHADER: &str = r#"
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
"#;

const FRAGMENT_SHADER: &str = r#"
precision highp float;

varying vec4 v_color;
varying vec2 v_texcoord;
varying vec2 v_texsize;
varying vec2 v_scale;
varying float v_texindex;

uniform float u_zoom;
uniform vec4 u_cam_color;
uniform sampler2D u_textures[gl_MaxTextureImageUnits];

void main() {
    
    vec2 texcoord = v_texcoord;

    // The following method utliizes bilinear
    // sampling to apply per-texel anti-aliasing,
    // which will help to prevent image distortions,
    // while keeping the texels sharp enough.

    // Calculate the ratio between
    // the pixel and texel sizes.
    vec2 pixPerTex = abs(v_scale) * abs(u_zoom);

    // Calculate the offset of the pixel
    // from the center of the texel.

    vec2 pixoffset = clamp(fract(v_texcoord) * pixPerTex, 0.0, 0.5) - clamp((1.0 - fract(v_texcoord)) * pixPerTex, 0.0, 0.5);

    // Because we don't want to devide by zero,
    // make sure the texture size isn't zero.
    if (v_texsize != vec2(0.0,0.0)) {

        // We will apply the offset to the texture coordinates
        texcoord = (floor(v_texcoord) + 0.5 + pixoffset) / v_texsize;
    }

    int index = int(v_texindex);

    // gl_FragColor = texture2D(u_textures[index], texcoord) * v_color * u_cam_color;
    // ERROR: '[]' : Index expression must be constant

    // This is a workaround for the above error.
    for (int i=0; i<gl_MaxTextureImageUnits; i++) {
        if (index == i) {
            gl_FragColor = texture2D(u_textures[i], texcoord) * v_color * u_cam_color;
        }
    }
}
"#;

const ATTRIBUTE_MATRIX: [(&str, i32, i32); 6] = [
    // name, size, offset
    ("a_position", 2, 0),
    ("a_color", 4, 8),
    ("a_texcoord", 2, 24),
    ("a_texsize", 2, 32),
    ("a_scale", 2, 40),
    ("a_texindex", 1, 48),
];

const UNIFORM_LIST: [&str; 5] = [
    "u_resolution",
    "u_camera",
    "u_zoom",
    "u_cam_color",
    "u_textures",
];

/// This enum will help
/// the `AssetDefinition`\
/// struct to store data of
/// different types of assets.
pub enum AssetData {
    ImageData{width: i32, height: i32, texture: WebGlTexture},
}

impl AssetData {
    /// Using a rowid and the webgl context,
    /// this function will return an `AssetData`\
    /// enum variant with the image data of the
    /// asset with the provided rowid.
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

/// A struct that will be used to
/// store all the data which is loaded\
/// for a single asset defined in the
/// project file/game data file.
pub struct AssetDefinition {
    pub row: TableRow,
    pub asset_data: AssetData,
    pub config: rhai::Map,
}

impl AssetDefinition {
    /// Using a rhai engine, row data and
    /// a webgl context, this function will\
    /// load the asset's configuration and
    /// data and return a new asset definition,
    /// or an error.
    pub fn new(engine: &rhai::Engine, row: TableRow, gl_context: &WebGlRenderingContext) -> Result<Self, JsValue> {
        // Load the asset's data using the rowid.
        let asset_data = match row {
            // An asset of type 1 is an sprite asset.
            TableRow::Asset(id, 1) => AssetData::new_image_data(id, gl_context),
            // Assets of any other type
            // aren't implemented yet.
            TableRow::Asset(_, _) => { Err(concat!("Audio / Font asset definitions are not implemented in",
                " this version of the engine. Please remove any use of them from your project.").into()) },
            _ => { Err("Can't define an element as an asset.".into()) },
        };
        // Return an error if the asset data couldn't be loaded.
        if let Some(err) = asset_data.as_ref().err() {
            return Err(JsValue::from_str(&row.to_err_string(&err.as_string()
            .unwrap_or(String::from("Uncaught image data loading error.")))));
        }
        // Load the asset's configuration and
        // parse it into a rhai map (JSON object).
        let json = engine.parse_json(&match row {
            TableRow::Asset(id, _) => data::get_asset_config(id),
            _ => { return Err("Can't define an element as an asset.".into()); },
        }, false);
        // Return an error if any occured
        // while parsing the config.
        if let Some(err) = json.as_ref().err() {
            return Err(JsValue::from_str(&row.to_err_string(&err.to_string())));
        }
        // Return the asset definition.
        Ok(Self{row,
        asset_data: asset_data.expect(
            concat!("This Err should",
            " have been caught by this",
            " function beforehand")
        ),
        config: json.expect(
            concat!("This Err should",
            " have been caught by this",
            " function beforehand")
        )})
    }
}

/// This struct stores all the
/// rendering components of the game.\
/// It will be used to manage the
/// rendering proccess of the game.
pub struct WebGlRenderer {
    pub gl_context: WebGlRenderingContext,
    pub gl_program: WebGlProgram,
    uniform_locations: HashMap<String,web_sys::WebGlUniformLocation>,
    vertex_buffer: WebGlBuffer,
    index_buffer: WebGlBuffer,
    vertex_vec: Vec<f32>,
    texture_slots: Vec<u32>,
    max_texture_units: i32,
}

impl WebGlRenderer {
    /// This function will create a new
    /// webgl renderer and return it. 
    /// 
    /// The renderer includes a webgl context, a shader program,\
    /// a vertex buffer and an index buffer among other things,\
    /// which will be used to render the game.
    pub fn new(game: &element::Game) -> Result<Self, JsValue> {
        // Activate the canvas webgl context.
        let gl_context = activate_context(
        game.canvas_width, game.canvas_height)?;
        // Create the shader program
        let (gl_program, 
        uniform_locations) = 
            create_program(&gl_context)?;
        // Create the vertex buffer
        let vertex_buffer = gl_context
            .create_buffer()
            .ok_or("failed to create buffer")?;
        // Bind the vertex buffer
        // to the WebGL context.
        gl_context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        // Use the shader program
        // in the rendering context.
        gl_context.use_program(Some(&gl_program));

        // Enable the vertex attributes
        for (index, &(_, size, offset)) in ATTRIBUTE_MATRIX.iter().enumerate() {
            gl_context.enable_vertex_attrib_array(index as u32);
            gl_context.vertex_attrib_pointer_with_i32(index as u32, 
            size, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, offset);
        }

        // Get the maximun texture units we can use on the fragment shader
        let max_texture_units = gl_context
        .get_parameter(WebGlRenderingContext::MAX_TEXTURE_IMAGE_UNITS)
        .unwrap().as_f64().unwrap() as i32;

        // Set the uniform 'u_textures' to an array of all
        // the texture units we might use in the fragment shader.
        if let Some(location) = uniform_locations.get("u_textures") {
            gl_context.uniform1iv_with_i32_array(Some(location),
            (0..max_texture_units).collect::<Vec<i32>>().as_slice());
        } else { return Err("Couldn't find uniform 'u_textures'".into()); }
        
        // Allocate the vertex buffer's memory
        gl_context.buffer_data_with_i32(
            WebGlRenderingContext::ARRAY_BUFFER,
            MAX_QUAD_COUNT * VERTICES_PER_QUAD * FLOATS_PER_VERTEX * 4,
            WebGlRenderingContext::DYNAMIC_DRAW,
        );

        // Create the index buffer
        let index_buffer = gl_context
            .create_buffer()
            .ok_or("failed to create buffer")?;
        // Bind the index buffer
        // to the WebGL context.
        gl_context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));


        let mut indcies: Vec<u16> = Vec::new();
        for i in 0..MAX_QUAD_COUNT {
            // Copy data into the Vec<u16> 
            // from the slice which includes
            // the indcies which represent 
            // the order of the vertices'rendering.
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
            // Allocate the index buffer's memory,
            // and copy the indcies into it.
            let indcies_array = js_sys::Uint16Array::view(indcies.as_slice());
            gl_context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
                &indcies_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }
        // Create webgl white texture object
        let white_texture = gl_context.create_texture();
        // Activate texture unit 0 with this white texture
        gl_context.active_texture(WebGlRenderingContext::TEXTURE0);
        // bind the white texture to the webgl context
        gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, white_texture.as_ref());
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
        // Allocate memory for the texture slots
        // vector and add the white texture to it.
        let mut texture_slots: Vec<u32> = vec![0];
        texture_slots.reserve_exact((max_texture_units-1) as usize);

        // Allocate memory for the vertex vector
        let mut vertex_vec: Vec<f32> = Vec::new();
        vertex_vec.reserve_exact((MAX_QUAD_COUNT * VERTICES_PER_QUAD * FLOATS_PER_VERTEX) as usize);

        // Return the webgl renderer.
        Ok(Self{gl_context, gl_program, uniform_locations, vertex_buffer, index_buffer,
        vertex_vec, texture_slots, max_texture_units})
    }

    /// This function will render the
    /// scene using the provided game,\
    /// scene and object properties, and
    /// the provided asset definitions.
    /// 
    /// This function will also use the
    /// provided elapsed time to animate\
    /// the sprites of the scene's objects.
    pub fn render_scene(&mut self, game: &element::Game, scene_props: &element::Scene,
    object_stack: &Vec<engine_api::ElementHandler>, asset_defs: &AssetDefinitions, elapsed: f64)
     -> Result<(), JsValue> {
        // Use the scene rendering shader program.
        self.gl_context.use_program(Some(&self.gl_program));
        // Set the clear color.
        self.gl_context.clear_color(from_0_225_to_0_1(game.clear_red),
        from_0_225_to_0_1(game.clear_green), from_0_225_to_0_1(game.clear_blue), 1.0);

        // Clear the canvas
        self.gl_context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        // Set the blending method the alpha of the images will control
        self.gl_context.enable(WebGlRenderingContext::BLEND);
        self.gl_context.blend_func(WebGlRenderingContext::SRC_ALPHA, WebGlRenderingContext::ONE_MINUS_SRC_ALPHA);

        // Set the uniform values
        if let Some(location) = self.uniform_locations.get("u_camera") {
            self.gl_context.uniform2f(Some(location),
            scene_props.camera.position.x.floor(), scene_props.camera.position.y.floor());
        } else { return Err("Couldn't find uniform 'u_camera'".into()); }
        if let Some(location) = self.uniform_locations.get("u_zoom") {
            self.gl_context.uniform1f(Some(location), scene_props.camera.zoom);
        } else { return Err("Couldn't find uniform 'u_zoom'".into()); }
        if let Some(location) = self.uniform_locations.get("u_cam_color") {
            self.gl_context.uniform4f(Some(location), from_0_225_to_0_1(scene_props.camera.color.r),
            from_0_225_to_0_1(scene_props.camera.color.g),from_0_225_to_0_1(scene_props.camera.color.b),
            from_0_225_to_0_1(scene_props.camera.color.a));
        } else { return Err("Couldn't find uniform 'u_cam_color'".into()); }
        if let Some(location) = self.uniform_locations.get("u_resolution") {
            self.gl_context.uniform2f(Some(location), game.canvas_width as f32, game.canvas_height as f32);
        } else { return Err("Couldn't find uniform 'u_resolution'".into()); }

        // resize the canvas if needed
        {
            // Convert the canvas element into an HTMLCanvasElement object.
            let canvas: web_sys::HtmlCanvasElement = self.gl_context.canvas().unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;
            // Set the desired width and height of the canvas,
            // which is the size the canvas will be rendered
            // at regardless of how CSS displays it.
            canvas.set_attribute("width", &format!("{}", game.canvas_width))?;
            canvas.set_attribute("height", &format!("{}", game.canvas_height))?;
        } // `canvas` drops here.

        // Inform the webgl context of the canvas resize,
        // so that it can render the scene at the correct size.
        self.gl_context.viewport(0, 0, game.canvas_width as i32, game.canvas_height as i32);

        // This set of variables will store
        // data for different sprites in
        // each iteration of the loop, and
        // will be used to define the quads
        // which will make them show up on screen.
        let mut unit_id: f32;
        let mut tex_width: f32;
        let mut tex_height: f32;
        let mut quad_width: f32 = 1.0;
        let mut quad_height: f32 = 1.0;
        let mut texcoord_1: [f32; 2] = [0.0, 0.0];
        let mut texcoord_2: [f32; 2] = [0.0, 0.0];
        let mut origin_minus_offset: [f32; 2] = [0.0, 0.0];
        // Iterate over the scene's object
        // instances in the order of the
        // layers they are in and render them.
        for &index in scene_props.layers[0..scene_props.layers_len].iter()
        .flat_map(|layer| { layer.instances.iter() }) {
            // If the vertex vector is full,
            if (self.vertex_vec.len() as i32) / (FLOATS_PER_VERTEX * VERTICES_PER_QUAD * MAX_QUAD_COUNT) >= 1 {
                // flush all the data from the vertex vector
                // into the vertex buffer, draw the scene,
                // and clear the texture slots vector.
                self.flush();
            }
            // Get the object from the object stack.
            if let Some(object) = object_stack.get(index as usize) {
                // Get a mutable borrow of the object,
                // which will later switch to the sprite of the object.
                let object_or_sprite = Rc::clone(&object.properties);
                let mut object_or_sprite = object_or_sprite.borrow_mut();
                let mut object_or_sprite = object_or_sprite
                .write_lock::<element::Object>()
                .expect("write lock should succeed.");

                // Here the object will switch to a sprite.
                // This needs to be done because the sprite's
                // propertys are owned by the object, and borrowing
                // them at the same time with the mutable borrow of
                // the object will violate the borrowing rules.
                if object_or_sprite.sprites.len > 0 {
                    // Get the object's current sprite asset's id.
                    let cur_sprite_asset_id = object_or_sprite.sprites.cur_asset;
                    // Receive a mutable borrow of the active sprite.
                    let object_or_sprite = object_or_sprite.sprites.members
                    .get_mut(cur_sprite_asset_id)
                    .expect("the cur_asset index should exist in the members array of an AssetList.");
                    // Get a borrow (immutable) to the
                    // asset definition of the sprite.
                    if let Some(def) = asset_defs.get(&object_or_sprite.id) {
                        let texture_asset = def.as_ref()?;
                        // Get the asset data of the sprite,
                        // which is a `WebGlTexture`.
                        let gl_texture: &WebGlTexture;
                        match &texture_asset.asset_data {
                            AssetData::ImageData { width, height, texture } => {
                                tex_width = *width as f32;
                                tex_height = *height as f32;
                                gl_texture = texture;
                            }
                            //_ => { return Err("The asset data of an image asset wasn't image data.".into()); }
                        }
                        
                        // This variable will become
                        // true if the sprite has an
                        // animation with the same name
                        // as specified in the sprite's
                        // `cur_animation` property.
                        let mut found_anim = false;
                        // Iterate over the sprite's
                        // animations and find the
                        // one with the same name as
                        // specified in the sprite's
                        // `cur_animation` property.
                        for anim in &texture_asset.config["animations"]
                        .read_lock::<Vec<rhai::Dynamic>>().expect(concat!("Every sprite's config should have an",
                        " 'animations' array, which contains object-like members.")) as &Vec<rhai::Dynamic> {
                            let anim: &rhai::Map = &anim.read_lock::<rhai::Map>().expect(concat!("Every member of the 'animations'",
                            " array in a sprite's config should be an object-like member."));

                            if &anim["name"].read_lock::<rhai::ImmutableString>().expect(
                            concat!("Every member of the 'animations' array in a sprite's config should",
                            " have a 'name' property with a string.")) as &str != object_or_sprite.cur_animation {
                                continue;
                            }
                            // If the animation was found,
                            // set the `found_anim` variable
                            // to true and start getting all
                            // the necessary information for
                            // rendering the sprite.
                            found_anim = true;

                            // Get the sprite's animation
                            // frame rate from its config.
                            let fps = dynamic_to_number(&texture_asset.config["fps"])
                            .expect("Every sprite's config should have a 'fps' property with an integer number.") as i32;

                            // Get the animation's frames
                            let frames = &anim["frames"]
                            .read_lock::<Vec<rhai::Dynamic>>().expect(concat!("Every",
                            " member of the 'animations' array in a sprite's config should",
                            " have a 'frames' array, which contains object-like members.")) as &Vec<rhai::Dynamic>;
                            // If the animation is not finished,
                            if !object_or_sprite.is_animation_finished {
                                // Add the elapsed time to the
                                // animation time of the sprite.
                                object_or_sprite.animation_time += elapsed; 
                                // Set the current frame of the
                                // sprite's animation according
                                // to the animation time and the
                                // animation's frame rate.
                                object_or_sprite.cur_frame = ((fps as f64) * object_or_sprite.animation_time * 0.001).floor() as u32;
                                // If the current frame is out of
                                // the animation's frames range,
                                if object_or_sprite.cur_frame >= frames.len() as u32 {
                                    // Set the current frame to the
                                    // last frame of the animation.
                                    object_or_sprite.cur_frame = (frames.len() - 1) as u32;
                                    // Mark the animation as finished,
                                    // or calibrate the animation time
                                    // and current frame if the animation
                                    // should be repeated.
                                    if !object_or_sprite.repeat {
                                        object_or_sprite.is_animation_finished = true;
                                    } else {
                                        object_or_sprite.cur_frame = 0;
                                        object_or_sprite.animation_time = 0.0;
                                    }
                                }
                            }
                            // Get the current frame's properties
                            let current_frame: &rhai::Map = &frames[object_or_sprite.cur_frame as usize]
                            .read_lock::<rhai::Map>().expect("Every frame should be an object-like member.");
                            // Get the current frame's area
                            // coordinates on the sprite's texture.
                            let area: &rhai::Map = &current_frame["area"].read_lock::<rhai::Map>()
                            .expect("Every frame should have a 'area' object-like property");
                            // Convert the area coordinates
                            // into valid webgl texture coordinates.

                            // The first area point is
                            // relative to the top left
                            // corner of the texture.
                            texcoord_1 = [
                                dynamic_to_number(&area["x1"])
                                .expect("x1 should be a number.") / tex_width, 
                                dynamic_to_number(&area["y1"])
                                .expect("y1 should be a number.") / tex_height
                            ];
                            // The second area point is
                            // relative to the bottom right
                            // corner of the texture.
                            texcoord_2 = [
                                1.0 - (dynamic_to_number(&area["x2"])
                                .expect("x2 should be a number.") / tex_width),
                                1.0 - (dynamic_to_number(&area["y2"])
                                .expect("y2 should be a number.") / tex_height)
                            ];

                            // Calculate the width and height
                            // of the quad which will be used
                            // to render the sprite.

                            // Subtract the area x coordinates
                            // of the two points from the
                            // texture's width to get the
                            // width of the quad.
                            quad_width = tex_width - (
                                dynamic_to_number(&area["x1"])
                                .expect("x1 should be a number.") +
                                dynamic_to_number(&area["x2"])
                                .expect("x2 should be a number.")
                            );
                            // Subtract the area y coordinates
                            // of the two points from the
                            // texture's height to get the
                            // height of the quad.
                            quad_height = tex_height - (
                                dynamic_to_number(&area["y1"])
                                .expect("y1 should be a number.") +
                                dynamic_to_number(&area["y2"])
                                .expect("y2 should be a number.")
                            );
                            
                            // Get the offset of the current
                            // frame from the sprite's config.
                            let offset: &rhai::Map = &current_frame["offset"].read_lock::<rhai::Map>()
                            .expect("Every frame should have a 'offset' object-like property");
                            // Get the origin point of the
                            // sprite from its config.
                            let origin = &texture_asset.config["origin"]
                            .read_lock::<rhai::Map>().expect(concat!("Every sprite's config should",
                            " have a 'origin' object-like property")) as &rhai::Map;
                            // Calculate the final origin
                            // point of the sprite, considering
                            // the offset of the current frame.
                            origin_minus_offset = [
                                dynamic_to_number(&origin["x"])
                                .expect("origin.x should be a number") -
                                dynamic_to_number(&offset["x"])
                                .expect("offset.x should be a number"),
                                dynamic_to_number(&origin["y"])
                                .expect("origin.y should be a number") -
                                dynamic_to_number(&offset["y"])
                                .expect("offset.y should be a number")
                            ];
                            // Break the loop
                            // and start trying
                            // to render the sprite.
                            break;
                        }

                        // If the animation wasn't found,
                        if !found_anim {
                            // Skip this object
                            continue;
                        }

                        // If the sprite's texture is already
                        // in the texture slots vector,
                        if let Some((idx, _)) = self.texture_slots.iter()
                        .enumerate().find(|&(_, &slot)| { object_or_sprite.id == slot }) {
                            // The texture unit at that index
                            // contains the sprite's texture, so
                            // we should use that unit to render
                            // the sprite.
                            unit_id = idx as f32;
                        } else if self.texture_slots.len() < (self.max_texture_units as usize) {
                            // If the sprite's texture is not
                            // in the texture slots vector, but
                            // there's still room for it, bind
                            // the texture to the first free
                            // texture unit, add the texture's
                            // id to the texture slots vector,
                            // and use that unit to render the sprite.
                            self.gl_context.active_texture(WebGlRenderingContext::TEXTURE0 + (self.texture_slots.len() as u32));
                            self.gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(gl_texture));
                            self.texture_slots.push(object_or_sprite.id);
                            unit_id = (self.texture_slots.len() as f32)-1_f32;
                        } else {
                            // If there's no room for the sprite's
                            // texture in the texture slots vector,
                            // flush all the data from the vertex vector
                            // into the vertex buffer, draw the scene,
                            // and clear the texture slots vector.
                            self.flush();
                            // Bind the texture to the first
                            // texture unit, add the texture's
                            // id to the texture slots vector,
                            // and use that unit to render the sprite.
                            self.gl_context.active_texture(WebGlRenderingContext::TEXTURE0 + (self.texture_slots.len() as u32));
                            self.gl_context.bind_texture(WebGlRenderingContext::TEXTURE_2D, Some(gl_texture));
                            self.texture_slots.push(object_or_sprite.id);
                            unit_id = (self.texture_slots.len() as f32)-1_f32;
                        }
                    } else if object_or_sprite.id == 0 {
                        // If the asset definition of the sprite
                        // couldn't be found, but the specified
                        // sprite id is 0, render a colored quad
                        // using our white texture.
                        unit_id = 0.0; tex_width = 1.0;
                        tex_height = 1.0; quad_width = 1.0;
                        quad_height = 1.0; texcoord_1 = [0.0, 0.0];
                        texcoord_2 = [0.0, 0.0]; origin_minus_offset = [0.0, 0.0];
                    } else {
                        // If the sprite's id isn't
                        // 0, skip this object.
                        continue;
                    }
                } else {
                    // If the object doesn't
                    // have any sprites, skip it
                    continue; 
                }// Here the sprite switches back to being an object.

                // Generate the quad which will be used
                // to render the sprite, and add it to
                // the vertex vector.
                self.vertex_vec.extend_from_slice(&generate_textured_quad(object_or_sprite.position.x.floor() - 
                (origin_minus_offset[0] * object_or_sprite.scale.x), object_or_sprite.position.y.floor() - 
                (origin_minus_offset[1] * object_or_sprite.scale.y), [from_0_225_to_0_1(object_or_sprite.color.r),
                from_0_225_to_0_1(object_or_sprite.color.g),from_0_225_to_0_1(object_or_sprite.color.b),
                from_0_225_to_0_1(object_or_sprite.color.a)], quad_width * object_or_sprite.scale.x,
                quad_height * object_or_sprite.scale.y, texcoord_1, texcoord_2,
                [tex_width, tex_height], [object_or_sprite.scale.x, object_or_sprite.scale.y],
                unit_id));
            }
        }
        // Flush all the data that's left in the
        // vertex vector into the vertex buffer,
        // draw the scene, and clear the texture
        // slots vector.
        self.flush();
        // The rendering proccess
        // for a single frame ends here.
        Ok(())
    }

    /// This function will be used to
    /// flush the vertex vector into the
    /// vertex buffer and draw the scene.
    fn flush(&mut self) {
        // Bind the vertex buffer
        // to the WebGL context.
        self.gl_context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
        // Bind the index buffer
        // to the WebGL context.
        self.gl_context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.

        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let vertices_array = js_sys::Float32Array::view(self.vertex_vec.as_slice());
            self.gl_context.buffer_sub_data_with_f64_and_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                0.0,
                &vertices_array,
            );
        }
        // Draw the scene rectangle on the canvas
        self.gl_context.draw_elements_with_i32(
            WebGlRenderingContext::TRIANGLES,
            (self.vertex_vec.len() as i32 * INDCIES_PER_QUAD) / (VERTICES_PER_QUAD * FLOATS_PER_VERTEX),
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
        // Clear the vertex vector
        // and the texture slots vector.
        self.vertex_vec.clear();
        self.texture_slots.truncate(1);
    }
}

/// This function will find the
/// canvas element in the page,\
/// set its width and height,
/// and return its WebGL context.\
/// If the canvas element was already
/// used, it will return an error.
fn activate_context(width: f32, height: f32) -> Result<WebGlRenderingContext, JsValue> {
    // Don't allow the canvas webgl
    // context to be used more than once.
    if unsafe { CANVAS_ID.is_empty() } {
        return Err("The canvas webgl context was already used.".into());
    }
    // Get the page's document.
    let document = web_sys::window().unwrap().document().unwrap();
    // Get to canvas from the document.
    let canvas = document.get_element_by_id(unsafe{ CANVAS_ID }).unwrap();
    // Sets the canvas id to an empty string,
    // so that it can't be used again.
    unsafe { CANVAS_ID = ""; }
    // Convert the canvas element into an HTMLCanvasElement object.
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    // Set the desired width and height of the canvas,
    // which is the size the canvas will be rendered
    // at regardless of how CSS displays it.
    canvas.set_attribute("width", &format!("{width}"))?;
    canvas.set_attribute("height", &format!("{height}"))?;

    // Get the WebGL context
    // from the canvas.
    let context = canvas
    .get_context_with_context_options("webgl",
        WebGlContextAttributes::new()
        .alpha(false)
        .premultiplied_alpha(true)
        .dyn_ref::<JsValue>().unwrap()
    )?
    .unwrap()
    .dyn_into::<WebGlRenderingContext>()?;
    // Enable premultiplied alpha unpacking,
    // which will make webgl premultiply the
    // alpha channel of the image data for us.
    context.pixel_storei(WebGlRenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);
    // Return the WebGL context.
    Ok(context)
}

/// This function will compile
/// the vertex and fragment shaders\
/// and link them into a shader program.
/// It will also bind the attribute locations\
/// to the shader program and look up the
/// uniform locations. The shader program\
/// and the uniform locations will be returned.
fn create_program(gl_context: &WebGlRenderingContext)
 -> Result<(WebGlProgram,HashMap<String, web_sys::WebGlUniformLocation>), JsValue> {
    // Create the vertex shader
    let vert_shader = compile_shader(
        &gl_context,
        WebGlRenderingContext::VERTEX_SHADER,
        VERTEX_SHADER,
    )?;

    // Create the fragment shader
    let frag_shader = compile_shader(
        &gl_context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        FRAGMENT_SHADER,
    )?;

    // Create the shader program using
    // the vertex and fragment shaders.
    let gl_program = link_program(&gl_context, &vert_shader, &frag_shader,
    Some(|gl_context: &WebGlRenderingContext, gl_program: &WebGlProgram| {
        // Bind attribute locations.
        for (index, &(attribute,_,_)) in ATTRIBUTE_MATRIX.iter().enumerate() {
            gl_context.bind_attrib_location(&gl_program, index as u32, attribute);
        }
    }))?;

    // Look up uniform locations.
    let mut uniform_locations: HashMap<String, web_sys::WebGlUniformLocation> = HashMap::new();
    for &uniform in UNIFORM_LIST.iter() {
        uniform_locations.insert(String::from(uniform), gl_context
            .get_uniform_location(&gl_program, uniform)
            .ok_or(format!("Unable to get uniform location ({})", uniform))?
        );
    }

    // Return the shader program
    // and the uniform locations.
    Ok((gl_program, uniform_locations))
}

/// Creates an array of f32 floats with
/// the vertices which should represent\
/// a desired textured rectangle.
fn generate_textured_quad(x: f32, y: f32, color: [f32; 4],
width: f32, height: f32, texpoint_1: [f32; 2],
texpoint_2: [f32; 2], tex_size: [f32; 2],
scale: [f32; 2], texunit_id: f32) -> [f32; (VERTICES_PER_QUAD * FLOATS_PER_VERTEX) as usize] {
    let x1 = x;
    let x2 = x + width;
    let y1 = y;
    let y2 = y + height;
    //  x, y, red, green, blue, alpha, texture_x(0-1), texture_y(0-1),
    //  tex_width, tex_height, scale_x, scale_y, texture_unit_id
    [
        x1, y1, color[0], color[1], color[2], color[3], texpoint_1[0], texpoint_1[1],
        tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
        x2, y1, color[0], color[1], color[2], color[3], texpoint_2[0], texpoint_1[1],
        tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
        x1, y2, color[0], color[1], color[2], color[3], texpoint_1[0], texpoint_2[1],
        tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
        x2, y2, color[0], color[1], color[2], color[3], texpoint_2[0], texpoint_2[1],
        tex_size[0], tex_size[1], scale[0], scale[1], texunit_id,
    ]
}

/// Receives a byte and makes it
/// go from 0 to 1 instead of from\
/// 0 to 255, for use with the WebGL context.
fn from_0_225_to_0_1(color: u8) -> f32 {
    return (color as f32) / 255_f32; 
}

/// Compiles a shader and
/// returns a WebGlShader\
/// object if the compilation
/// is successful. otherwise,\
/// the error log is returned.
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

/// Links a new program
/// to a WebGl context and\
/// returns a WebGlProgram
/// object if the linking\
/// is successful. otherwise,
/// the error log is returned.
fn link_program(
    gl_context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
    before_link: Option<impl Fn(&WebGlRenderingContext, &WebGlProgram) -> ()>,
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
    // Call the provided function
    // before linking the program.
    if let Some(before_link) = before_link {
        before_link(&gl_context, &program);
    }
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