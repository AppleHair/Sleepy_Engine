
use std::collections::HashMap;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlBuffer};

use crate::{data, game::TableRow};

use super::engine_api::{element, asset, self};

const MAX_QUAD_COUNT: i32 = 1000;
const INDCIES_PER_QUAD: i32 = 6;
const VERTICES_PER_QUAD: i32 = 4;
const FLOATS_PER_VERTEX: i32 = 6;

pub enum ProgramDataLocation {
    Attribute(u32),
    Uniform(web_sys::WebGlUniformLocation),
}

//
pub enum AssetData {
    ImageData{width: i32, height: i32, pixels: Vec<u8>},
}

//
impl AssetData {
    pub fn new_image_data(id: u32) -> AssetData {
        // Use the id to get the image png data.
        let image_data = data::get_asset_data(id);

        // Extract the image's width and height from the png data.
        let width = i32::from_be_bytes([image_data[16], image_data[17], image_data[18], image_data[19]]);
        let height = i32::from_be_bytes([image_data[20], image_data[21], image_data[22], image_data[23]]);
        // Extract the image's pixel data from the png data using the
        // 'image' crate, while also flipping the image vertically.
        let pixels = image::load_from_memory_with_format(&image_data,
        image::ImageFormat::Png).expect("Couldn't load PNG file.")
        .flipv().to_rgba8().pixels().map(|p| p.0).collect::<Vec<[u8; 4]>>().concat();
        // Return the image data.
        AssetData::ImageData{width, height, pixels}
    }
    //
    pub fn recycle_image(&mut self, id: u32) {
        //
        match self {
            AssetData::ImageData{ width, height, pixels } => {
                // Use the id to get the new image png data.
                let new_data = data::get_asset_data(id);
                // Extract the image's width and height from the png data.
                *width = i32::from_be_bytes([new_data[16], new_data[17], new_data[18], new_data[19]]);
                *height = i32::from_be_bytes([new_data[20], new_data[21], new_data[22], new_data[23]]);
                // Clear the old pixel data.
                pixels.clear();
                // Extract the image's pixel data from the png data using the
                // 'image' crate, while also flipping the image vertically.
                pixels.extend(image::load_from_memory_with_format(&new_data,
                image::ImageFormat::Png).expect("Couldn't load PNG file.")
                .flipv().to_rgba8().pixels().map(|p| p.0).collect::<Vec<[u8; 4]>>().concat().into_iter());
            },
            //_ => (),
        }
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
    pub fn new(engine: &rhai::Engine, row: TableRow) -> Result<Self, String> {
        //
        let asset_data = match row {
            TableRow::Asset(id, 1) => AssetData::new_image_data(id),
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
            return Err(row.to_err_string(&err.to_string()));
        }
        //
        Ok(
            Self {
                //
                row,
                //
                asset_data,
                //
                config: json.expect("This Err should have been caught by this function beforehand"),
            }
        )
    }
    //
    pub fn recycle(&mut self, engine: &rhai::Engine, row: TableRow) -> Result<(), String> {
        self.row = row;
        //
        match self.row {
            TableRow::Asset(id, 1) => self.asset_data.recycle_image(id),
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
            return Err(row.to_err_string(&err.to_string()));
        }
        //
        self.config = json.expect("This Err should have been caught by this function beforehand");
        //
        Ok(())
    }
}

//
pub fn create_rendering_components(canvas_width: i32, canvas_height: i32)
 -> Result<(WebGlRenderingContext, WebGlProgram,
HashMap<String, ProgramDataLocation>, WebGlBuffer, WebGlBuffer), JsValue> {
    //
    let gl = create_context(
        canvas_width, 
        canvas_height
    )?;
    //
    let (gl_program, 
    program_data) = 
        create_scene_rendering_program(&gl)?;
    // 
    let vertex_buffer = gl
        .create_buffer()
        .ok_or("failed to create buffer")?;
    //
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    //
    gl.buffer_data_with_i32(
        WebGlRenderingContext::ARRAY_BUFFER,
        MAX_QUAD_COUNT * VERTICES_PER_QUAD * FLOATS_PER_VERTEX * 4,
        WebGlRenderingContext::DYNAMIC_DRAW,
    );

    //
    let index_buffer = gl
        .create_buffer()
        .ok_or("failed to create buffer")?;
    //
    gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

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
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &indcies_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
    //
    Ok((gl, gl_program, program_data, vertex_buffer, index_buffer))
}

//
pub fn render_scene(context: &WebGlRenderingContext, program: &WebGlProgram,
data_locations: &HashMap<String,ProgramDataLocation>, vertex_buffer: &web_sys::WebGlBuffer,
index_buffer: &web_sys::WebGlBuffer, scene: &element::Scene, asset_defs: &HashMap<u32, AssetDefinition>,
object_stack: &Vec<engine_api::Element<engine_api::Object>>, elapsed: f64) -> Result<(), JsValue> {
    // Use the scene rendering shader program.
    context.use_program(Some(&program));

    // Create a vertex array for the scene rectangle.
    let vertices = generate_colored_rectangle_vertex(0.0, 0.0,
    scene.width.floor(), scene.height.floor(), hex_color_to_rgba(&scene.in_color));

    // Bind the vertex buffer
    // to the WebGL context.
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    // Bind the index buffer
    // to the WebGL context.
    context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

    // Copy the data form the vertex array
    // into the vertex array buffer.
    context.buffer_sub_data_with_i32_and_array_buffer(
        WebGlRenderingContext::ARRAY_BUFFER,
        0,
        &vertices.buffer(),
    );

    //
    let position_attribute_location: u32;
    //
    if let ProgramDataLocation::Attribute(idx) = data_locations
    .get("a_position")
    .ok_or("Couldn't find attribute 'a_position'")? {
        //
        position_attribute_location = idx.clone();
    } else {
        //
        return Err("Couldn't find attribute 'a_position'".into());
    }
    // Enable the attribute-reading method
    context.enable_vertex_attrib_array(position_attribute_location);
    // Tell the GPU how to read the vertex buffer by attributes
    context.vertex_attrib_pointer_with_i32(position_attribute_location, 
    2, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 0);

    //
    let color_attribute_location: u32;
    //
    if let ProgramDataLocation::Attribute(idx) = data_locations
    .get("a_color")
    .ok_or("Couldn't find attribute 'a_color'")? {
        //
        color_attribute_location = idx.clone();
    } else {
        //
        return Err("Couldn't find attribute 'a_color'".into());
    }
    // Enable the attribute-reading method
    context.enable_vertex_attrib_array(color_attribute_location);
    // Tell the GPU how to read the vertex buffer by attributes
    context.vertex_attrib_pointer_with_i32(color_attribute_location, 
    4, WebGlRenderingContext::FLOAT, false, FLOATS_PER_VERTEX * 4, 8);
    
    //
    let resolution_uniform_location: web_sys::WebGlUniformLocation;
    //
    if let ProgramDataLocation::Uniform(loc) = data_locations
    .get("u_resolution")
    .ok_or("Couldn't find uniform 'u_resolution'")? {
        //
        resolution_uniform_location = loc.clone();
    } else {
        //
        return Err("Couldn't find uniform 'u_resolution'".into());
    }
    //
    let canvas = context.canvas().unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;
    // 
    context.uniform2f(Some(&resolution_uniform_location), canvas.width() as f32, canvas.height() as f32);

    //
    let camera_uniform_location: web_sys::WebGlUniformLocation;
    //
    if let ProgramDataLocation::Uniform(loc) = data_locations
    .get("u_camera")
    .ok_or("Couldn't find uniform 'u_camera'")? {
        //
        camera_uniform_location = loc.clone();
    } else {
        //
        return Err("Couldn't find uniform 'u_camera'".into());
    }
    //
    context.uniform2f(Some(&camera_uniform_location), scene.camera.position.x.round(), scene.camera.position.y.round());

    {
        // Get the outside-color of the stage.
        let outcolor = hex_color_to_rgba(&scene.out_color);
        // Set the clear color to the outside color.
        context.clear_color(outcolor[0], outcolor[1], outcolor[2], outcolor[3]);
    }// 'outcolor' drops here

    // Clear the canvas
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    // Draw the scene rectangle on the canvas
    context.draw_elements_with_i32(
        WebGlRenderingContext::TRIANGLES,
        (vertices.length() as i32 * INDCIES_PER_QUAD) / (VERTICES_PER_QUAD * FLOATS_PER_VERTEX),
        WebGlRenderingContext::UNSIGNED_SHORT,
        0,
    );

    // The Rendering is Done!
    Ok(())
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
    let context = canvas
    .get_context("webgl")?
    .unwrap()
    .dyn_into::<WebGlRenderingContext>()?;

    //
    Ok(context)
}

fn create_scene_rendering_program(context: &WebGlRenderingContext)
 -> Result<(WebGlProgram,HashMap<String,ProgramDataLocation>), JsValue> {
    // Create the vertex shader
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec2 a_position;
        attribute vec4 a_color;

        uniform vec2 u_resolution;
        uniform vec2 u_camera;

        varying vec4 v_color;

        void main() {

            v_color = a_color;

            vec2 camRelative = a_position - u_camera;
            vec2 clipSpace = camRelative * 2.0 / u_resolution;
            gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
        }
    "#,
    )?;

    // Create the fragment shader
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        precision mediump float;

        varying vec4 v_color;

        void main() {
            gl_FragColor = v_color;
        }
    "#,
    )?;

    // Create the shader program using
    // the vertex and fragment shaders.
    let program = link_program(&context, &vert_shader, &frag_shader)?;

    //
    let mut data_locations: HashMap<String,ProgramDataLocation> = HashMap::new();

    // Look up attribute locations.
    data_locations.insert(String::from("a_position"), ProgramDataLocation::Attribute(context
        .get_attrib_location(&program, "a_position") as u32
    ));
    data_locations.insert(String::from("a_color"), ProgramDataLocation::Attribute(context
        .get_attrib_location(&program, "a_color") as u32
    ));

    // Look up uniform locations.
    data_locations.insert(String::from("u_resolution"), ProgramDataLocation::Uniform(context
        .get_uniform_location(&program, "u_resolution")
        .ok_or("Unable to get uniform location (u_resolution)")?
    ));
    data_locations.insert(String::from("u_camera"), ProgramDataLocation::Uniform(context
        .get_uniform_location(&program, "u_camera")
        .ok_or("Unable to get uniform location (u_camera)")?
    ));

    Ok((program, data_locations))
}

// Creates a Float32Array with the 
// vertices which should represent a 
// desired colored rectangle, while only using
// 5 arguments: x, y, width, height and color.
fn generate_colored_rectangle_vertex(x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) -> js_sys::Float32Array {
    // Create a new Float32Array with the size of 8 elements.
    let vertices = js_sys::Float32Array::new(
    &JsValue::from_f64((VERTICES_PER_QUAD * FLOATS_PER_VERTEX) as f64));
    
    // Copy data into the Float32Array
    // from the rust slice which returns
    // from this block.
    vertices.copy_from(&{
        let x1 = x;
        let x2 = x + width;
        let y1 = y;
        let y2 = y + height;
        // The vertices which
        // should represent the
        // rectangle's points.
        [
            x1, y1, color[0], color[1], color[2], color[3],
            x2, y1, color[0], color[1], color[2], color[3],
            x1, y2, color[0], color[1], color[2], color[3],
            x2, y2, color[0], color[1], color[2], color[3],
        ]
    });
    
    // Return the
    // Float32Array
    vertices
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
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    // Create a shader with the
    // provided WebGL context.
    let shader = context
        .create_shader(shader_type)
        .ok_or(String::from("Unable to create shader"))?;
    // Attach the provided GLSL
    // source to the shader.
    context.shader_source(&shader, source);
    // Compile the GLSL source
    // attached to the shader.
    context.compile_shader(&shader);

    // Check the shader's compile status
    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // Return the shader if it's compiled.
        Ok(shader)
    } else {
        // Return the compilation error log
        // if the shader isn't compiled.
        Err(context
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
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    // Create a program with the
    // provided WebGL context.
    let program = context
        .create_program()
        .ok_or(String::from("Unable to create program"))?;
    // Attach the provided vertex
    // and fragment shaders to the program.
    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    // Link the program to the
    // provided WebGL context.
    context.link_program(&program);

    // Check the program's link status
    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        // Return the program if it's linked.
        Ok(program)
    } else {
        // Return the linking error log
        // if the program isn't linked.
        Err(context
            .get_program_info_log(&program)
            .unwrap_or(String::from("Unknown error occurred while creating shader program")))
    }
}