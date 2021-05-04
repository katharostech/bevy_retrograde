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
        .run();
}

fn setup(mut commands: Commands, mut ui_tree: ResMut<UiTree>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(CameraBundle {
        camera: Camera {
            size: CameraSize::FixedHeight(200),
            background_color: Color::new(0.09, 0.1, 0.22, 1.),
            ..Default::default()
        },
        ..Default::default()
    });

    // Spawn an LDtk map just to give a decent backdrop for our UI
    commands.spawn().insert_bundle(LdtkMapBundle {
        map: asset_server.load("maps/map.ldtk"),
        position: Position::new(-200, -100, 0),
        ..Default::default()
    });

    *ui_tree = UiTree(widget! {
        (ui::my_widget)
    });
}

mod ui {
    use bevy_retro::ui::raui::prelude::*;

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
                left: 15.,
                right: 15.,
                top: 15.,
                bottom: 15.,
            },
            ..Default::default()
        });

        widget! {
            (content_box | {shared_props} [
                (popup: {popup_props})
            ])
        }
    }

    fn popup(ctx: WidgetContext) -> WidgetNode {
        let panel_props = ctx.props.clone().with(PaperProps {
            frame: None,
            ..Default::default()
        });
        let text_props = Props::new(TextBoxProps {
            text: "Manage Inventory".into(),
            font: TextBoxFont {
                name: "cozette.bdf".into(),
                size: 1.,
            },
            width: TextBoxSizeValue::Fill,
            alignment: TextBoxAlignment::Center,
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
                top: 40.,
                ..Default::default()
            },
            ..Default::default()
        });

        // let button_props = Props::new(FlexBoxItemLayout {
        //     margin: Rect {
        //         top: 50.,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // });

        widget! {
            (vertical_paper: {panel_props} [
                (text_box: {text_props})
                (image_box: {image_props})
                // (start_button: {button_props})
            ])
        }
    }

    #[pre_hooks(
        // This allows us to get a `ButtonProps` instance from our widget state which will keep
        // track of whether or not we are clicked, hovered over, etc.
        use_button_notified_state
    )]
    fn start_button(mut ctx: WidgetContext) -> WidgetNode {
        let ButtonProps {
            context,
            selected,
            trigger,
            ..
        } = ctx.state.read_cloned_or_default();

        let button_props = Props::new(NavItemActive)
            .with(ButtonNotifyProps(ctx.id.to_owned().into()))
            .with(FlexBoxItemLayout {
                basis: Some(100.),
                ..Default::default()
            });

        let button_panel_props = ctx.props.clone().with(PaperProps {
            frame: None,
            variant: String::from("button-up"),
            ..Default::default()
        });

        let label_props = Props::new(TextPaperProps {
            text: if context || selected {
                "Hovered!".into()
            } else if trigger {
                "Clicked!".into()
            } else {
                "Button Test".into()
            },
            use_main_color: true,
            width: TextBoxSizeValue::Exact(80.),
            ..Default::default()
        })
        .with(FlexBoxItemLayout {
            grow: 1.0,
            shrink: 1.0,
            fill: 1.0,
            align: 0.5,
            ..Default::default()
        });

        let size_box_props = Props::new(SizeBoxProps {
            height: SizeBoxSizeValue::Exact(30.),
            width: SizeBoxSizeValue::Content,
            ..Default::default()
        })
        .with(FlexBoxItemLayout {
            grow: 1.0,
            shrink: 1.0,
            fill: 1.0,
            align: 0.5,
            ..Default::default()
        });

        widget! {
            (#{ctx.key} button: {button_props} {
                content = (vertical_box [
                    (size_box: {size_box_props} {
                        content = (paper: {button_panel_props} [
                            (vertical_box [
                                (text_paper: {label_props})
                            ])
                        ])
                    })
                ])
            })
        }
    }
}
