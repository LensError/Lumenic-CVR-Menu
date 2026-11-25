use std::collections::{HashMap, VecDeque};

use lumenic::{App, TextureId};

pub const MAX_SURFACE_TEXTURES: usize = 512;

#[derive(Default)]
pub struct SurfaceTextureCache {
    pub map: HashMap<String, TextureId>,
    order: VecDeque<String>,
}

impl SurfaceTextureCache {
    pub fn get(&self, key: &str) -> Option<TextureId> {
        self.map.get(key).copied()
    }

    pub fn insert(&mut self, app: &mut App, key: String, tex: TextureId) {
        if let Some(old) = self.map.insert(key.clone(), tex) {
            app.destroy_texture(old);
            self.order.retain(|k| k != &key);
        }
        // Keep local PFP, UILib mod icons, and extension assets pinned so browsing content thumbnails never evicts them.
        if key != "local_pfp" && !key.starts_with("uilib:") && !key.starts_with("ext:") {
            self.order.push_back(key);
        }

        while self.order.len() > MAX_SURFACE_TEXTURES {
            if let Some(oldest) = self.order.pop_front() {
                if let Some(old_tex) = self.map.remove(&oldest) {
                    app.destroy_texture(old_tex);
                }
            }
        }
    }
}

