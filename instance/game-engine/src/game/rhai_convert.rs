use rhai::Dynamic;

pub fn dynamic_to_number(dynam: &Dynamic) -> Result<f64, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f64);
    }
    Ok(dynam.as_float()? as f64)
}