use bevy::prelude::*;
use bevy_retro::prelude::*;
use bevy_retro::ui::raui::prelude::widget;

// Create a stage label that will be used for our game logic stage
#[derive(StageLabel, Debug, Eq, Hash, PartialEq, Clone)]
struct GameStage;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy Retro UI".into(),
            ..Default::default()
        })
        .add_plugins(RetroPlugins)
        .add_startup_system(setup.system())
        .add_event::<ButtonClicked>()
        .add_system(scroll_background.system())
        .run();
}

/// Event sent when our UI button is clicked
struct ButtonClicked;

/// Marker component for our map background
struct Map;

fn setup(mut commands: Commands, mut ui_tree: ResMut<UiTree>, asset_server: Res<AssetServer>) {
    // Spawn the camera
    commands.spawn_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(200),
            background_color: Color::new(0.09, 0.1, 0.22, 1.),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn an LDtk map, just to give a decent backdrop for our UI
    commands
        .spawn()
        .insert_bundle(LdtkMapBundle {
            map: asset_server.load("maps/map.ldtk"),
            position: Position::new(-200, -100, 0),
            ..Default::default()
        })
        .insert(Map);

    // Set the UI tree. The `UiTree` Resource is used to set the widget tree that should be
    // rendered. There can be only one widget tree rendered at a time, but the tree may be as simple
    // or as complex as you desire.
    *ui_tree = UiTree(widget! {
        (ui::my_widget)
    });
}

/// System that scrolls the background when the button is clicked
fn scroll_background(
    mut button_clicked_events: EventReader<ButtonClicked>,
    mut maps: Query<&mut Position, With<Map>>,
) {
    for _ in button_clicked_events.iter() {
        for mut pos in maps.iter_mut() {
            pos.x += 1;
        }
    }
}

// It's recommended to put your UI widgets in a separate module so that the imports of the RAUI
// types such as `Vec2` don't get mixed up with the Bevy types.
//
// Also, be sure to checkout the RAUI website to learn more about how to make UI's with RAUI:
// https://raui-labs.github.io/raui/
mod ui {
    use bevy::{app::Events, prelude::World};
    use bevy_retro::ui::raui::prelude::*;

    use crate::ButtonClicked;

    pub fn my_widget(_ctx: WidgetContext) -> WidgetNode {
        // Create shared properties that will be accessible to all child widgets, used for the theme
        // in our case.
        let shared_props = Props::default()
            // Add the theme properties
            .with({
                let mut theme = ThemeProps::default();

                theme.content_backgrounds.insert(
                    String::new(),
                    ThemedImageMaterial::Image(ImageBoxImage {
                        id: "ui/panel.png".to_owned(),
                        scaling: ImageBoxImageScaling::Frame((20.0, false).into()),
                        ..Default::default()
                    }),
                );

                theme.content_backgrounds.insert(
                    String::from("button-up"),
                    ThemedImageMaterial::Image(ImageBoxImage {
                        id: "ui/button-up.png".to_owned(),
                        scaling: ImageBoxImageScaling::Frame((8.0, false).into()),
                        ..Default::default()
                    }),
                );

                theme.content_backgrounds.insert(
                    String::from("button-down"),
                    ThemedImageMaterial::Image(ImageBoxImage {
                        id: "ui/button-down.png".to_owned(),
                        scaling: ImageBoxImageScaling::Frame((8.0, false).into()),
                        ..Default::default()
                    }),
                );

                theme.text_variants.insert(
                    String::new(),
                    ThemedTextMaterial {
                        font: TextBoxFont {
                            name: "cozette.bdf".into(),
                            // Font's in Bevy Retro don't really have sizes so we can just set this to
                            // one
                            size: 1.0,
                        },
                        ..Default::default()
                    },
                );

                theme
            });

        // Create the props for our popup
        let popup_props = Props::new(ContentBoxItemLayout {
            margin: Rect {
                left: 20.,
                right: 20.,
                top: 20.,
                bottom: 20.,
            },
            ..Default::default()
        });

        widget! {
            (nav_content_box | {shared_props} [
                (popup: {popup_props})
            ])
        }
    }

    // A simple popup-type component
    fn popup(ctx: WidgetContext) -> WidgetNode {
        let panel_props = ctx.props.clone().with(PaperProps {
            frame: None,
            ..Default::default()
        });

        let text_props = Props::new(TextBoxProps {
            text: "The Red Radish".into(),
            font: TextBoxFont {
                name: "cozette.bdf".into(),
                size: 1.,
            },
            width: TextBoxSizeValue::Fill,
            horizontal_align: TextBoxHorizontalAlign::Center,
            ..Default::default()
        })
        .with(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            fill: 1.0,
            align: 0.5,
            ..Default::default()
        });

        let image_props = Props::new(ImageBoxProps {
            material: ImageBoxMaterial::Image(ImageBoxImage {
                id: "redRadish.png".into(),
                ..Default::default()
            }),
            width: ImageBoxSizeValue::Exact(32.),
            height: ImageBoxSizeValue::Exact(32.),
            ..Default::default()
        })
        .with(FlexBoxItemLayout {
            grow: 0.0,
            shrink: 0.0,
            fill: 1.0,
            align: 0.5,
            margin: Rect {
                top: 30.,
                ..Default::default()
            },
            ..Default::default()
        });

        let button_props = Props::new(FlexBoxItemLayout {
            align: 0.5,
            margin: Rect {
                top: 15.,
                ..Default::default()
            },
            ..Default::default()
        });

        widget! {
            (nav_vertical_paper: {panel_props} [
                (text_box: {text_props})
                (image_box: {image_props})
                (start_button: {button_props})
            ])
        }
    }

    #[pre_hooks(
        // This allows us to get a `ButtonProps` instance from our widget state which will keep
        // track of whether or not we are clicked, hovered over, etc.
        use_button_notified_state
    )]
    fn start_button(mut ctx: WidgetContext) -> WidgetNode {
        // Get our button state
        let ButtonProps {
            selected: hover,
            trigger: clicked,
            ..
        } = ctx.state.read_cloned_or_default();

        // We can access the Bevy ECS world through the process context
        let world = ctx.process_context.get_mut::<World>().unwrap();

        // We use a scope to contain the mutable access and allow us to borrow the world after we
        // are done with the button events, if we needed to.
        {
            // And we can use that to get access to world resources and send events
            let mut button_events = world.get_resource_mut::<Events<ButtonClicked>>().unwrap();

            // Lets send a button clicked event if we are clicked
            if clicked {
                button_events.send(ButtonClicked);
            }
        }

        // In the rest of this we style our button

        let button_props = ctx
            .props
            .clone()
            .with(NavItemActive)
            .with(ButtonNotifyProps(ctx.id.to_owned().into()));

        let button_panel_props = ctx.props.clone().with(PaperProps {
            frame: None,
            variant: if clicked {
                String::from("button-down")
            } else {
                String::from("button-up")
            },
            ..Default::default()
        });

        let label_props = Props::new(TextPaperProps {
            text: "Start Game".to_owned(),
            width: TextBoxSizeValue::Fill,
            height: TextBoxSizeValue::Fill,
            horizontal_align_override: Some(TextBoxHorizontalAlign::Center),
            vertical_align_override: Some(TextBoxVerticalAlign::Middle),
            transform: Transform {
                translation: Vec2 {
                    x: 0.,
                    y: if clicked { 2. } else { 0. },
                },
                ..Default::default()
            },
            ..Default::default()
        });

        let scale = if hover { 1.05 } else { 1. };

        let size_box_props = Props::new(SizeBoxProps {
            width: SizeBoxSizeValue::Exact(85. * scale),
            height: SizeBoxSizeValue::Exact(25. * scale),
            ..Default::default()
        });

        widget! {
            (button: {button_props} {
                content = (size_box: {size_box_props} {
                    content = (horizontal_paper: {button_panel_props} [
                        (text_paper: {label_props})
                    ])
                })
            })
        }
    }
}
