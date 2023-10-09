
use rhai::Dynamic;

//
#[derive(Clone)]
pub struct Sprite {
    pub id: u32,
    pub cur_animation: String,
    pub cur_frame: u32,
    pub is_animation_finished: bool,
    pub animation_time: f64,
    pub repeat: bool,
}

impl Sprite {
    pub fn new(new_id: u32) -> Self { Self {
        id: new_id, cur_animation: String::new(),
        cur_frame: 0_u32, is_animation_finished: true,
        animation_time: 0_f64, repeat: false
    } }
    pub fn recycle(&mut self, new_id: u32) {
        self.id = new_id; self.cur_animation.clear();
        self.cur_frame = 0_u32; self.is_animation_finished = true;
        self.animation_time = 0_f64; self.repeat = false;
    }
    pub fn get_id(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_cur_animation(&mut self) -> String { self.cur_animation.clone() }
    pub fn get_cur_frame(&mut self) -> rhai::INT { self.cur_frame.clone() as rhai::INT }
    pub fn get_is_animation_finished(&mut self) -> bool { self.is_animation_finished.clone() }
    pub fn get_animation_time(&mut self) -> rhai::FLOAT { self.animation_time.clone() as rhai::FLOAT }
    pub fn get_repeat(&mut self) -> bool { self.repeat.clone() }

    //
    pub fn play_animation(&mut self, name: &str) {
        //
        self.cur_animation.clear();
        //
        self.cur_animation.push_str(name);
        //
        self.cur_frame = 0;
        //
        self.animation_time = 0.0;
        //
        self.repeat = false;
        //
        self.is_animation_finished = false;
    }

    //
    pub fn play_animation_on_time(&mut self, name: &str, time: rhai::FLOAT) {
        //
        self.cur_animation.clear();
        //
        self.cur_animation.push_str(name);
        //
        self.cur_frame = 0;
        //
        self.animation_time = time as f64;
        //
        self.repeat = false;
        //
        self.is_animation_finished = false;
    }

    pub fn set_repeat(&mut self, value: bool) { self.repeat = value; }

    pub fn set_cur_animation(&mut self, value: &str) -> Result<(), Box<rhai::EvalAltResult>> {
        //
        if !self.is_animation_finished {
            return Err(concat!("Tried to set the current animation while an animation was playing.",
            " Note: To start playing another animation while an animation is playing, use the",
            " 'play_animation' method, and if you want it to loop, provide the optional 'repeat'",
            " boolean parameter.").into());
        }
        //
        self.cur_animation.clear();
        //
        self.cur_animation.push_str(value);
        //
        Ok(())
    }

    pub fn set_cur_frame(&mut self, value: rhai::INT) -> Result<(), Box<rhai::EvalAltResult>> {
        //
        if !self.is_animation_finished {
            return Err(concat!("Tried to set the current frame while an animation was playing.",
            " Note: To start playing another animation while an animation is playing, use the",
            " 'play_animation' method, and if you want it to loop, provide the optional 'repeat'",
            " boolean parameter.").into());
        }
        //
        self.cur_frame = value as u32;
        //
        Ok(())
    }

    //
    pub fn to_string(&mut self) -> String {
        //
        format!(
        "Sprite {{\n\tid: {}, cur_animation: \"{}\", cur_frame: {},\n\tis_animation_finished: {}, animation_time: {}, repeat: {} \n}}",
        self.id, self.cur_animation, self.cur_frame, self.is_animation_finished, self.animation_time, self.repeat)
    }
}

//
#[allow(dead_code)]
#[derive(Clone)]
pub struct Audio {
    pub id: u32,
    pub tag: String,
    pub own_tag: bool,
    pub replay: bool,
    pub volume: f32,
    pub paused: bool,
    pub repeat: bool,
    pub repeat_start_time: f64,
    pub audio_time: f64,
}

#[allow(dead_code)]
impl Audio {
    pub fn new(new_id: u32) -> Self { Self {
        id: new_id, tag: String::new(),
        audio_time: 0_f64, paused: false,
        repeat: false, volume: 1_f32, replay: false,
        repeat_start_time: 0_f64, own_tag: false,
    } }
    pub fn recycle(&mut self, new_id: u32) {
        self.id = new_id; self.tag.clear(); 
        self.audio_time = 0_f64; self.paused = false;
        self.repeat = false; self.volume = 1_f32; self.replay = false;
        self.repeat_start_time = 0_f64; self.own_tag = false;
    }
    pub fn get_id(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_tag(&mut self) -> String { self.tag.clone() }
    pub fn get_audio_time(&mut self) -> rhai::FLOAT { self.audio_time.clone() as rhai::FLOAT }
    pub fn get_paused(&mut self) -> bool { self.paused.clone() }
    pub fn get_repeat(&mut self) -> bool { self.repeat.clone() }
    pub fn get_repeat_start_time(&mut self) -> rhai::FLOAT { self.repeat_start_time.clone() as rhai::FLOAT }
    pub fn get_volume(&mut self) -> rhai::FLOAT { self.volume.clone() as rhai::FLOAT }

    pub fn play(&mut self) {
        //
        self.audio_time = 0.0;
        //
        self.paused = false;
        //
        self.replay = true;
    }

    pub fn obtain_tag(&mut self, tag: &str) {
        //
        self.tag.clear();
        //
        self.tag.push_str(tag);
        //
        self.own_tag = false;
    }

    pub fn set_volume(&mut self, value: rhai::FLOAT) { self.volume = value as f32; }
    pub fn set_repeat_start_time(&mut self, value: rhai::FLOAT) { self.repeat_start_time = value as f64; }
    pub fn set_repeat(&mut self, value: bool) { self.repeat = value; }
    pub fn set_paused(&mut self, value: bool) { self.paused = value; }

    //
    pub fn to_string(&mut self) -> String {
        //
        format!(
        "Audio {{\n\tid: {}, tag: \"{}\", audio_time: {},\n\tpaused: {}, repeat: {}, repeat_start_time: {}, volume: {} \n}}",
        self.id, self.tag, self.audio_time, self.paused, self.repeat, self.repeat_start_time, self.volume)
    }
}

//
#[allow(dead_code)]
#[derive(Clone)]
pub struct Font {
    pub id: u32,
    pub text: String,
}

#[allow(dead_code)]
impl Font {
    pub fn new(new_id: u32) -> Self {
        Self { id: new_id, text: String::new() }
    }
    pub fn recycle(&mut self, new_id: u32) {
        self.id = new_id; self.text.clear();
    }
    pub fn get_id(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_text(&mut self) -> String { self.text.clone() }
    pub fn set_text(&mut self, value: &str) { self.text.clear(); self.text.push_str(value); }

    //
    pub fn to_string(&mut self) -> String {
        //
        format!("Font {{\n\tid: {}, text: \"{}\"\n}}",self.id, self.text)
    }
}

//
#[derive(Clone)]
pub struct AssetList<T: Clone> {
    pub members: Vec<T>,
    pub cur_asset: usize,
    pub len: usize,
}

//
impl<T: Clone> AssetList<T> {
    //
    pub fn new(vec: Vec<T>) -> Self {
        //
        let length = vec.len();
        //
        Self { members: vec, cur_asset: 0,
        len: length }
    }
    //
    pub fn len(&mut self) -> rhai::INT {
        //
        self.len as rhai::INT
    }
    //
    pub fn get_cur_asset(&mut self) -> rhai::INT { self.cur_asset.clone() as rhai::INT }
    pub fn set_cur_asset(&mut self, value: rhai::INT) {
        //
        self.cur_asset = if value < 0 {
            //
            value.checked_abs().map_or(0, |n| self.len - (n as usize).min(self.len))
        } else {
            //
            (value as usize).min(self.len)
        };
    }
    //
    pub fn get_asset(&mut self, idx: rhai::INT) -> T {
        //
        let actual_index = if idx < 0 {
            //
            idx.checked_abs().map_or(0, |n| self.len - (n as usize).min(self.len))
        } else {
            //
            (idx as usize).min(self.len)
        };
        //
        self.members[actual_index].clone()
    }
    //
    pub fn set_asset(&mut self, idx: rhai::INT, asset: T) {
        //
        let actual_index = if idx < 0 {
            //
            idx.checked_abs().map_or(0, |n| self.len - (n as usize).min(self.len))
        } else {
            //
            (idx as usize).min(self.len)
        };
        //
        self.members[actual_index] = asset;
    }
}

impl AssetList<Sprite> {
    //
    pub fn contains(&mut self, id: rhai::INT) -> Dynamic {
        //
        if let Some(idx) = self.members.iter()
        .position(|asset| -> bool { asset.id == id as u32 }) {
            //
            if idx < self.len {
                //
                Dynamic::from(idx)
            } else {
                //
                Dynamic::UNIT
            }
        } else {
            //
            Dynamic::UNIT
        }
    }
    //
    pub fn recycle(&mut self, vec: Vec<i32>) {
        //
        self.cur_asset = 0;
        //
        self.len = vec.len();
        //
        let mut i = 0_usize;
        //
        for id in vec {
            if i < self.members.len() {
                //
                self.members[i].recycle(id as u32);
                //
                i += 1;
                continue;
            }
            //
            self.members.push(Sprite::new(id as u32));
            //
            i += 1;
        }
    }
}

#[allow(dead_code)]
impl AssetList<Audio> {
    //
    pub fn contains(&mut self, id: rhai::INT) -> Dynamic {
        //
        if let Some(idx) = self.members.iter()
        .position(|asset| -> bool { asset.id == id as u32 }) {
            //
            if idx < self.len {
                //
                Dynamic::from(idx)
            } else {
                //
                Dynamic::UNIT
            }
        } else {
            //
            Dynamic::UNIT
        }
    }
    //
    pub fn recycle(&mut self, vec: Vec<i32>) {
        //
        self.cur_asset = 0;
        //
        self.len = vec.len();
        //
        let mut i = 0_usize;
        //
        for id in vec {
            if i < self.members.len() {
                //
                self.members[i].recycle(id as u32);
                //
                i += 1;
                continue;
            }
            //
            self.members.push(Audio::new(id as u32));
            //
            i += 1;
        }
    }
}

#[allow(dead_code)]
impl AssetList<Font> {
    //
    pub fn contains(&mut self, id: rhai::INT) -> Dynamic {
        //
        if let Some(idx) = self.members.iter()
        .position(|asset| -> bool { asset.id == id as u32 }) {
            //
            if idx < self.len {
                //
                Dynamic::from(idx)
            } else {
                //
                Dynamic::UNIT
            }
        } else {
            //
            Dynamic::UNIT
        }
    }
    //
    pub fn recycle(&mut self, vec: Vec<i32>) {
        //
        self.cur_asset = 0;
        //
        self.len = vec.len();
        //
        let mut i = 0_usize;
        //
        for id in vec {
            if i < self.members.len() {
                //
                self.members[i].recycle(id as u32);
                //
                i += 1;
                continue;
            }
            //
            self.members.push(Font::new(id as u32));
            //
            i += 1;
        }
    }
}