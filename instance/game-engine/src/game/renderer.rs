use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlBuffer};

use super::engine_api::element;

const MAX_QUAD_COUNT: i32 = 1000;
const INDCIES_PER_QUAD: i32 = 6;
const VERTICES_PER_QUAD: i32 = 4;

pub enum ProgramDataLocation {
    Attribute(u32),
    Uniform(web_sys::WebGlUniformLocation),
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
        MAX_QUAD_COUNT * VERTICES_PER_QUAD * 8,
        WebGlRenderingContext::DYNAMIC_DRAW,
    );

    //
    let index_buffer = gl
        .create_buffer()
        .ok_or("failed to create buffer")?;
    //
    gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    //
    gl.buffer_data_with_i32(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        MAX_QUAD_COUNT * INDCIES_PER_QUAD * 2,
        WebGlRenderingContext::DYNAMIC_DRAW,
    );

    //
    let mut indcies: js_sys::Uint16Array;
    for i in 0..MAX_QUAD_COUNT {
        // Create a new Uint16Array with the size of 6 elements.
        indcies = js_sys::Uint16Array::new(
            &JsValue::from_f64(INDCIES_PER_QUAD as f64));
        // Copy data into the Uint16Array
        // from the rust slice which includes
        // the indcies which represent the order
        // of the vertices' rendering.
        indcies.copy_from(&[
            (0 + VERTICES_PER_QUAD * i) as u16,
            (1 + VERTICES_PER_QUAD * i) as u16,
            (2 + VERTICES_PER_QUAD * i) as u16,
            (2 + VERTICES_PER_QUAD * i) as u16,
            (1 + VERTICES_PER_QUAD * i) as u16,
            (3 + VERTICES_PER_QUAD * i) as u16,
        ]);
        //
        gl.buffer_sub_data_with_i32_and_array_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            INDCIES_PER_QUAD * i * 2,
            &indcies.buffer(),
        );
    }
    //
    Ok((gl, gl_program, program_data, vertex_buffer, index_buffer))
}

//
pub fn render_scene(context: &WebGlRenderingContext, program: &WebGlProgram,
data_locations: &HashMap<String,ProgramDataLocation>, vertex_buffer: &web_sys::WebGlBuffer,
index_buffer: &web_sys::WebGlBuffer, scene: &element::Scene) -> Result<(), JsValue> {
    // Use the scene rendering shader program.
    context.use_program(Some(&program));

    // Create a vertex and index arrays for the scene rectangle.
    let (vertices, _) = generate_rectangle(0.0, 0.0, scene.width.floor(), scene.height.floor());

    // Bind the vertex buffer
    // to the WebGL context.
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));

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
    // Tell the GPU how to read the vertex buffer by attributes
    context.vertex_attrib_pointer_with_i32(position_attribute_location, 
    2, WebGlRenderingContext::FLOAT, false, 0, 0);
    // Enable the attribute-reading method
    context.enable_vertex_attrib_array(position_attribute_location);
    
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

    //
    let incolor_uniform_location: web_sys::WebGlUniformLocation;
    //
    if let ProgramDataLocation::Uniform(loc) = data_locations
    .get("u_incolor")
    .ok_or("Couldn't find uniform 'u_incolor'")? {
        //
        incolor_uniform_location = loc.clone();
    } else {
        //
        return Err("Couldn't find uniform 'u_incolor'".into());
    }
    //
    context.uniform3fv_with_f32_array(Some(&incolor_uniform_location), &hex_color_to_rgb(&scene.in_color));

    {
        // Get the outside-color of the stage.
        let outcolor = hex_color_to_rgb(&scene.out_color);
        // Set the clear color to the outside color.
        context.clear_color(outcolor[0], outcolor[1], outcolor[2], 1.0);
    }// 'outcolor' drops here

    // Clear the canvas
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    // Draw the scene rectangle on the canvas
    context.draw_elements_with_i32(
        WebGlRenderingContext::TRIANGLES,
        (vertices.length() as i32 * INDCIES_PER_QUAD) / VERTICES_PER_QUAD * 2,
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
        uniform vec2 u_resolution;
        uniform vec2 u_camera;
        void main() {
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
        uniform vec3 u_incolor;
        void main() {
            gl_FragColor = vec4(u_incolor, 1);
        }
    "#,
    )?;

    // Create the shader program using
    // the vertex and fragment shaders.
    let program = link_program(&context, &vert_shader, &frag_shader)?;

    //
    let mut data_locations: HashMap<String,ProgramDataLocation> = HashMap::new();

    // Look up position attribute location.
    data_locations.insert(String::from("a_position"), ProgramDataLocation::Attribute(context
        .get_attrib_location(&program, "a_position") as u32
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
    data_locations.insert(String::from("u_incolor"), ProgramDataLocation::Uniform(context
        .get_uniform_location(&program, "u_incolor")
        .ok_or("Unable to get uniform location (u_incolor)")?
    ));

    Ok((program, data_locations))
}

// Creates a Float32Array and a Uint32Array with the 
// vertices and indcies which should represent a 
// desired rectangle, while only using 4 arguments: 
// x, y, width and height.
fn generate_rectangle(x: f32, y: f32, width: f32, height: f32) -> (js_sys::Float32Array, js_sys::Uint16Array) {
    // Create a new Float32Array with the size of 8 elements.
    let vertices = js_sys::Float32Array::new(
    &JsValue::from_f64(8.0));
    
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
            x1, y1,
            x2, y1,
            x1, y2,
            x2, y2,
        ]
    });

    // Create a new Uint32Array with the size of 6 elements.
    let indcies = js_sys::Uint16Array::new(
        &JsValue::from_f64(6.0));
    // Copy data into the Uint32Array
    // from the rust slice which includes
    // the indcies which represent the order
    // of the vertices' rendering.
    indcies.copy_from(&[0, 1, 2, 2, 1, 3]);
    
    // Return the
    // Float32Array
    // and Uint32Array
    (vertices, indcies)
}

// Receives a string borrow with a
// hex color code (#RRGGBB), and
// converts it into a slice of floats,
// for use with the WebGL context.
fn hex_color_to_rgb(hex: &str) -> [f32; 3] {
    // Result slice with
    // place-holder values
    let mut result: [f32; 3] = [0.0, 0.0, 0.0];
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