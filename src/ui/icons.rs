//! Icons embedded in the executable (`assets/icons/*.png`), uploaded once as textures.
//!
//! Icons are white on transparent, so they can be tinted (normal / hovered / active).

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Ui};

use crate::tools::ToolKind;

/// Icons are downscaled once at load time (sharper than letting the GPU shrink 512 px images).
const ICON_PIXELS: u32 = 64;

pub struct Icons {
    select: TextureHandle,
    move_tool: TextureHandle,
    scale: TextureHandle,
    block: TextureHandle,
}

impl Icons {
    pub fn load(ctx: &egui::Context) -> Self {
        let load = |name: &str, bytes: &[u8]| {
            ctx.load_texture(format!("icon_{name}"), decode(bytes), TextureOptions::LINEAR)
        };
        Self {
            select: load("select", include_bytes!("../../assets/icons/select.png")),
            move_tool: load("move", include_bytes!("../../assets/icons/move.png")),
            scale: load("scale", include_bytes!("../../assets/icons/scale.png")),
            block: load("block", include_bytes!("../../assets/icons/block.png")),
        }
    }

    pub fn tool(&self, tool: ToolKind) -> &TextureHandle {
        match tool {
            ToolKind::Select => &self.select,
            ToolKind::Move => &self.move_tool,
            ToolKind::Scale => &self.scale,
            ToolKind::Block => &self.block,
        }
    }
}

/// Draws a tinted icon inside `rect`.
pub fn paint(ui: &Ui, texture: &TextureHandle, rect: Rect, tint: Color32) {
    egui::Image::from_texture(egui::load::SizedTexture::new(texture.id(), rect.size()))
        .tint(tint)
        .paint_at(ui, rect);
}

fn decode(bytes: &[u8]) -> ColorImage {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .expect("embedded icon is a valid PNG")
        .resize(ICON_PIXELS, ICON_PIXELS, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    ColorImage::from_rgba_unmultiplied(size, image.as_raw())
}
