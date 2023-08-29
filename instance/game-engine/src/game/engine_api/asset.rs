
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
    pub fn get_id(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_id_float(&mut self) -> rhai::FLOAT { self.id.clone() as rhai::FLOAT }
    pub fn get_cur_animation(&mut self) -> String { self.cur_animation.clone() }
    pub fn get_cur_frame(&mut self) -> rhai::INT { self.cur_frame.clone() as rhai::INT }
    pub fn get_cur_frame_float(&mut self) -> rhai::FLOAT { self.cur_frame.clone() as rhai::FLOAT }
    pub fn get_is_animation_finished(&mut self) -> bool { self.is_animation_finished.clone() }
    pub fn get_animation_time(&mut self) -> rhai::FLOAT { self.animation_time.clone() as rhai::FLOAT }
    pub fn get_animation_time_int(&mut self) -> rhai::INT { self.animation_time.clone() as rhai::INT }
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
    pub fn play_or_repeat_animation(&mut self, name: &str, repeat: bool) {
        //
        self.cur_animation.clear();
        //
        self.cur_animation.push_str(name);
        //
        self.cur_frame = 0;
        //
        self.animation_time = 0.0;
        //
        self.repeat = repeat;
        //
        self.is_animation_finished = false;
    }

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

    pub fn set_cur_frame_float(&mut self, value: rhai::FLOAT) -> Result<(), Box<rhai::EvalAltResult>> {
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
}

//
#[derive(Clone)]
pub struct Audio {
    pub id: u32,
    pub tag: String,
    pub audio_time: f64,
    pub paused: bool,
    pub repeat: bool,
    pub repeat_start_time: f64,
    pub volume: f32,
}

impl Audio {
    pub fn get_id(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_id_float(&mut self) -> rhai::FLOAT { self.id.clone() as rhai::FLOAT }
    pub fn get_audio_time(&mut self) -> rhai::FLOAT { self.audio_time.clone() as rhai::FLOAT }
    pub fn get_audio_time_int(&mut self) -> rhai::INT { self.audio_time.clone() as rhai::INT }
    pub fn get_paused(&mut self) -> bool { self.paused.clone() }
    pub fn get_repeat(&mut self) -> bool { self.repeat.clone() }
    pub fn get_repeat_start_time(&mut self) -> rhai::FLOAT { self.repeat_start_time.clone() as rhai::FLOAT }
    pub fn get_repeat_start_time_int(&mut self) -> rhai::INT { self.repeat_start_time.clone() as rhai::INT }
    pub fn get_volume(&mut self) -> rhai::FLOAT { self.volume.clone() as rhai::FLOAT }
    pub fn get_volume_int(&mut self) -> rhai::INT { self.volume.clone() as rhai::INT }

    pub fn play(&mut self) {
        //
        self.tag.clear();
        //
        self.audio_time = 0.0;
        //
        self.repeat = false;
        //
        self.volume = 1.0;
        //
        self.paused = false;
    }

    pub fn play_with_tag(&mut self, tag: &str) {
        //
        self.tag.clear();
        //
        self.tag.push_str(tag);
        //
        self.audio_time = 0.0;
        //
        self.repeat = false;
        //
        self.volume = 1.0;
        //
        self.paused = false;
    }

    pub fn set_volume(&mut self, value: rhai::FLOAT) { self.volume = value as f32; }
    pub fn set_volume_int(&mut self, value: rhai::INT) { self.volume = value as f32; }
    pub fn set_repeat_start_time(&mut self, value: rhai::FLOAT) { self.repeat_start_time = value as f64; }
    pub fn set_repeat_start_time_int(&mut self, value: rhai::INT) { self.repeat_start_time = value as f64; }
    pub fn set_repeat(&mut self, value: bool) { self.repeat = value; }
    pub fn set_paused(&mut self, value: bool) { self.paused = value; }
}

//
#[derive(Clone)]
pub struct Font {
    pub id: u32,
    pub text: String,
}

impl Font {
    pub fn get_id(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_id_float(&mut self) -> rhai::FLOAT { self.id.clone() as rhai::FLOAT }
    pub fn get_text(&mut self) -> String { self.text.clone() }
    pub fn set_text(&mut self, value: &str) { self.text.clear(); self.text.push_str(value); }
}

//
#[derive(Clone)]
pub struct AssetList<T: Clone> {
    pub members: Vec<T>,
    pub locked_on: Vec<usize>,
    pub is_locked: bool,
}

//
impl<T: Clone> AssetList<T> {
    //
    pub fn len(&mut self) -> rhai::INT {
        //
        if self.is_locked {
            //
            self.locked_on.len() as rhai::INT
        } else {
            //
            self.members.len() as rhai::INT
        }
    }
    //
    pub fn lock_with_indcies(&mut self, indcies: rhai::Array)
     -> Result<(), Box<rhai::EvalAltResult>> {
        //
        if self.is_locked {
            //
            return Err(concat!("You can only lock an AssetList",
            " once, and only on the 'create' callback.")
            .into());
        }
        //
        let len = self.members.len();
        //
        let mut actual_index: usize;
        //
        for index in indcies {
            //
            if let Ok(idx) = index.as_int() {
                //
                actual_index = if idx < 0 {
                    //
                    idx.checked_abs().map_or(0, |n| len - (n as usize).min(len))
                } else {
                    //
                    (idx as usize).min(len)
                };
                //
                if !self.locked_on.contains(&actual_index) {
                    //
                    self.locked_on.push(actual_index)
                }
            } else {
                //
                return Err(concat!("When locking an AssetList on specific members,",
                " you should only provide INT values to specify the indcies.")
                .into());
            }
        }
        //
        self.is_locked = true;
        //
        Ok(())
    }
    pub fn lock(&mut self) -> Result<(), Box<rhai::EvalAltResult>> {
        //
        if self.is_locked {
            //
            Err(concat!("You can only lock an AssetList",
            " once, and only on the 'create' callback.")
            .into())
        } else {
            //
            self.is_locked = true;
            //
            Ok(())
        }
    }
    //
    pub fn get_asset(&mut self, idx: rhai::INT) -> Result<T, Box<rhai::EvalAltResult>> {
        //
        let len = self.members.len();
        //
        let actual_index = if idx < 0 {
            //
            idx.checked_abs().map_or(0, |n| len - (n as usize).min(len))
        } else {
            //
            (idx as usize).min(len)
        };
        //
        if self.is_locked && !self.locked_on.contains(&actual_index) {
            //
            Err(concat!("Tried to get an AssetList member, but the AssetList was locked.",
            " Note: You can provide an indcies array to the AssetList::lock method, to",
            " specify which members will still be available after the AssetList gets locked.")
            .into())
        } else {
            //
            Ok(self.members[actual_index].clone())
        }    
    }
    //
    pub fn set_asset(&mut self, idx: rhai::INT, asset: T) -> Result<(), Box<rhai::EvalAltResult>> {
        //
        let len = self.members.len();
        //
        let actual_index = if idx < 0 {
            //
            idx.checked_abs().map_or(0, |n| len - (n as usize).min(len))
        } else {
            //
            (idx as usize).min(len)
        };
        if self.is_locked && !self.locked_on.contains(&actual_index) {
            //
            Err(concat!("Tried to set an AssetList member, but the AssetList was locked.",
            " Note: You can provide an indcies array to the AssetList::lock method, to",
            " specify which members will still be available after the AssetList gets locked.")
            .into())
        } else {
            //
            self.members[actual_index] = asset;
            //
            Ok(())
        }
    }
}

impl AssetList<Sprite> {
    //
    pub fn contains(&mut self, id: rhai::INT) -> Dynamic {
        //
        if self.is_locked {
            //
            if let Some(&index) = self.locked_on.iter()
            .find(|&&idx| -> bool {
                //
                self.members[idx].id == id as u32
            }) {
                //
                Dynamic::from(index)
            } else {
                //
                Dynamic::UNIT
            }
        } else {
            //
            if let Some(idx) = self.members.iter()
            .position(|asset| -> bool {
                //
                asset.id == id as u32
            }) {
                //
                Dynamic::from(idx)
            } else {
                //
                Dynamic::UNIT
            }
        }
    }
}

impl AssetList<Audio> {
    //
    pub fn contains(&mut self, id: rhai::INT) -> Dynamic {
        //
        if self.is_locked {
            //
            if let Some(&index) = self.locked_on.iter()
            .find(|&&idx| -> bool {
                //
                self.members[idx].id == id as u32
            }) {
                //
                Dynamic::from(index)
            } else {
                //
                Dynamic::UNIT
            }
        } else {
            //
            if let Some(idx) = self.members.iter()
            .position(|asset| -> bool {
                //
                asset.id == id as u32
            }) {
                //
                Dynamic::from(idx)
            } else {
                //
                Dynamic::UNIT
            }
        }
    }
}

impl AssetList<Font> {
    //
    pub fn contains(&mut self, id: rhai::INT) -> Dynamic {
        //
        if self.is_locked {
            //
            if let Some(&index) = self.locked_on.iter()
            .find(|&&idx| -> bool {
                //
                self.members[idx].id == id as u32
            }) {
                //
                Dynamic::from(index)
            } else {
                //
                Dynamic::UNIT
            }
        } else {
            //
            if let Some(idx) = self.members.iter()
            .position(|asset| -> bool {
                //
                asset.id == id as u32
            }) {
                //
                Dynamic::from(idx)
            } else {
                //
                Dynamic::UNIT
            }
        }
    }
}