use std::collections::{HashMap, VecDeque};

use lumenic::{App, TextureId};

pub const MAX_SURFACE_TEXTURES: usize = 512;
pub const MAX_EXT_TEXTURES: usize = 192;

#[derive(Default)]
pub struct SurfaceTextureCache {
    pub map: HashMap<String, TextureId>,
    order: VecDeque<String>,
    ext_order: VecDeque<String>,
}

impl SurfaceTextureCache {
    pub fn get(&self, key: &str) -> Option<TextureId> {
        self.map.get(key).copied()
    }

    pub fn insert(&mut self, app: &mut App, key: String, tex: TextureId) {
        if let Some(old) = self.map.insert(key.clone(), tex) {
            app.destroy_texture(old);
            self.order.retain(|k| k != &key);
            self.ext_order.retain(|k| k != &key);
        }
        // Keep local PFP and UILib mod icons pinned; bound extension textures in their own FIFO queue.
        if key.starts_with("ext:") {
            self.ext_order.push_back(key);
        } else if key != "local_pfp" && !key.starts_with("uilib:") {
            self.order.push_back(key);
        }

        while self.order.len() > MAX_SURFACE_TEXTURES {
            if let Some(oldest) = self.order.pop_front() {
                if let Some(old_tex) = self.map.remove(&oldest) {
                    app.destroy_texture(old_tex);
                }
            }
        }
        while self.ext_order.len() > MAX_EXT_TEXTURES {
            if let Some(oldest) = self.ext_order.pop_front() {
                if let Some(old_tex) = self.map.remove(&oldest) {
                    app.destroy_texture(old_tex);
                }
            }
        }
    }
}

