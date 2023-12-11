
use rhai::Dynamic;

use crate::game::dynamic_to_number;

/// An `Asset` Trait, which
/// will help us implement\
/// the `AssetList` type,
/// by providing a common\
/// interface for all the
/// different types of
/// asset propertys.
pub trait Asset {
    /// Get the asset's rowid.
    fn get_id(&self) -> u32;
    /// Recycle the asset propertys.
    fn recycle(&mut self, new_id: u32);
    /// Create new asset propertys
    fn new(new_id: u32) -> Self;
}

/// A `Sprite` struct,
/// which will be used to\
/// store a sprite's propertys
/// for a specific\
/// `Object`.
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
    pub fn get_id_rhai(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_cur_animation(&mut self) -> String { self.cur_animation.clone() }
    pub fn get_cur_frame(&mut self) -> rhai::INT { self.cur_frame.clone() as rhai::INT }
    pub fn get_is_animation_finished(&mut self) -> bool { self.is_animation_finished.clone() }
    pub fn get_animation_time(&mut self) -> rhai::FLOAT { self.animation_time.clone() as rhai::FLOAT }
    pub fn get_repeat(&mut self) -> bool { self.repeat.clone() }

    /// Given an animation name,
    /// this method will setup the\
    /// sprite's propertys to play
    /// that animation from the\
    /// next rendering frame.
    pub fn play_animation(&mut self, name: &str) {
        self.cur_animation.clear();
        self.cur_animation.push_str(name);
        self.cur_frame = 0;
        self.animation_time = 0.0;
        self.repeat = false;
        self.is_animation_finished = false;
    }

    /// Given an animation
    /// name and a time in 
    /// seconds, this method 
    /// will setup the sprite's\
    /// propertys to play that
    /// animation form the specified 
    /// time at the next rendering frame.
    pub fn play_animation_on_time(&mut self, name: &str, time: rhai::FLOAT) {
        self.cur_animation.clear();
        self.cur_animation.push_str(name);
        self.cur_frame = 0;
        self.animation_time = time as f64;
        self.repeat = false;
        self.is_animation_finished = false;
    }

    pub fn set_repeat(&mut self, value: bool) { self.repeat = value; }
    pub fn set_is_animation_finished(&mut self, value: bool) { self.is_animation_finished = value; }

    /// This setter will prevent
    /// you from setting the current
    /// animation while an animation
    /// is playing.
    pub fn set_cur_animation(&mut self, value: &str) -> Result<(), Box<rhai::EvalAltResult>> {
        if !self.is_animation_finished {
            return Err(concat!("Tried to set the current animation while an animation was playing.",
            " Note: To start playing another animation while an animation is playing, use the",
            " 'play_animation' method, and if you want it to loop, provide the optional 'repeat'",
            " boolean parameter.").into());
        }
        self.cur_animation.clear();
        self.cur_animation.push_str(value);
        Ok(())
    }

    /// This setter will prevent you
    /// from setting the current frame
    /// while an animation is playing.
    pub fn set_cur_frame(&mut self, value: rhai::INT) -> Result<(), Box<rhai::EvalAltResult>> {
        if !self.is_animation_finished {
            return Err(concat!("Tried to set the current frame while an animation was playing.",
            " Note: To start playing another animation while an animation is playing, use the",
            " 'play_animation' method, and if you want it to loop, provide the optional 'repeat'",
            " boolean parameter.").into());
        }
        self.cur_frame = value as u32;
        Ok(())
    }
}

// Implemenation of the `Asset`
// trait for the `Sprite` struct.
impl Asset for Sprite {
    fn get_id(&self) -> u32 { self.id }
    fn recycle(&mut self, new_id: u32) {
        self.id = new_id; self.cur_animation.clear();
        self.cur_frame = 0_u32; self.is_animation_finished = true;
        self.animation_time = 0_f64; self.repeat = false;
    }
    fn new(new_id: u32) -> Self { Self {
        id: new_id, cur_animation: String::new(),
        cur_frame: 0_u32, is_animation_finished: true,
        animation_time: 0_f64, repeat: false
    } }
}

/// An `Audio` struct,
/// which will be used to\
/// store a audio's propertys
/// for a specific\
/// `Object`.
/// 
/// ## Important Note
/// 
/// The audio system itself was not\
/// implemented as part of this project,\
/// therefore this API remains unused.
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
    pub fn get_id_rhai(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_tag(&mut self) -> String { self.tag.clone() }
    pub fn get_audio_time(&mut self) -> rhai::FLOAT { self.audio_time.clone() as rhai::FLOAT }
    pub fn get_paused(&mut self) -> bool { self.paused.clone() }
    pub fn get_repeat(&mut self) -> bool { self.repeat.clone() }
    pub fn get_repeat_start_time(&mut self) -> rhai::FLOAT { self.repeat_start_time.clone() as rhai::FLOAT }
    pub fn get_volume(&mut self) -> rhai::FLOAT { self.volume.clone() as rhai::FLOAT }

    pub fn play(&mut self) {
        self.audio_time = 0.0;
        self.paused = false;
        self.replay = true;
    }

    pub fn obtain_tag(&mut self, tag: &str) {
        self.tag.clear();
        self.tag.push_str(tag);
        self.own_tag = false;
    }

    pub fn set_volume(&mut self, value: rhai::FLOAT) { self.volume = value as f32; }
    pub fn set_repeat_start_time(&mut self, value: rhai::FLOAT) { self.repeat_start_time = value as f64; }
    pub fn set_repeat(&mut self, value: bool) { self.repeat = value; }
    pub fn set_paused(&mut self, value: bool) { self.paused = value; }
}

// Implemenation of the `Asset`
// trait for the `Audio` struct.
impl Asset for Audio {
    fn get_id(&self) -> u32 { self.id }
    fn recycle(&mut self, new_id: u32) {
        self.id = new_id; self.tag.clear(); 
        self.audio_time = 0_f64; self.paused = false;
        self.repeat = false; self.volume = 1_f32; self.replay = false;
        self.repeat_start_time = 0_f64; self.own_tag = false;
    }
    fn new(new_id: u32) -> Self { Self {
        id: new_id, tag: String::new(),
        audio_time: 0_f64, paused: false,
        repeat: false, volume: 1_f32, replay: false,
        repeat_start_time: 0_f64, own_tag: false,
    } }
}

/// A `Font` struct,
/// which will be used to\
/// store a font's propertys
/// for a specific\
/// `Object`.
/// 
/// ## Important Note
/// 
/// The font system itself was not\
/// implemented as part of this project,\
/// therefore this API remains unused.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Font {
    pub id: u32,
    pub text: String,
}

#[allow(dead_code)]
impl Font {
    pub fn get_id_rhai(&mut self) -> rhai::INT { self.id.clone() as rhai::INT }
    pub fn get_text(&mut self) -> String { self.text.clone() }
    pub fn set_text(&mut self, value: &str) { self.text.clear(); self.text.push_str(value); }
}

// Implemenation of the `Asset`
// trait for the `Font` struct.
impl Asset for Font {
    fn get_id(&self) -> u32 { self.id }
    fn recycle(&mut self, new_id: u32) {
        self.id = new_id; self.text.clear();
    }
    fn new(new_id: u32) -> Self {
        Self { id: new_id, text: String::new() }
    }
}

/// A struct which represents an established\
/// list of assets, which will be used by a\
/// certain `Object`. The Object will be able\
/// to use one asset form the list at a time,\
/// and will be able to switch between them\
/// using the `cur_asset` attribute.
#[derive(Clone)]
pub struct AssetList<T: Clone + Asset> {
    pub members: Vec<T>,
    pub cur_asset: usize,
    pub len: usize,
}

impl<T: Clone + Asset> AssetList<T> {
    /// Create a new `AssetList` instance
    /// from a vector of assets.
    pub fn new(vec: Vec<T>) -> Self {
        // Get the vector's length.
        let length = vec.len();
        // Create the new instance.
        Self { members: vec, cur_asset: 0,
        len: length }
    }

    pub fn len(&mut self) -> rhai::INT { self.len as rhai::INT }
    pub fn get_cur_asset(&mut self) -> rhai::INT { self.cur_asset.clone() as rhai::INT }

    /// This setter will prevent you
    /// from setting the current asset\
    /// with an index that is out of
    /// bounds. It won't raise an error,\
    /// but it will clamp the index to the
    /// length of the list, and sum\
    /// negative indices with the length
    /// of the list, preventing it\
    /// from being out of bounds.
    pub fn set_cur_asset(&mut self, value: rhai::INT) {
        self.cur_asset = if value < 0 {
            value.checked_abs().map_or(0, |n| self.len - (n as usize).min(self.len))
        } else {
            (value as usize).min(self.len)
        };
    }

    /// This getter will prevent you
    /// from getting values from asset\
    /// indices which are out of bounds.
    /// It won't raise an error,\
    /// but it will clamp the index to the
    /// length of the list, and sum\
    /// negative indices with the length
    /// of the list, preventing it\
    /// from being out of bounds.
    pub fn get_asset(&mut self, idx: rhai::INT) -> T {
        let actual_index = if idx < 0 {
            idx.checked_abs().map_or(0, |n| self.len - (n as usize).min(self.len))
        } else {
            (idx as usize).min(self.len)
        };
        self.members[actual_index].clone()
    }
    
    /// This setter will prevent you
    /// from setting values in asset\
    /// indices which are out of bounds.
    /// It won't raise an error,\
    /// but it will clamp the index to the
    /// length of the list, and sum\
    /// negative indices with the length
    /// of the list, preventing it\
    /// from being out of bounds.
    /// 
    /// Additionally, it will prevent
    /// you from setting an asset with\
    /// a different id than the one
    /// that is already in the list, but\
    /// instead of raising an error, it
    /// will just ignore the new asset.
    pub fn set_asset(&mut self, idx: rhai::INT, asset: T) {
        let actual_index = if idx < 0 {
            idx.checked_abs().map_or(0, |n| self.len - (n as usize).min(self.len))
        } else {
            (idx as usize).min(self.len)
        };
        if self.members[actual_index].get_id() == asset.get_id() {
            self.members[actual_index] = asset;
        }
    }

    /// Finds an asset in the list
    /// by its rowid, and returns its
    /// index. If the asset is not
    /// found, it returns `()`.
    pub fn find(&mut self, id: rhai::INT) -> Dynamic {
        if let Some(idx) = self.members.iter()
        .position(|asset| -> bool { asset.get_id() == id as u32 }) {
            if idx < self.len {
                Dynamic::from(idx)
            } else {
                Dynamic::UNIT
            }
        } else {
            Dynamic::UNIT
        }
    }

    /// Recyclea an existing `AssetList`
    /// instance using a vector of rowids.
    pub fn recycle(&mut self, vec: &Vec<Dynamic>) {
        self.cur_asset = 0;
        self.len = vec.len();

        for (index, id) in vec.into_iter().enumerate() {
            let id = dynamic_to_number(id)
            .expect(concat!("Every object's config should",
            " contain a 'sprites' array, which should only have",
            " integer members.")) as u32;

            if index < self.members.len() {
                self.members[index].recycle(id);
                continue;
            } 
            self.members.push(T::new(id));
        }
    }
}