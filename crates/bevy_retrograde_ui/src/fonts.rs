use std::{collections::BTreeMap, sync::Arc};

use crate::bdf;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use bevy_egui::{
    egui::{self, mutex::Mutex, Widget},
    EguiContext,
};
use image::{GenericImage, Rgba, RgbaImage};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    TargetBin,
};

pub type RetroFontCache = Arc<Mutex<HashMap<Handle<RetroFont>, RetroFontCacheItem>>>;

#[derive(Clone)]
pub struct RetroFontCacheItem {
    pub texture_id: egui::TextureId,
    pub font_data: Arc<RetroFontData>,
}

/// Loop through all [`RetroFont`] assets and map their texture ids and uvs to their handle
pub(crate) fn font_texture_update(
    fonts: Res<Assets<RetroFont>>,
    mut egui_ctx: ResMut<EguiContext>,
) {
    for (handle_id, font) in fonts.iter() {
        let texture_id = egui_ctx.add_image(font.data.texture.clone());
        let handle = Handle::weak(handle_id);

        let ctx = egui_ctx.ctx_mut();
        let mut memory = ctx.memory();
        let mut retro_font_texture_datas = memory
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
    }
}

pub struct RetroLetter<'a> {
    pub codepoint: char,
    pub font: &'a Handle<RetroFont>,
}

impl<'a> Widget for RetroLetter<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let empty_response = ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());

        let (texture, uv, size) = {
            let ctx = ui.ctx();
            let mut memory = ctx.memory();
            let retro_font_texture_datas = memory
                .data
                .get_temp_mut_or_default::<RetroFontCache>(egui::Id::null())
                .lock();

            if let Some(data) = retro_font_texture_datas.get(&self.font) {
                let font = &data.font_data.font;
                let size = if let Some(glyph) = font.glyphs.get(&self.codepoint) {
                    egui::Vec2::new(glyph.bounds.width as f32, font.bounds.height as f32)
                } else {
                    return empty_response;
                };
                let uv = if let Some(uv) = data.font_data.glyph_uvs.get(&self.codepoint) {
                    uv
                } else {
                    return empty_response;
                };
                (data.texture_id, *uv, size)
            } else {
                return empty_response;
            }
        };

        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

        let mut mesh = egui::Mesh::default();
        mesh.texture_id = texture;
        mesh.add_rect_with_uv(rect, uv, egui::Color32::RED);

        ui.painter().add(mesh);

        response
    }
}

/// A bitmap font asset that can be loaded from .bdf files
#[derive(TypeUuid)]
#[uuid = "fd2ca871-a323-4811-bae9-aa3c18d0e266"]
pub struct RetroFont {
    pub data: Arc<RetroFontData>,
}

pub struct RetroFontData {
    pub texture: Handle<Image>,
    pub font: bdf::Font,
    pub glyph_uvs: HashMap<char, egui::Rect>,
}

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
