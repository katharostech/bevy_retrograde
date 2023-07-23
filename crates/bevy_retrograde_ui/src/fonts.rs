//! Bitmap font asset loader

use std::{collections::BTreeMap, sync::Arc};

use crate::bdf;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use bevy_egui::{
    egui::{self, mutex::Mutex},
    EguiContexts,
};
use image::{GenericImage, Rgba, RgbaImage};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};

/// Retro font texture cache. Used internally, but may be useful for advanced users.
pub type RetroFontCache = Arc<Mutex<HashMap<Handle<RetroFont>, RetroFontCacheItem>>>;

/// Record in the retro font texture cache. Used internally, but may be useful for advanced users.
#[derive(Clone)]
pub struct RetroFontCacheItem {
    pub texture_id: egui::TextureId,
    pub font_data: Arc<RetroFontData>,
}

/// Loop through all [`RetroFont`] assets and map their texture ids and uvs to their handle
pub(crate) fn font_texture_update(fonts: Res<Assets<RetroFont>>, mut egui_ctx: EguiContexts) {
    for (handle_id, font) in fonts.iter() {
        let texture_id = egui_ctx.add_image(font.data.texture.clone_weak());
        let handle = Handle::weak(handle_id);

        let ctx = egui_ctx.ctx_mut();
        ctx.memory_mut(|ctx| {
            let mut retro_font_texture_datas = ctx
                .data
                .get_temp_mut_or_default::<RetroFontCache>(egui::Id::null())
                .lock();

            let texture_data =
                retro_font_texture_datas
                    .entry(handle)
                    .or_insert_with(|| RetroFontCacheItem {
                        texture_id,
                        font_data: font.data.clone(),
                    });
            if !Arc::ptr_eq(&texture_data.font_data, &font.data) {
                texture_data.font_data = font.data.clone();
            }
            texture_data.texture_id = texture_id;
        });
    }
}

/// A bitmap font asset that can be loaded from .bdf files
#[derive(TypeUuid, TypePath)]
#[uuid = "fd2ca871-a323-4811-bae9-aa3c18d0e266"]
pub struct RetroFont {
    pub data: Arc<RetroFontData>,
}

/// The data inside of a [`RetroFont`]
pub struct RetroFontData {
    pub texture: Handle<Image>,
    pub font: bdf::Font,
    pub glyph_uvs: HashMap<char, egui::Rect>,
}

/// [`RetroFont`] asset loader implementation
#[derive(Default)]
pub struct RetroFontLoader;

impl AssetLoader for RetroFontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            // Parse the font
            let font = bdf::parse(bytes)?;

            let mut glyph_uvs = HashMap::default();

            let texture_width = 1024;
            let texture_height = 1024;

            // Start packing glyphs into the texture image
            let mut rects_to_place = GroupedRectsToPlace::<char, ()>::new();
            for glyph in font.glyphs.values() {
                rects_to_place.push_rect(
                    glyph.codepoint,
                    None,
                    RectToInsert::new(glyph.bounds.width, glyph.bounds.height, 1),
                );
            }
            let mut target_bins = BTreeMap::new();
            target_bins.insert(0, TargetBin::new(1024, 1024, 1));
            let pack_info = pack_rects(
                &rects_to_place,
                &mut target_bins,
                &volume_heuristic,
                &contains_smallest_box,
            )
            .expect("Pack font texture");

            // Render the font texture with all the glyphs in it
            let mut image_buf = RgbaImage::new(texture_width, texture_height);

            for glyph in font.glyphs.values() {
                let bounds = &glyph.bounds;
                let (_, location) = pack_info.packed_locations().get(&glyph.codepoint).unwrap();

                if !glyph.codepoint.is_whitespace() {
                    glyph_uvs.insert(
                        glyph.codepoint,
                        egui::Rect::from_min_size(
                            egui::Pos2::new(
                                location.x() as f32 / texture_width as f32,
                                location.y() as f32 / texture_height as f32,
                            ),
                            egui::Vec2::new(
                                location.width() as f32 / texture_width as f32,
                                location.height() as f32 / texture_height as f32,
                            ),
                        ),
                    );

                    let mut sub_img = image_buf.sub_image(
                        location.x(),
                        location.y(),
                        location.width(),
                        location.height(),
                    );

                    for x in 0..bounds.width {
                        for y in 0..bounds.height {
                            let pixel = sub_img.get_pixel_mut(x, y);

                            *pixel =
                                Rgba([255, 255, 255, if glyph.bitmap.get(x, y) { 255 } else { 0 }]);
                        }
                    }
                }
            }

            let image = Image::new(
                Extent3d {
                    width: texture_width,
                    height: texture_height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                image_buf.into_raw(),
                TextureFormat::Rgba8Unorm,
            );

            let texture = load_context.set_labeled_asset(
                "texture",
                LoadedAsset::new(image).with_dependency(load_context.path().into()),
            );

            let retro_font = RetroFont {
                data: Arc::new(RetroFontData {
                    font,
                    texture,
                    glyph_uvs,
                }),
            };
            load_context.set_default_asset(LoadedAsset::new(retro_font));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["bdf"]
    }
}
