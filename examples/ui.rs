use bevy::{
    prelude::*,
    render::camera::{
        DepthCalculation, OrthographicCameraBundle, OrthographicProjection, ScalingMode,
    },
};
use bevy_retrograde::prelude::*;

struct UiTheme {
    panel_bg: BorderImage,
    button_up_bg: BorderImage,
    button_down_bg: BorderImage,
    font: Handle<RetroFont>,
}

impl FromWorld for UiTheme {
    fn from_world(world: &mut World) -> Self {
        Self {
            panel_bg: BorderImage::load_from_world(
                world,
                "ui/panel.png",
                UVec2::new(48, 48),
                Rect::all(8.0),
            ),
            button_up_bg: BorderImage::load_from_world(
                world,
                "ui/button-up.png",
                UVec2::new(32, 16),
                Rect::all(8.0),
            ),
            button_down_bg: BorderImage::load_from_world(
                world,
                "ui/button-down.png",
                UVec2::new(32, 16),
                Rect::all(8.0),
            ),
            font: world
                .get_resource::<AssetServer>()
                .unwrap()
                .load("cozette.bdf"),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retrograde LDtk Map".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins::default())
        .insert_resource(LevelSelection::Index(0))
        .init_resource::<UiTheme>()
        .add_startup_system(setup)
        .add_system(update_ui_scale)
        .add_system(ui)
        .run();
}

const CAMERA_HEIGHT: f32 = 200.0;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Enable hot reload
    asset_server.watch_for_changes().unwrap();

    // Spawn the camera
    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            scale: CAMERA_HEIGHT / 2.0,
            scaling_mode: ScalingMode::FixedVertical,
            depth_calculation: DepthCalculation::ZDifference,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    });

    // Spawn the map
    let map = asset_server.load("maps/map.ldtk");
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: map,
        // We offset the map a little to move it more to the center of the screen, because maps are
        // spawned with (0, 0) as the top-left corner of the map
        transform: Transform::from_xyz(-180., -100., 0.),
        ..Default::default()
    });
}

/// This system makes sure that the UI scale of Egui matches our game scale so that a pixel in egui
/// will be the same size as a pixel in our sprites.
fn update_ui_scale(mut egui_settings: ResMut<EguiSettings>, windows: Res<Windows>) {
    if let Some(window) = windows.get_primary() {
        let window_height = window.height();
        let scale = window_height / CAMERA_HEIGHT;
        egui_settings.scale_factor = scale as f64;
    }
}

fn ui(
    mut map: Query<&mut Transform, With<Handle<LdtkAsset>>>,
    mut ctx: ResMut<EguiContext>,
    ui_theme: Res<UiTheme>,
) {
    let mut map_transform: Mut<Transform> = if let Ok(map) = map.get_single_mut() {
        map
    } else {
        return;
    };

    let ctx = ctx.ctx_mut();

    // Create an egui central panel this will cover the entire game screen
    egui::CentralPanel::default()
        // Because it covers the whole screen, make sure that it doesn't overlay the egui background frame
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            // Get the screen rect
            let screen_rect = ui.max_rect();
            // Calculate a margin of 15% of the screen size
            let outer_margin = screen_rect.size() * 0.15;
            let outer_margin = Rect {
                left: outer_margin.x,
                right: outer_margin.x,
                // Make top and bottom margins smaller
                top: outer_margin.y / 2.0,
                bottom: outer_margin.y / 2.0,
            };

            // Render a bordered frame
            BorderedFrame::new(&ui_theme.panel_bg)
                .margin(outer_margin)
                .padding(Rect::all(8.0))
                .show(ui, |ui| {
                    // Make sure the frame ocupies the entire rect that we allocated for it.
                    //
                    // Without this it would only take up enough size to fit it's content.
                    ui.set_min_size(ui.available_size());

                    // Create a vertical list of items, centered horizontally
                    ui.vertical_centered(|ui| {
                        ui.retro_label("Bevy Retro + Egui = ♥", &ui_theme.font);

                        ui.add_space(10.0);
                        RetroLabel::new("Click a button to scale the background", &ui_theme.font)
                            .color(egui::Color32::GREEN)
                            .show(ui);

                        // Now switch the layout to bottom_up so that we can start adding widgets
                        // from the bottom of the frame.
                        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                            ui.add_space(4.0);

                            if RetroButton::new("Scale Down", &ui_theme.font)
                                .padding(Rect::all(7.0))
                                .border(&ui_theme.button_up_bg)
                                .on_click_border(&ui_theme.button_down_bg)
                                .show(ui)
                                .clicked()
                            {
                                map_transform.scale -= Vec3::splat(0.2);
                            }

                            if RetroButton::new("Scale Up", &ui_theme.font)
                                .padding(Rect::all(7.0))
                                .border(&ui_theme.button_up_bg)
                                .on_click_border(&ui_theme.button_down_bg)
                                .show(ui)
                                .clicked()
                            {
                                map_transform.scale += Vec3::splat(0.2);
                            }
                        });
                    });
                })
        });
}
