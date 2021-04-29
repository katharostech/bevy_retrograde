use bevy::prelude::*;
use bevy_retro::prelude::*;
use bevy_retro::ui::raui::prelude::*;
use ui::raui::prelude::Color;

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

fn setup(mut commands: Commands, mut ui_tree: ResMut<UiTree>) {
    commands.spawn_bundle(CameraBundle::default());

    *ui_tree = UiTree(widget! {
        (my_widget)
    });
}

pub fn my_widget(_ctx: WidgetContext) -> WidgetNode {
    // We may do any amount of processing in the body of the function.

    // For now we will simply be creating a text box properties struct that we
    // will use to configure the `text_box` component.
    let text_box_props = TextBoxProps {
        text: "Hello world!".to_owned(),
        color: raui::core::widget::utils::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        },
        font: TextBoxFont {
            name: "cozette.bdf".to_owned(),
            size: 16.0,
        },
        ..Default::default()
    };

    let image_box1_props = ImageBoxProps {
        material: ImageBoxMaterial::Color(ImageBoxColor {
            color: Color {
                r: 0.2,
                g: 0.2,
                b: 1.,
                a: 1.,
            },
            ..Default::default()
        }),
        width: ImageBoxSizeValue::Fill,
        ..Default::default()
    };

    let image_box2_props = ImageBoxProps {
        material: ImageBoxMaterial::Image(ImageBoxImage {
            id: "redRadish.png".into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let container_props = HorizontalBoxProps {
        ..Default::default()
    };

    widget! {
        (horizontal_box: {container_props} [
            (image_box: {image_box1_props})
            (text_box: {text_box_props})
            (image_box: {image_box2_props})
        ])
    }
}
