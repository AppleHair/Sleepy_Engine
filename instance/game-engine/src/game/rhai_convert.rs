use rhai::Dynamic;

pub fn dynamic_to_f64(dynam: &Dynamic) -> Result<f64, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f64);
    }
    Ok(dynam.as_float()? as f64)
}

pub fn dynamic_to_i64(dynam: &Dynamic) -> Result<i64, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as i64);
    }
    Ok(dynam.as_float()? as i64)
}

pub fn dynamic_to_u64(dynam: &Dynamic) -> Result<u64, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as u64);
    }
    Ok(dynam.as_float()? as u64)
}

pub fn dynamic_to_f32(dynam: &Dynamic) -> Result<f32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as f32);
    }
    Ok(dynam.as_float()? as f32)
}

pub fn dynamic_to_i32(dynam: &Dynamic) -> Result<i32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as i32);
    }
    Ok(dynam.as_float()? as i32)
}

pub fn dynamic_to_u32(dynam: &Dynamic) -> Result<u32, &str> {
    if dynam.is_int() { 
        return Ok(dynam.as_int()? as u32);
    }
    Ok(dynam.as_float()? as u32)
}